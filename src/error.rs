use std::{result, rc::Rc, cell::RefCell};

use crate::{builtins::{object::Object, error::create_error}, value::Value};

pub type JSIResult<T> = result::Result<T, JSIError>;

#[derive(Debug,PartialEq,Clone)]
pub enum JSIErrorType {
  // 语法错误,遇到了不符合语法规范的标记（token）或标记顺序
  SyntaxError,
  // 类型错误
  TypeError,
  // 引用错误，不存在的变量
  ReferenceError,
  // 范围错误，如设置 array 的length为非数字
  RangeError,
  Unknown,
}

#[derive(Debug, Clone)]
pub struct JSIError {
    pub error_type: JSIErrorType,
    pub message: String,
    pub line: i32,
    pub column: i32,
    pub value: Option<Value>
}

impl JSIError {
    pub fn new(error_type: JSIErrorType, message: String, line: i32, column: i32) -> JSIError {
      return JSIError {
        error_type,
        message,
        line,
        column,
        value: None
      }
    }

    pub fn to_error_object(&self, global: &Rc<RefCell<Object>>) -> Rc<RefCell<Object>> {
      if let Some(value) = &self.value {
        return value.to_object(global);
      }
      let new_error = create_error(global, Value::String(self.message.clone()));
      // TODO: set error line/stack
      let obj = if let Value::Object(obj) = new_error {
        Some(obj)
      } else {
        None
      }.unwrap();
      return obj;
    }

    pub fn set_value(&mut self, value: Value) {
      self.value = Some(value);
    }
}