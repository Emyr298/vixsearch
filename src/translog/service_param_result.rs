use crate::translog::entity::Transaction;

pub struct InsertParam {
    pub collection_id: String,
    pub op_seq: u64,
    pub transaction: Transaction,
}
