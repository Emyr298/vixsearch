pub struct GetMetadataPortResult {
    pub offsets: Vec<u64>,
    pub filter_hash_cnt: u32,
    pub filter_bits: Vec<u64>,
    pub smallest_key: String,
    pub biggest_key: String,
}
