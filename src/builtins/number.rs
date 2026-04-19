use std::{rc::Rc};
use crate::constants::{GLOBAL_NUMBER_NAME, PROTO_PROPERTY_NAME};
use crate::context::{Context};
use crate::error::{JSIError, JSIErrorType};
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
fn to_string(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let value = value_of(call_ctx, vec![])?;

  // 获取 radix 参数
  let radix: u32 = if args.len() > 0 {
    if let Some(r) = args[0].to_number(call_ctx.ctx) {
      let r_int = r as i32;
      if r_int < 2 || r_int > 36 {
        return Err(JSIError::new(JSIErrorType::RangeError, String::from("toString() radix argument must be between 2 and 36"), 0, 0));
      }
      r_int as u32
    } else {
      10
    }
  } else {
    10
  };

  if let Value::Number(num) = value {
    if radix == 10 {
      Ok(Value::String(num.to_string()))
    } else {
      // 将数字转换为指定进制
      if num.is_nan() {
        Ok(Value::String(String::from("NaN")))
      } else if num.is_infinite() {
        if num > 0.0 {
          Ok(Value::String(String::from("Infinity")))
        } else {
          Ok(Value::String(String::from("-Infinity")))
        }
      } else {
        // 处理整数部分
        let int_part = num.trunc() as i64;
        let int_str = if int_part < 0 {
          format!("-{}", format_int_radix(-int_part, radix))
        } else {
          format_int_radix(int_part, radix)
        };
        Ok(Value::String(int_str))
      }
    }
  } else {
    Ok(Value::String(value.to_string(call_ctx.ctx)))
  }
}

// 辅助函数：将整数转换为指定进制的字符串
fn format_int_radix(n: i64, radix: u32) -> String {
  if n == 0 {
    return String::from("0");
  }
  let digits = "0123456789abcdefghijklmnopqrstuvwxyz";
  let mut result = String::new();
  let mut num = n;
  while num > 0 {
    let digit = (num % radix as i64) as usize;
    result.push(digits.chars().nth(digit).unwrap());
    num /= radix as i64;
  }
  result.chars().rev().collect()
}


// Number.prototype.valueOf
fn value_of(call_ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  if let Value::Number(_) = &call_ctx.this {
    return Ok(call_ctx.this.clone())
  }

  let number_obj = match &call_ctx.this {
    Value::Object(number) => {
      if let ClassType::Number = number.borrow().class_type  {
        Some(number)
      } else {
        None
      }
    },
    Value::NumberObj(number) => Some(number),
    _ => None,
  };

  if let Some(number) = number_obj {
    let init = number.borrow().get_inner_property_value(String::from("value"));
    if let Some(value) = init {
      let res = value.to_number(call_ctx.ctx);
      if let Some(num) = res {
        return Ok(Value::Number(num))
      }
    }
  }
  Err(JSIError::new(JSIErrorType::TypeError, format!("Number.prototype.valueOf requires that 'this' be a Number"), 0, 0))
}

fn create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut param = Value::Undefined;
  if args.len() > 0 {
    param = args[0].clone();
  }

  // 判断是否是构造函数调用
  // 当作为构造函数调用时，this 是新创建的 NumberObj
  // 当作为普通函数调用时，返回原始值
  let is_constructor_call = match &call_ctx.this {
    Value::NumberObj(_) => true,
    Value::Object(obj) => {
      // 检查是否是 Number 类型的对象
      obj.borrow().class_type == ClassType::Number
    },
    _ => false
  };

  if is_constructor_call {
    // 构造函数调用，返回 Number 对象
    Ok(create_number(call_ctx.ctx, param))
  } else {
    // 普通函数调用，返回原始数字值
    let num_value = param.to_number(call_ctx.ctx);
    if let Some(num) = num_value {
      Ok(Value::Number(num))
    } else {
      Ok(Value::Number(0f64))
    }
  }
}