use crate::utils::vixerr::Error;

#[derive(Debug)]
pub enum Ordering<ResultT> {
    Less,
    Equal(ResultT),
    NotFound,
    Greater,
}

pub fn binary_search<ValueT, CompareFnT, ResultT>(arr: &[ValueT], compare_fn: CompareFnT) -> Result<Option<ResultT>, Error>
where
    CompareFnT: Fn(&ValueT) -> Result<Ordering<ResultT>, Error>,
{
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;

        match compare_fn(&arr[mid])? {
            Ordering::Less => left = mid + 1,
            Ordering::Greater => right = mid,
            Ordering::Equal(value) => return Ok(Some(value)),
            Ordering::NotFound => return Ok(None),
        }
    }

    Ok(None)
}
