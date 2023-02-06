use std::{rc::Rc, cell::RefCell};

use crate::{value::Value, ast_node::{ClassType}};

use super::{object::{create_object, Object}, global::get_global_object};

 pub fn create_number(global: &Rc<RefCell<Object>>, init: Value) -> Value {
  let global_number = get_global_object(global, String::from("Number"));
  let number = create_object(global, ClassType::String, None);
  let number_clone = Rc::clone(&number);
  let mut number_mut = (*number_clone).borrow_mut();
  number_mut.constructor = Some(Rc::downgrade(&global_number));
  number_mut.set_inner_property_value(String::from("value"), init);
  Value::NumberObj(number)
}

pub fn bind_global_number(global:  &Rc<RefCell<Object>>) {
  let number_rc = get_global_object(global, String::from("Number"));
  let bool = (*number_rc).borrow_mut();
  if let Some(prop)= &bool.prototype {
    
  }
}
