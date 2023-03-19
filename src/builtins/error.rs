use std::{rc::Rc};
use crate::context::{Context};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, constants::GLOBAL_ERROR_NAME, error::JSIResult};

use super::{object::{create_object, Property}, global::get_global_object, function::builtin_function};

 // ref:https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-error-objects
 pub fn create_error(ctx: &Context, init: Value) -> Value {
  let global_error = get_global_object(ctx, GLOBAL_ERROR_NAME.to_string());
  let error = create_object(ctx, ClassType::Error, None);
  let error_clone = Rc::clone(&error);
  let mut error_mut = (*error_clone).borrow_mut();
  error_mut.constructor = Some(Rc::downgrade(&global_error));
  error_mut.define_property(String::from("message"),  Property { enumerable: true, value: Value::String(init.to_string(ctx)) });
  Value::Object(error)
}

pub fn bind_global_error(ctx: &Context, ) {
  let error_rc = get_global_object(ctx, GLOBAL_ERROR_NAME.to_string());
  let mut error = (*error_rc).borrow_mut();
  let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
  error.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  if let Some(prop)= &error.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    
  }
}

fn create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut param = Value::Undefined;
  if args.len() > 0 {
    param = args[0].clone();
  }
  Ok(create_error(call_ctx.ctx, param))
}