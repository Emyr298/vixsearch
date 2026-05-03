use super::entity::Point;

pub struct SearchResult {
    pub result: Vec<Point>,
}

impl SearchResult {
    pub fn new(result: Vec<Point>) -> Self {
        Self { result }
    }
}
