use std::result;

pub type JSIResult<T> = result::Result<T, Error>;
#[derive(Debug, Clone)]
pub struct Error {
    
}