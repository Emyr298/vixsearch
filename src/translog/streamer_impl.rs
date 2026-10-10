use std::sync::Arc;

use crate::{translog::{GetTransactionsParam, Transaction, TranslogPort, TranslogStreamer}, utils::vixerr::Error};

pub struct TranslogStreamerImpl {
    adapter: Arc<dyn TranslogPort>,
    collection_id: String,
    streamed_op_seq: u64,
}

impl TranslogStreamerImpl {
    pub fn new(adapter: Arc<dyn TranslogPort>, collection_id: String, committed_op_seq: u64) -> Box<dyn TranslogStreamer> {
        Box::new(TranslogStreamerImpl {
            adapter,
            collection_id,
            streamed_op_seq: committed_op_seq,
        })
    }
}

impl TranslogStreamer for TranslogStreamerImpl {
    fn get_transactions(&mut self, max_count: u64) -> Result<Vec<Transaction>, Error> {
        let result = self.adapter.get_transactions(GetTransactionsParam {
            collection_id: self.collection_id.clone(),
            from_op_seq: self.streamed_op_seq + 1,
            count: max_count,
        })?;

        self.streamed_op_seq = result.last_op_seq;
        Ok(result.transactions)
    }
}
