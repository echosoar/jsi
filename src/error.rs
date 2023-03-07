use std::result;

pub type JSIResult<T> = result::Result<T, JSIError>;

#[derive(Debug,PartialEq,Clone)]
pub enum JSIErrorType {
  // 语法错误,遇到了不符合语法规范的标记（token）或标记顺序
  SyntaxError,
}

#[derive(Debug, Clone)]
pub struct JSIError {
    pub error_type: JSIErrorType,
    pub message: String,
    pub line: i32,
    pub column: i32,
}

impl JSIError {
    pub fn new(error_type: JSIErrorType, message: String, line: i32, column: i32) -> JSIError {
      return JSIError {
        error_type,
        message,
        line,
        column
      }
    }
}