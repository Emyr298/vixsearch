use crate::{shared, utils::vixerr::Error};

pub trait Buffer<ParamT, ResultT> {
    fn search(&self, param: ParamT) -> Result<ResultT, Error>;
    fn insert(&self, id: &str, value: &shared::Value);
    fn flush(&self);
}
