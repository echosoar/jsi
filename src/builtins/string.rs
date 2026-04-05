use std::error::Error;
use std::{rc::Rc};
use crate::constants::{PROTO_PROPERTY_NAME, GLOBAL_STRING_NAME};
use crate::context::{Context};
use crate::error::{JSIError, JSIErrorType};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, error::JSIResult};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Property}, function::builtin_function};
use super::array::create_array;

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
    let name = String::from("charAt");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, char_at) });
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 0f64, to_string) });
    let name = String::from("valueOf");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, to_string) });
    let name = String::from("includes");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, string_includes) });
    let name = String::from("indexOf");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, string_index_of) });
    let name = String::from("trim");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 0f64, string_trim) });
    let name = String::from("startsWith");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, string_starts_with) });
    let name = String::from("endsWith");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, string_ends_with) });
    let name = String::from("slice");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 2f64, string_slice) });
    let name = String::from("toLowerCase");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 0f64, string_to_lower_case) });
    let name = String::from("toUpperCase");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 0f64, string_to_upper_case) });
    let name = String::from("split");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, string_split) });
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
  let string = get_string(call_ctx);
  if let Ok(str) = string {
    return Ok(Value::String(str))
  }
  Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.toString requires that 'this' be a String"), 0, 0))
}

// String.prototype.charAt
fn char_at(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  if call_ctx.this.is_not_strict_null() {
    return  Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.charAt called on null or undefined"), 0, 0))
  }
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("unknown error"), 0, 0))
  }
  let str = match string {
    Ok(str) => str,
    _ => String::from(""),
  };
  let mut index: usize = 0;
  if args.len() > 0 {
    let arg = &args[0];
    let index_number = arg.to_number(call_ctx.ctx);
    if let Some(index_f64) = index_number {
      index = index_f64 as usize;
    } else {
      let arg_type = arg.type_of();
      return Err(JSIError::new(JSIErrorType::TypeError, format!("Cannot convert {:?} to primitive value at String.charAt", arg_type), 0, 0))
    }
  }
  let utf16: Vec<u16> = str.as_str().encode_utf16().collect();
  if index < utf16.len() {
    let ch = char::from_u32(utf16[index] as u32).unwrap_or('\0');
    return Ok(Value::String(ch.to_string()))
  }

  return Ok(Value::String("".to_string()))
}

fn get_string(call_ctx: &mut CallContext) -> Result<String, Box<dyn Error>> {
  if let Value::String(str) = &call_ctx.this {
    return Ok(str.clone())
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
      return Ok(value.to_string(call_ctx.ctx))
    }
  }
  Err("error".into())
}

// String.prototype.includes
fn string_includes(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.includes called on incompatible receiver"), 0, 0))
  }
  let str = string.unwrap();

  let search_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    String::from("undefined")
  };

  let mut position: usize = 0;
  if args.len() > 1 {
    if let Some(pos) = args[1].to_number(call_ctx.ctx) {
      position = if pos < 0f64 { 0 } else { pos as usize };
    }
  }

  if position > str.len() {
    return Ok(Value::Boolean(false));
  }

  let result = str[position..].contains(&search_str);
  Ok(Value::Boolean(result))
}

// String.prototype.indexOf
fn string_index_of(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.indexOf called on incompatible receiver"), 0, 0))
  }
  let str = string.unwrap();

  let search_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    String::from("undefined")
  };

  let mut position: usize = 0;
  if args.len() > 1 {
    if let Some(pos) = args[1].to_number(call_ctx.ctx) {
      position = if pos < 0f64 { 0 } else { pos as usize };
    }
  }

  if position > str.len() {
    return Ok(Value::Number(-1f64));
  }

  let result = str[position..].find(&search_str);
  match result {
    Some(index) => Ok(Value::Number((position + index) as f64)),
    None => Ok(Value::Number(-1f64)),
  }
}

// String.prototype.trim
fn string_trim(call_ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.trim called on incompatible receiver"), 0, 0))
  }
  let str = string.unwrap();
  Ok(Value::String(str.trim().to_string()))
}

// String.prototype.startsWith
fn string_starts_with(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.startsWith called on incompatible receiver"), 0, 0))
  }
  let str = string.unwrap();

  let search_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    String::from("undefined")
  };

  let mut position: usize = 0;
  if args.len() > 1 {
    if let Some(pos) = args[1].to_number(call_ctx.ctx) {
      position = if pos < 0f64 { 0 } else { pos as usize };
    }
  }

  if position > str.len() {
    return Ok(Value::Boolean(false));
  }

  Ok(Value::Boolean(str[position..].starts_with(&search_str)))
}

// String.prototype.endsWith
fn string_ends_with(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.endsWith called on incompatible receiver"), 0, 0))
  }
  let str = string.unwrap();

  let search_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    String::from("undefined")
  };

  let mut end_position: usize = str.len();
  if args.len() > 1 {
    if let Some(pos) = args[1].to_number(call_ctx.ctx) {
      end_position = if pos < 0f64 { 0 } else { pos as usize };
    }
  }

  if end_position > str.len() {
    end_position = str.len();
  }

  Ok(Value::Boolean(str[..end_position].ends_with(&search_str)))
}

// String.prototype.slice
fn string_slice(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.slice called on incompatible receiver"), 0, 0))
  }
  let str = string.unwrap();
  let len = str.len() as i32;

  let mut start: i32 = 0;
  if args.len() > 0 {
    if let Some(pos) = args[0].to_number(call_ctx.ctx) {
      start = pos as i32;
      // 处理负数索引
      if start < 0 {
        start = len + start;
        if start < 0 {
          start = 0;
        }
      }
    }
  }

  let mut end: i32 = len;
  if args.len() > 1 {
    if let Some(pos) = args[1].to_number(call_ctx.ctx) {
      end = pos as i32;
      // 处理负数索引
      if end < 0 {
        end = len + end;
        if end < 0 {
          end = 0;
        }
      }
    }
  }

  if start > end {
    return Ok(Value::String(String::from("")));
  }

  if start > len {
    start = len;
  }
  if end > len {
    end = len;
  }

  let start = start as usize;
  let end = end as usize;

  Ok(Value::String(str[start..end].to_string()))
}

// String.prototype.toLowerCase
fn string_to_lower_case(call_ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.toLowerCase called on incompatible receiver"), 0, 0))
  }
  let str = string.unwrap();
  Ok(Value::String(str.to_lowercase()))
}

// String.prototype.toUpperCase
fn string_to_upper_case(call_ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.toUpperCase called on incompatible receiver"), 0, 0))
  }
  let str = string.unwrap();
  Ok(Value::String(str.to_uppercase()))
}

// String.prototype.split
fn string_split(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let string = get_string(call_ctx);
  if let Err(_) = string {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("String.prototype.split called on incompatible receiver"), 0, 0))
  }
  let str = string.unwrap();

  // 如果没有分隔符，返回包含原字符串的单元素数组
  if args.len() == 0 {
    let arr = create_array(call_ctx.ctx, 0);
    if let Value::Array(arr_obj) = &arr {
      let mut arr_mut = arr_obj.borrow_mut();
      arr_mut.define_property(String::from("0"), Property { enumerable: true, value: Value::String(str) });
      arr_mut.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(1f64) });
    }
    return Ok(arr);
  }

  let separator = &args[0];

  // 如果分隔符是 undefined，返回包含原字符串的单元素数组
  if let Value::Undefined = separator {
    let arr = create_array(call_ctx.ctx, 0);
    if let Value::Array(arr_obj) = &arr {
      let mut arr_mut = arr_obj.borrow_mut();
      arr_mut.define_property(String::from("0"), Property { enumerable: true, value: Value::String(str) });
      arr_mut.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(1f64) });
    }
    return Ok(arr);
  }

  // 如果分隔符是空字符串，将每个字符作为数组元素
  let sep_str = separator.to_string(call_ctx.ctx);
  if sep_str.is_empty() {
    let arr = create_array(call_ctx.ctx, 0);
    if let Value::Array(arr_obj) = &arr {
      let mut arr_mut = arr_obj.borrow_mut();
      let chars: Vec<char> = str.chars().collect();
      for (i, ch) in chars.iter().enumerate() {
        arr_mut.define_property(i.to_string(), Property { enumerable: true, value: Value::String(ch.to_string()) });
      }
      arr_mut.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(chars.len() as f64) });
    }
    return Ok(arr);
  }

  // 正常分割
  let parts: Vec<&str> = str.split(&sep_str).collect();
  let arr = create_array(call_ctx.ctx, 0);
  if let Value::Array(arr_obj) = &arr {
    let mut arr_mut = arr_obj.borrow_mut();
    for (i, part) in parts.iter().enumerate() {
      arr_mut.define_property(i.to_string(), Property { enumerable: true, value: Value::String(part.to_string()) });
    }
    arr_mut.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(parts.len() as f64) });
  }
  Ok(arr)
}