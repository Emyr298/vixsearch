use std::{collections::HashMap, fmt::{Display, Formatter, Result}};

use crate::shared::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollectionID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SegmentID(pub String);

impl Display for SegmentID {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentID(pub String);

impl Display for DocumentID {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentSeqID(pub u64);

impl Display for DocumentSeqID {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: DocumentID,
    pub seq_id: DocumentSeqID,
    pub payload: HashMap<String, Value>,
}
