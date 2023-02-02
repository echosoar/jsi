use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{value::Value};
// 上下文环境
#[derive(Debug, Clone)]
pub struct Scope {
  pub id: i32,
  pub parent: Option<Rc<RefCell<Scope>>>,
  pub from: Option<Rc<RefCell<Scope>>>,
  pub childs: Vec<Rc<RefCell<Scope>>>,
  variables: HashMap<String, Value>
}

impl Scope {
  pub fn new() -> Scope {
    Scope {
      id: 0,
      childs: vec![],
      parent: None,
      from: None,
      variables: HashMap::new(),
    }
  }

  pub fn set_value(&mut self, name: String, value: Value) {
    self.variables.insert(name, value);
  }
}

pub fn get_value_and_scope(scope: Rc<RefCell<Scope>>, identifier: String) -> (Option<Value>, Rc<RefCell<Scope>>) {
  // println!("get_value_and_scope: {:?} {:?}", identifier, scope);
  let s = scope.borrow();
  let value = s.variables.get(&identifier);
  if let Some(val) = value {
    return (Some(val.clone()), Rc::clone(&scope))
  } else {
    if let Some(parent) = &scope.borrow().parent {
      get_value_and_scope(Rc::clone(parent), identifier)
    } else {
      (None, Rc::clone(&scope))
    }
  }
}