pub struct Operation {
    pub collection_id: String,
    pub key: String,
    pub op_seq: u64,
    pub value: Vec<u8>,
}
