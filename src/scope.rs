use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{bytecode::ByteCode, value::{Value, ValueInfo}};
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
  pub function_call_args: Vec<ValueInfo>,
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

pub fn get_value_info_and_scope(scope: Rc<RefCell<Scope>>, identifier: String) -> (Option<ValueInfo>, Rc<RefCell<Scope>>, bool) {
  // println!("get_value_and_scope: {:?} {:?}", identifier, scope);
  let s = scope.borrow();
  let value = s.variables.get(&identifier);
  if let Some(val) = value {
    let value_info = ValueInfo {
      name: Some(identifier.clone()),
      value: val.value.clone(),
      is_const: val.is_const,
      reference: None,
      access_path: identifier.clone(),
    };
    return (Some(value_info), Rc::clone(&scope), val.is_const)
  } else {
    if let Some(parent) = &scope.borrow().parent {
      get_value_info_and_scope(Rc::clone(parent), identifier)
    } else {
      (None, Rc::clone(&scope), false)
    }
  }
}

pub fn get_value_and_scope(scope: Rc<RefCell<Scope>>, identifier: String) -> (Option<Value>, Rc<RefCell<Scope>>, bool) {
  let (value_info, scope, is_const) = get_value_info_and_scope(scope, identifier);
  if let Some(value_info) = value_info {
    return (Some(value_info.value), scope, is_const);
  }
  // 如果没有找到变量，返回 None
  (None, scope, false)
}