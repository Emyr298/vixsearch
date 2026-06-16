use crate::{index, query as qry, utils::vixerr::Error};

pub trait Buffer: 'static {
    fn search(&self, query: Box<dyn qry::SearchQuery>) -> Result<index::SearchResult, Error>;
    fn insert(&self, lookup: Box<dyn index::Lookup>, entry: Box<dyn index::Entry>) -> Result<(), Error>;
    fn flush(&self);
}
