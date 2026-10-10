use std::sync::Arc;

use crate::{errcode::FATAL_ERROR, translog::{GetTransactionsParam, Transaction, TranslogPort, TranslogStreamer, state::CollectionState}, utils::vixerr::Error};

pub struct TranslogStreamerImpl {
    adapter: Arc<dyn TranslogPort>,
    state: Arc<CollectionState>,
    collection_id: String,
    streamed_op_seq: u64,
}

impl TranslogStreamerImpl {
    pub fn new(adapter: Arc<dyn TranslogPort>, state: Arc<CollectionState>, collection_id: String, last_committed_op_seq: u64) -> Box<dyn TranslogStreamer> {
        Box::new(TranslogStreamerImpl {
            adapter,
            state,
            collection_id,
            streamed_op_seq: last_committed_op_seq,
        })
    }
}

impl TranslogStreamer for TranslogStreamerImpl {
    fn get_transactions(&mut self, max_count: u64) -> Result<Vec<Transaction>, Error> {
        let mut result = Vec::<Transaction>::new();

        let mut segment_start_op_seq = self.streamed_op_seq + 1;
        let mut streamed_op_seq = self.streamed_op_seq;
        let mut iter_max_count = max_count;

        while iter_max_count > 0 {
            let segment = match self.state.segment_by_op_seq(segment_start_op_seq) {
                Some(s) => s,
                None => return Error::code(FATAL_ERROR)
                    .message(format!("No segment found for op_seq: {} in collection: {}", segment_start_op_seq, self.collection_id))
                    .throw(),
            };

            let get_result = self.adapter.get_transactions(GetTransactionsParam {
                collection_id: self.collection_id.clone(),
                from_op_seq: segment_start_op_seq,
                count: max_count,
            })?;

            iter_max_count -= get_result.transactions.len() as u64;
            segment_start_op_seq = get_result.last_op_seq + 1;
            streamed_op_seq = get_result.last_op_seq;

            result.extend(get_result.transactions);
        }

        self.streamed_op_seq = streamed_op_seq;
        Ok(result)
    }
}
