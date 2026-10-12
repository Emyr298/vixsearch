use std::{sync::{Arc, mpsc::{self, Receiver, SyncSender}}, thread};

use crate::{errcode::{FATAL_ERROR, SYSTEM_ERROR}, translog::{GetTransactionsParam, Transaction, TranslogPort, TranslogStreamer, error::LOG_TAIL_REACHED, state::CollectionState}, utils::vixerr::Error};

pub struct TranslogStreamerImpl {
    collection_id: String,
    adapter: Arc<dyn TranslogPort>,
    state: Arc<CollectionState>,
    receiver: Receiver<Transaction>,
}

impl TranslogStreamerImpl {
    pub fn new(
        collection_id: &str,
        adapter: Arc<dyn TranslogPort>,
        state: Arc<CollectionState>,
        start_op_seq: u64,
        is_continuous: bool,
        max_batch: usize,
        max_outstanding: usize,
    ) -> Box<dyn TranslogStreamer> {
        let (sender, receiver) = mpsc::sync_channel::<Transaction>(max_outstanding);

        let streamer = Box::new(TranslogStreamerImpl {
            adapter,
            state,
            collection_id: collection_id.to_string(),
            receiver,
        });

        streamer.stream_trx_logs(start_op_seq, sender, max_batch, max_outstanding, is_continuous);

        streamer
    }

    fn stream_trx_logs(&self, start_op_seq: u64, sender: SyncSender<Transaction>, max_batch: usize, max_outstanding: usize, is_continuous: bool) {
        let mut snapshot = self.state.snapshot_segments();

        let mut cur_seg_idx = snapshot.iter()
            .rposition(|s| s.minimum_op_seq >= start_op_seq)
            .unwrap_or(0);

        let mut last_streamed_op_seq = start_op_seq - 1;

        let collection_id = self.collection_id.clone();
        let adapter = self.adapter.clone();
        let state = self.state.clone();

        thread::spawn(move || {
            loop {
                if cur_seg_idx >= snapshot.len() {
                    if !is_continuous {
                        return // sender will be dropped here, receiver will get error after last message
                    }

                    // refresh snapshot
                    snapshot = state.snapshot_segments(); // can't be zero in length
                    cur_seg_idx = snapshot.iter()
                        .rposition(|s| s.minimum_op_seq >= last_streamed_op_seq + 1)
                        .unwrap_or(0);
                }

                let seg = &snapshot[cur_seg_idx];

                let result = match adapter.get_transactions(GetTransactionsParam {
                    collection_id: collection_id.clone(),
                    segment_id: seg.id.clone(),
                    from_op_seq: last_streamed_op_seq + 1,
                    max_count: max_batch,
                }) {
                    Ok(r) => r,
                    Err(e) if e.code != LOG_TAIL_REACHED => {
                        eprintln!("failed to adapter.get_transactions: {}", e);
                        return
                    },
                    Err(e) => {
                        if !state.is_active_segment(&seg.id) {
                            cur_seg_idx += 1;
                        }
                        continue
                    },
                };

                for trx in result.transactions {
                    if let Err(e) = sender.send(trx) {
                        eprintln!("failed to sender.send: {}", e);
                        return
                    }
                }

                last_streamed_op_seq = result.last_op_seq
            }
        });
    }
}

impl TranslogStreamer for TranslogStreamerImpl {
    fn recv_trx_log(&mut self, max_count: u64) -> Result<Transaction, Error> {
        match self.receiver.recv() {
            Ok(t) => Ok(t),
            Err(e) => Error::code(SYSTEM_ERROR)
                .message("sender is already closed")
                .wrap(e)
                .throw(),
        }
    }
}
