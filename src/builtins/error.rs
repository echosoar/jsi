use std::{rc::Rc};
use crate::constants::{PROTO_PROPERTY_NAME, GLOBAL_TYPE_ERROR_NAME};
use crate::context::{Context};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, constants::GLOBAL_ERROR_NAME, error::JSIResult};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Property},function::builtin_function};

 // ref:https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-error-objects
 // 实例化 Error 对象
 pub fn create_error(ctx: &mut Context, init: Value, error_type: &str) -> Value {
  let global_error = get_global_object_by_name(ctx, error_type);
  let error = create_object(ctx, ClassType::Error, None);
  let error_clone = Rc::clone(&error);
  let mut error_mut = (*error_clone).borrow_mut();

  // 将实例化的 error 对象的 __proto__ 指向全局的 Error.prototype
  let global_prototype = get_global_object_prototype_by_name(ctx, error_type);
  error_mut.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&global_prototype)));

  error_mut.constructor = Some(Rc::downgrade(&global_error));

  let msg =  init.to_string(ctx);

  error_mut.define_property(String::from("message"),  Property { enumerable: true, value: Value::String(msg)});
  Value::Object(error)
}

pub fn bind_global_error(ctx: &mut Context, error_type: &str) {
  // Error
  let create_function = builtin_function(ctx, error_type.to_string(), 1f64, create);

  let error_rc = get_global_object_by_name(ctx, error_type);
  let mut error = (*error_rc).borrow_mut();
  error.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  if error_type != GLOBAL_ERROR_NAME {
    return
  }
  if let Some(prop)= &error.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 0f64, to_string) });
  }
}

// 创建实例化对象
fn create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut param = Value::Undefined;
  if args.len() > 0 {
    param = args[0].clone();
  }
  Ok(create_error(call_ctx.ctx, param, call_ctx.func_name.as_str()))
}

// Error.prototype.toString
fn to_string(_: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  // let this = call_ctx.this;
  // TODO:
  Ok(Value::String(String::from("err")))
}