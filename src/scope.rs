use std::collections::HashMap;

use crate::{value::Value};
// 上下文环境
#[derive(Debug, Clone)]
pub struct Scope {
  pub parent: Option<Box<Scope>>,
  variables: HashMap<String, Value>
}

impl Scope {
  pub fn new() -> Scope {
    Scope {
      parent: None,
      variables: HashMap::new(),
    }
  }

  pub fn set_value(&mut self, name: String, value: Value) {
    self.variables.insert(name, value);
  }
}

pub fn get_value_by_scope(scope: &Scope, identifier: String) -> Option<&Value> {
  scope.variables.get(&identifier)
}