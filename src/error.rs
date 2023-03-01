use std::result;

pub type JSIResult<T> = result::Result<T, JSIError>;
#[derive(Debug, Clone)]
pub struct JSIError {
    
}