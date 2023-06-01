use std::cell::{RefCell};
use std::{rc::Rc};
use crate::constants::{PROTO_PROPERTY_NAME, GLOBAL_ARRAY_NAME};
use crate::context::{Context};
use crate::{value::Value, ast_node::{CallContext, ClassType}, error::{JSIResult, JSIError, JSIErrorType}};

use super::function::builtin_function;
use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::object::Object;
use super::{object::{create_object, Property}};

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
    prototype.define_builtin_function_property(ctx, String::from("concat"),  0, array_concat);
    prototype.define_builtin_function_property(ctx, String::from("join"),  1, array_join);
    prototype.define_builtin_function_property(ctx, String::from("push"),  1, array_push);
    prototype.define_builtin_function_property(ctx, String::from("toString"),  0, array_to_string);
  }
}

// Array.isArray
fn array_static_is_array(call_ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  match &call_ctx.this {
    Value::Array(array) => {
      let arrborrowed =  array.borrow();
      if let  ClassType::Array = arrborrowed.class_type {
        Ok(Value::Boolean(true))
      } else {
        Ok(Value::Boolean(false))
      }
    },
    _ =>  Ok(Value::Boolean(false)),
  }
}

// Array.prototype.concat
fn array_concat(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let new_arr_info = clone_array_object(call_ctx)?;
  let mut len = new_arr_info.1;
  let new_array = new_arr_info.0;
  let new_array_clone = Rc::clone(&new_array);
  let mut new_array_borrowed = new_array_clone.borrow_mut();
  for arg in args.iter() {
    match arg {
      Value::Array(arr) => {
        let iter = |_: i32, value: &Value, _: &mut Context| {
          new_array_borrowed.define_property(format!("{}", len), Property { enumerable: true, value: value.clone() });
          len += 1;
        };
        array_iter_mut(call_ctx, &arr,  iter);
      },
      _ => {
        new_array_borrowed.define_property(format!("{}", len), Property { enumerable: true, value: arg.clone() });
        len += 1;
      }
    }
  }
  new_array_borrowed.define_property(String::from("length"),  Property { enumerable: false, value: Value::Number(len.clone() as f64) });
  Ok(Value::Array(new_array))
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
  let this_array_obj = match &call_ctx.this {
    Value::Array(array) => {
      Some(Rc::clone(array))
    },
    _ => None,
  };
  if let Some(this_ref) = this_array_obj {
    array_iter_mut(call_ctx, &this_ref,  iter);
  }
 
  Ok(Value::String(string_list.join(join)))
}



// Array.prototype.push
fn array_push(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {

  let this_array_obj = get_array_object_from_this(&call_ctx.this);

  if let Some(this_ref) = this_array_obj {
    let mut this = this_ref.borrow_mut();
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
    return Err(JSIError::new(JSIErrorType::RangeError, format!("Invalid array length"), 0, 0))
  }
  return Ok(Value::Number(args.len() as f64));
}



// Array.prototype.toString
fn array_to_string(ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  array_join(ctx, vec![])
}

fn array_iter_mut<F: FnMut(i32, &Value, &mut Context)>(call_ctx: &mut CallContext, arr_rc: &Rc<RefCell<Object>>, mut callback: F) {
  let arr = arr_rc.borrow_mut();
  let len = arr.get_property_value(String::from("length"));
  if let Value::Number(len) = len {
    for index in 0..(len as i32) {
      (callback)(index, &arr.get_property_value(index.to_string()), call_ctx.ctx);
    }
  }
}

// 复制数组对象
fn clone_array_object(call_ctx: &mut CallContext) -> JSIResult<(Rc<RefCell<Object>>, i32)> {
  let new_arr = match create_array(call_ctx.ctx, 0) {
    Value::Array(arr) => Some(arr),
    _ => None
  }.unwrap();
  let new_arr_rc = Rc::clone(&new_arr);
  let mut new_arr_borrowed = new_arr_rc.borrow_mut();
  
  let mut length: i32 = 0;
  let this_array_obj = get_array_object_from_this(&call_ctx.this);
  if let Some(this_ref) = this_array_obj {
    let this = this_ref.borrow_mut();
    let length_value = this.get_property_value(String::from("length"));
    match length_value {
      Value::Number(len) => {
        length = len as i32;
      },
      _ => {},
    };
    for index in 0..(length as i32) {
      let value = this.get_property_value(index.to_string());
      new_arr_borrowed.define_property(index.to_string(), Property { enumerable: true, value: value.clone() });
    }
  } else {
    length = 1;
    new_arr_borrowed.define_property(String::from("0"), Property { enumerable: true, value: call_ctx.this.clone() });
  }

  
  new_arr_borrowed.define_property(String::from("length"),  Property { enumerable: false, value: Value::Number(length.clone() as f64) });
  Ok((new_arr, length))
}

pub fn create_list_from_array_list(call_ctx: &mut CallContext,value: &Value) -> JSIResult<Vec<Value>> {
  let mut list: Vec<Value> = vec![];
  match value {
    Value::Boolean(_) | Value::Number(_) | Value::String(_) | Value::NAN => Err(JSIError::new(JSIErrorType::TypeError, format!("eateListFromArrayLike called on non-object
    "), 0, 0)),
    Value::Null | Value::Undefined => Ok(list),
    _ => {
      let obj = value.to_object(call_ctx.ctx);
      let obj_borrow = obj.borrow();
      let len = obj_borrow.get_property_value(String::from("length"));
      if let Value::Number(len) = len {
        for index in 0..(len as i32) {
          let value = obj_borrow.get_property_value(index.to_string());
          list.push(value);
        }
      }
      Ok(list)
    },
  }
}

fn get_array_object_from_this<'a>(this_value: &'a Value) -> Option<&'a Rc<RefCell<Object>>>{
  match &this_value {
    Value::Array(array) | Value::Object(array) => {
      Some(array)
    },
    _ => None,
  }
}