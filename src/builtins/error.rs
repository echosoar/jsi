use std::{rc::Rc, cell::RefCell};

use crate::{value::Value, ast_node::{ClassType}};

use super::{object::{create_object, Object}, global::get_global_object, function::builtin_function};

 // ref:https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-error-objects
 pub fn create_error(global: &Rc<RefCell<Object>>, init: Value) -> Value {
  let global_error = get_global_object(global, String::from("Error"));
  let error = create_object(global, ClassType::Error, None);
  let error_clone = Rc::clone(&error);
  let mut error_mut = (*error_clone).borrow_mut();
  error_mut.constructor = Some(Rc::downgrade(&global_error));
  error_mut.set_inner_property_value(String::from("value"), init);
  Value::Object(error)
}

pub fn bind_global_error(global:  &Rc<RefCell<Object>>) {
  let error_rc = get_global_object(global, String::from("Error"));
  let error = (*error_rc).borrow_mut();
  if let Some(prop)= &error.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    
  }
}