use std::{rc::Rc};
use crate::constants::{PROTO_PROPERTY_NAME, GLOBAL_STRING_NAME};
use crate::context::{Context};
use crate::error::{JSIError, JSIErrorType};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, error::JSIResult};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Property}, function::builtin_function};

 pub fn create_string(ctx: &mut Context, init: Value) -> Value {
  let global_string = get_global_object_by_name(ctx, GLOBAL_STRING_NAME);
  let string = create_object(ctx, ClassType::String, None);
  let string_clone = Rc::clone(&string);
  let mut string_mut = (*string_clone).borrow_mut();
  string_mut.constructor = Some(Rc::downgrade(&global_string));
  if init.is_string() {
    string_mut.set_inner_property_value(String::from("value"), init);
  } else {
    string_mut.set_inner_property_value(String::from("value"), Value::String(init.to_string(ctx)));
  }
  

  let global_prototype = get_global_object_prototype_by_name(ctx, GLOBAL_STRING_NAME);
  string_mut.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&global_prototype)));
  Value::StringObj(string)
}

pub fn bind_global_string(ctx: &mut Context) {
  let string_rc = get_global_object_by_name(ctx, GLOBAL_STRING_NAME);
  let mut string = (*string_rc).borrow_mut();
  let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
  string.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  if let Some(prop)= &string.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = (*prototype_rc).borrow_mut();
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 0f64, to_string) });
    let name = String::from("valueOf");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, to_string) });
  }
}


fn create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut param = Value::Undefined;
  if args.len() > 0 {
    param = args[0].clone();
  }
  Ok(create_string(call_ctx.ctx, param))
}

// String.prototype.toString
fn to_string(call_ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  if let Value::String(_) = &call_ctx.this {
    return Ok(call_ctx.this.clone())
  }

  let string_obj = match &call_ctx.this {
    Value::Object(string) => {
      if let ClassType::String = string.borrow().class_type  {
        Some(string)
      } else {
        None
      }
    },
    Value::StringObj(string) => Some(string),
    _ => None,
  };
  if let Some(str) = string_obj {
    let init = str.borrow().get_inner_property_value(String::from("value"));
    if let Some(value) = init {
      return Ok(Value::String(value.to_string(call_ctx.ctx)))
    }
  }
  Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.toString requires that 'this' be a String"), 0, 0))
}