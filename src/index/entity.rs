pub type IndexType = &'static str;

pub fn field_key(collection: &str, field: &str) -> String {
    format!("{}:{}", collection, field)
}
