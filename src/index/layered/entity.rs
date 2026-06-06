pub fn buffer_key(collection: &str, field: &str) -> String {
    return format!("{}:{}", collection, field);
}
