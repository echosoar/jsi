use std::{rc::Rc};
use crate::constants::{GLOBAL_NUMBER_NAME, PROTO_PROPERTY_NAME};
use crate::context::{Context};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, error::JSIResult};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Property}, function::builtin_function};

 pub fn create_number(ctx: &mut Context, init: Value) -> Value {
  let global_number = get_global_object_by_name(ctx, GLOBAL_NUMBER_NAME);
  let number = create_object(ctx, ClassType::Number, None);
  let number_clone = Rc::clone(&number);
  let mut number_mut = (*number_clone).borrow_mut();
  number_mut.constructor = Some(Rc::downgrade(&global_number));
  number_mut.set_inner_property_value(String::from("value"), init);

  let global_prototype = get_global_object_prototype_by_name(ctx, GLOBAL_NUMBER_NAME);
  number_mut.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&global_prototype)));
  Value::NumberObj(number)
}

pub fn bind_global_number(ctx: &mut Context) {
  let number_rc = get_global_object_by_name(ctx, GLOBAL_NUMBER_NAME);
  let mut number = (*number_rc).borrow_mut();
  let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
  number.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  if let Some(prop)= &number.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = (*prototype_rc).borrow_mut();
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 0f64, to_string) });
    let name = String::from("valueOf");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, value_of) });
  }
}

// Number.prototype.toString
fn to_string(call_ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  let value = value_of(call_ctx, vec![])?;
  Ok(Value::String(value.to_string(call_ctx.ctx)))
}


// Number.prototype.valueOf
fn value_of(call_ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  let this_origin = call_ctx.this.upgrade();
  let this_rc = this_origin.unwrap();
  let init = this_rc.borrow().get_inner_property_value(String::from("value"));
  if let Some(value) = init {
    let res = value.to_number(call_ctx.ctx);
    if let Some(num) = res {
      return Ok(Value::Number(num))
    }
  }
  Ok(Value::NAN)
}

fn create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut param = Value::Undefined;
  if args.len() > 0 {
    param = args[0].clone();
  }
  Ok(create_number(call_ctx.ctx, param))
}