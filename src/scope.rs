use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{bytecode::ByteCode, value::Value};
// 上下文环境
#[derive(Debug, Clone)]
pub struct Scope {
  pub id: i32,
  pub parent: Option<Rc<RefCell<Scope>>>,
  pub from: Option<Rc<RefCell<Scope>>>,
  pub childs: Vec<Rc<RefCell<Scope>>>,
  pub labels: Vec<String>,
  // 当前上下文的 this
  pub this: Option<Value>,
  variables: HashMap<String, VariableInfo>,
  pub function_call_args: Vec<Value>,
}


#[derive(Debug, Clone)]
pub struct VariableInfo {
  pub value: Value,
  pub is_const: bool,
  pub bytecode: Vec<ByteCode>,
}

impl Scope {
  pub fn new() -> Scope {
    Scope {
      id: 0,
      childs: vec![],
      parent: None,
      from: None,
      this: None,
      labels: vec![],
      variables: HashMap::new(),
      function_call_args: vec![],
    }
  }

  pub fn set_value(&mut self, name: String, value: Value, is_const: bool) {
    self.variables.insert(name, VariableInfo{value, is_const, bytecode: vec![]});
  }

  pub fn set_bytecode(&mut self, name: String, value: Value, is_const: bool, bytecode: Vec<ByteCode>) {
    self.variables.insert(name, VariableInfo{value, is_const, bytecode});
  }
}

pub fn get_value_and_scope(scope: Rc<RefCell<Scope>>, identifier: String) -> (Option<Value>, Rc<RefCell<Scope>>, bool) {
  // println!("get_value_and_scope: {:?} {:?}", identifier, scope);
  let s = scope.borrow();
  let value = s.variables.get(&identifier);
  if let Some(val) = value {
    return (Some(val.value.clone()), Rc::clone(&scope), val.is_const)
  } else {
    if let Some(parent) = &scope.borrow().parent {
      get_value_and_scope(Rc::clone(parent), identifier)
    } else {
      (None, Rc::clone(&scope), false)
    }
  }
}