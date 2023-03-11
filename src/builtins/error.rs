use std::{rc::Rc, cell::RefCell};

use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, constants::GLOBAL_ERROR_NAME};

use super::{object::{create_object, Object, Property}, global::get_global_object, function::builtin_function};

 // ref:https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-error-objects
 pub fn create_error(global: &Rc<RefCell<Object>>, init: Value) -> Value {
  let global_error = get_global_object(global, GLOBAL_ERROR_NAME.to_string());
  let error = create_object(global, ClassType::Error, None);
  let error_clone = Rc::clone(&error);
  let mut error_mut = (*error_clone).borrow_mut();
  error_mut.constructor = Some(Rc::downgrade(&global_error));
  error_mut.define_property(String::from("message"),  Property { enumerable: true, value: Value::String(init.to_string(global)) });
  Value::Object(error)
}

pub fn bind_global_error(global:  &Rc<RefCell<Object>>) {
  let error_rc = get_global_object(global, GLOBAL_ERROR_NAME.to_string());
  let mut error = (*error_rc).borrow_mut();
  let create_function = builtin_function(global, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
  error.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  if let Some(prop)= &error.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    
  }
}

fn create(ctx: &mut CallContext, args: Vec<Value>) -> Value {
  let mut param = Value::Undefined;
  if args.len() > 0 {
    param = args[0].clone();
  }
  let global = ctx.global.upgrade();
  if let Some(global) = &global {
    create_error(global, param)
  } else {
    Value::Undefined
  }
}