pub struct Operation {
    pub collection_id: String,
    pub key: String,
    pub op_seq: i64,
    pub value: Vec<u8>,
}
