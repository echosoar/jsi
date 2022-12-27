use std::collections::HashMap;

use crate::value::Value;
// 上下文环境
pub struct Scope {
  variables: HashMap<String, Value>
}

impl Scope {
  pub fn new() -> Scope {
    Scope {
      variables: HashMap::new(),
    }
  }
  
  pub fn set_value(&mut self, name: String, value: Value) {
    self.variables.insert(name, value);
  }
}