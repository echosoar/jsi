use std::{rc::Rc, cell::RefCell};

use crate::{value::Value, ast_node::{ClassType}};

use super::{object::{create_object, Object}, global::get_global_object};

 pub fn create_string(global: &Rc<RefCell<Object>>, init: Value) -> Value {
  let global_string = get_global_object(global, String::from("String"));
  let string = create_object(global, ClassType::String, None);
  let string_clone = Rc::clone(&string);
  let mut string_mut = (*string_clone).borrow_mut();
  string_mut.constructor = Some(Rc::downgrade(&global_string));
  string_mut.set_inner_property_value(String::from("value"), init);
  Value::StringObj(string)
}

pub fn bind_global_string(global:  &Rc<RefCell<Object>>) {
  let string_rc = get_global_object(global, String::from("String"));
  let bool = (*string_rc).borrow_mut();
  if let Some(prop)= &bool.prototype {
    
  }
}
