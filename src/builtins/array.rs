use std::{rc::Rc};
use crate::constants::{PROTO_PROPERTY_NAME, GLOBAL_ARRAY_NAME};
use crate::context::{Context};
use crate::{value::Value, ast_node::{CallContext, ClassType}, error::{JSIResult, JSIError, JSIErrorType}};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Property}, function::builtin_function};

 // ref: https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#prod-FunctionDeclaration
 pub fn create_array(ctx: &mut Context, length: usize) -> Value {
  let global_array = get_global_object_by_name(ctx, GLOBAL_ARRAY_NAME);
  let array = create_object(ctx, ClassType::Array, None);
  let array_clone = Rc::clone(&array);
  let mut array_mut = (*array_clone).borrow_mut();
  array_mut.define_property(String::from("length"),  Property { enumerable: true, value: Value::Number(length as f64) });
  
  let global_prototype = get_global_object_prototype_by_name(ctx, GLOBAL_ARRAY_NAME);
  array_mut.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&global_prototype)));
  // 绑定 fun.constructor = global.Array
  array_mut.constructor = Some(Rc::downgrade(&global_array));
  Value::Array(array)
}

pub fn create_array_from_values(ctx: &mut Context, values: Vec<Value>) -> Value {
  let new_array = create_array(ctx, 0);
  if let Value::Array(arr_obj) = &new_array {

    let mut arr = arr_obj.borrow_mut();
    arr.define_property(String::from("length"),  Property { enumerable: false, value: Value::Number(values.len() as f64) });
    let mut index = 0;
    for value in values.iter() { 
      arr.define_property(index.to_string(), Property { enumerable: true, value: value.clone() });
      index += 1
    }
  }
  new_array
}

pub fn bind_global_array(ctx: &mut Context) {
  let arr_rc = get_global_object_by_name(ctx, GLOBAL_ARRAY_NAME);
  let mut arr = (*arr_rc).borrow_mut();
  let name = String::from("isArray");
  arr.property.insert(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, array_static_is_array) });

  if let Some(prop)= &arr.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 0f64, array_to_string) });
    let name = String::from("join");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, array_join) });
    let name = String::from("push");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, array_push) });
  }
}

// Array.isArray
fn array_static_is_array(ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  let this_origin = ctx.this.upgrade();
  let this_rc = this_origin.unwrap();
  let this = this_rc.borrow();
  if let ClassType::Array = this.class_type {
    Ok(Value::Boolean(true))
  } else {
    Ok(Value::Boolean(false))
  }
}


// Array.prototype.toString
fn array_to_string(ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  array_join(ctx, vec![])
}

// Array.prototype.join
fn array_join(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut join = ",";
  if args.len() > 0 {
    if let Value::String(join_param) = &args[0] {
      join = join_param;
    }
  }
  let mut string_list: Vec<String> = vec![];
  let iter = |_: i32, value: &Value, ctx: &mut Context| {
    string_list.push(value.to_string(ctx));
  };
  array_iter_mut(call_ctx, iter);
  Ok(Value::String(string_list.join(join)))
}

fn array_iter_mut<F: FnMut(i32, &Value, &mut Context)>(call_ctx: &mut CallContext, mut callback: F) {
  let this_origin = call_ctx.this.upgrade();
  let this_rc = this_origin.unwrap();
  let this = this_rc.borrow_mut();
  let len = this.get_property_value(String::from("length"));
  if let Value::Number(len) = len {
    for index in 0..(len as i32) {
      (callback)(index, &this.get_property_value(index.to_string()), call_ctx.ctx);
    }
  }
}

// Array.prototype.push
fn array_push(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  // 插入值
  let this_rc = call_ctx.this.upgrade().unwrap();
  let mut this = this_rc.borrow_mut();
  let len_opt = this.get_property_value(String::from("length")).to_number(call_ctx.ctx);
  if let Some(len) = len_opt {
    let mut len = len as usize;
    for value in args.iter() { 
      this.define_property(len.to_string(), Property { enumerable: true, value: value.clone() });
      len += 1
    }
    let new_length = Value::Number(len as f64);
    this.define_property(String::from("length"),  Property { enumerable: false, value: new_length.clone() });
    return Ok(new_length)
  }
  Err(JSIError::new(JSIErrorType::RangeError, format!("Invalid array length"), 0, 0))
}

