use std::fmt::{Display, Formatter, Result};

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
