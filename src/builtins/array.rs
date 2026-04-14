use std::cell::{RefCell};
use std::{rc::Rc};
use crate::constants::{PROTO_PROPERTY_NAME, GLOBAL_ARRAY_NAME};
use crate::context::{Context};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{CallContext, ClassType}, error::{JSIResult, JSIError, JSIErrorType}};

use super::function::builtin_function;
use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::object::Object;
use super::{object::{create_object, Property}};

// Pre-generated string representations for small integers (0-999)
// This avoids repeated to_string() allocations for common array indices
fn index_to_string(index: i32) -> String {
  if index >= 0 && index < 100 {
    const SMALL_INT_STRINGS: [&str; 100] = [
      "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
      "10", "11", "12", "13", "14", "15", "16", "17", "18", "19",
      "20", "21", "22", "23", "24", "25", "26", "27", "28", "29",
      "30", "31", "32", "33", "34", "35", "36", "37", "38", "39",
      "40", "41", "42", "43", "44", "45", "46", "47", "48", "49",
      "50", "51", "52", "53", "54", "55", "56", "57", "58", "59",
      "60", "61", "62", "63", "64", "65", "66", "67", "68", "69",
      "70", "71", "72", "73", "74", "75", "76", "77", "78", "79",
      "80", "81", "82", "83", "84", "85", "86", "87", "88", "89",
      "90", "91", "92", "93", "94", "95", "96", "97", "98", "99"
    ];
    SMALL_INT_STRINGS[index as usize].to_string()
  } else {
    index.to_string()
  }
}

// 将字符串转换为数值索引，如果不是有效的数组索引则返回 None
// 使用 u64 来支持超大索引（JavaScript 数组最大索引是 2^32-1）
fn string_to_index(s: &str) -> Option<u64> {
  // 尝试解析为数字
  if let Ok(num) = s.parse::<u64>() {
    // 检查是否是有效的数组索引（字符串形式与数字形式一致）
    // JavaScript 数组索引最大值是 2^32 - 1
    if num <= 4294967295 && num.to_string() == s {
      return Some(num);
    }
  }
  None
}

// 获取数组对象中实际存在的所有数值索引（按数值排序）
// 用于优化稀疏数组的遍历
fn get_array_indices(obj: &Object) -> Vec<u64> {
  let mut indices: Vec<u64> = Vec::new();
  for key in obj.property_list.iter() {
    if let Some(index) = string_to_index(key) {
      indices.push(index);
    }
  }
  indices.sort();
  indices
}

// 获取数组对象中在指定范围内的实际存在的数值索引（按数值排序）
// 使用 u64 来避免溢出问题
fn get_array_indices_in_range(obj: &Object, start: u64, end: u64) -> Vec<u64> {
  let mut indices: Vec<u64> = Vec::new();
  for key in obj.property_list.iter() {
    if let Some(index) = string_to_index(key) {
      if index >= start && index < end {
        indices.push(index);
      }
    }
  }
  indices.sort();
  indices
}

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
  // 添加构造函数方法，支持 new Array()
  let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, array_create);
  arr.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  let name = String::from("isArray");
  arr.property.insert(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 1f64, array_static_is_array) });

  if let Some(prop)= &arr.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    prototype.define_builtin_function_property(ctx, String::from("concat"),  0, array_concat);
    prototype.define_builtin_function_property(ctx, String::from("join"),  1, array_join);
    prototype.define_builtin_function_property(ctx, String::from("push"),  1, array_push);
    prototype.define_builtin_function_property(ctx, String::from("pop"),  0, array_pop);
    prototype.define_builtin_function_property(ctx, String::from("toString"),  0, array_to_string);
    prototype.define_builtin_function_property(ctx, String::from("map"),  1, array_map);
    prototype.define_builtin_function_property(ctx, String::from("forEach"),  1, array_for_each);
    prototype.define_builtin_function_property(ctx, String::from("filter"),  1, array_filter);
    prototype.define_builtin_function_property(ctx, String::from("includes"),  1, array_includes);
    prototype.define_builtin_function_property(ctx, String::from("indexOf"),  1, array_index_of);
    prototype.define_builtin_function_property(ctx, String::from("fill"),  1, array_fill);
    prototype.define_builtin_function_property(ctx, String::from("find"),  1, array_find);
    prototype.define_builtin_function_property(ctx, String::from("findIndex"),  1, array_find_index);
    prototype.define_builtin_function_property(ctx, String::from("reverse"),  0, array_reverse);
    prototype.define_builtin_function_property(ctx, String::from("shift"),  0, array_shift);
    prototype.define_builtin_function_property(ctx, String::from("unshift"),  1, array_unshift);
    prototype.define_builtin_function_property(ctx, String::from("sort"),  1, array_sort);
    prototype.define_builtin_function_property(ctx, String::from("slice"),  2, array_slice);
    prototype.define_builtin_function_property(ctx, String::from("splice"),  2, array_splice);
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

// Array constructor: new Array(item1, item2, ...) or new Array(length)
// 当只有一个数字参数时，创建指定长度的空数组
// 当有多个参数或单个非数字参数时，创建包含这些元素的数组
fn array_create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  if args.len() == 0 {
    // new Array() -> 空数组
    return Ok(create_array(call_ctx.ctx, 0));
  }

  if args.len() == 1 {
    // 检查是否是数字参数（用于设置长度）
    let first_arg = &args[0];
    if let Value::Number(num) = first_arg {
      let num = *num;
      // 检查是否是有效的数组长度（正整数或0）
      let len = num as i32;
      if len >= 0 && num == (len as f64) {
        // new Array(5) -> 创建长度为5的空数组
        return Ok(create_array(call_ctx.ctx, len as usize));
      }
      // 如果不是整数或负数，抛出 RangeError
      if num < 0f64 || num.is_nan() {
        return Err(JSIError::new(JSIErrorType::RangeError, String::from("Invalid array length"), 0, 0));
      }
    }
    // 单个非数字参数 -> 创建包含该元素的数组
    let new_array = create_array(call_ctx.ctx, 1);
    if let Value::Array(arr) = &new_array {
      let mut arr_mut = arr.borrow_mut();
      arr_mut.define_property(String::from("0"), Property { enumerable: true, value: first_arg.clone() });
    }
    return Ok(new_array);
  }

  // 多个参数 -> 创建包含所有元素的数组
  let new_array = create_array(call_ctx.ctx, args.len());
  if let Value::Array(arr) = &new_array {
    let mut arr_mut = arr.borrow_mut();
    for (index, value) in args.iter().enumerate() {
      arr_mut.define_property(index.to_string(), Property { enumerable: true, value: value.clone() });
    }
  }
  Ok(new_array)
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
  let arr = arr_rc.borrow();
  let len = arr.get_property_value(String::from("length"));
  if let Value::Number(len) = len {
    // Use u64 for length to handle large indices properly
    let len_u64 = len as u64;
    // For small arrays, use direct iteration for better performance
    if len_u64 <= 1000 {
      for index in 0..(len as i32) {
        (callback)(index, &arr.get_property_value(index.to_string()), call_ctx.ctx);
      }
    } else {
      // For large/sparse arrays, only iterate over actual existing indices
      let indices = get_array_indices(&arr);
      for index in indices.iter() {
        // Only call callback for indices within the valid length
        if *index < len_u64 {
          (callback)(*index as i32, &arr.get_property_value(index.to_string()), call_ctx.ctx);
        }
      }
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
    // Use actual indices for sparse arrays
    let len_u64 = length as u64;
    if len_u64 <= 1000 {
      for index in 0..length {
        let value = this.get_property_value(index.to_string());
        new_arr_borrowed.define_property(index.to_string(), Property { enumerable: true, value: value.clone() });
      }
    } else {
      let indices = get_array_indices(&this);
      for index in indices.iter() {
        if *index < len_u64 {
          let value = this.get_property_value(index.to_string());
          new_arr_borrowed.define_property(index.to_string(), Property { enumerable: true, value: value.clone() });
        }
      }
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
        let len_u64 = len as u64;
        if len_u64 <= 1000 {
          for index in 0..(len as i32) {
            let value = obj_borrow.get_property_value(index.to_string());
            list.push(value);
          }
        } else {
          let indices = get_array_indices(&obj_borrow);
          for index in indices.iter() {
            if *index < len_u64 {
              let value = obj_borrow.get_property_value(index.to_string());
              list.push(value);
            }
          }
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

// Array.prototype.map
fn array_map(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let callback = if args.len() > 0 {
    &args[0]
  } else {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.map requires a callback function"), 0, 0))
  };

  let callback_func = match callback {
    Value::Function(func) => Rc::clone(func),
    _ => return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.map callback must be a function"), 0, 0))
  };

  let this_array_obj = get_array_object_from_this(&call_ctx.this);
  let len = if let Some(this_ref) = this_array_obj {
    let this = this_ref.borrow();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      len as u64
    } else {
      return Ok(create_array(call_ctx.ctx, 0));
    }
  } else {
    return Ok(create_array(call_ctx.ctx, 0));
  };

  let this_value = call_ctx.this.clone();
  let new_array = create_array(call_ctx.ctx, 0);
  let new_array_clone = match &new_array {
    Value::Array(arr) => Rc::clone(arr),
    _ => return Err(JSIError::new(JSIErrorType::TypeError, format!("Failed to create array"), 0, 0))
  };
  let mut new_arr_borrowed = new_array_clone.borrow_mut();

  // Get the array reference for iteration
  let this_ref = get_array_object_from_this(&call_ctx.this).unwrap();

  // For sparse arrays, only iterate over actual existing indices
  if len <= 1000 {
    for index in 0..len {
      // Get element directly during iteration instead of pre-copying
      let this_borrowed = this_ref.borrow();
      let element = this_borrowed.get_property_value(index.to_string());
      let callback_args: Vec<crate::value::ValueInfo> = vec![
        element.to_value_info(),
        Value::Number(index as f64).to_value_info(),
        this_value.to_value_info()
      ];
      // Need to release the borrow before calling the function
      drop(this_borrowed);
      call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
      let result = call_ctx.ctx.pop_stack_value();
      new_arr_borrowed.define_property(index.to_string(), Property { enumerable: true, value: result });
    }
  } else {
    let indices = {
      let this_borrowed = this_ref.borrow();
      get_array_indices(&this_borrowed)
    };
    for index in indices.iter() {
      if *index < len {
        let this_borrowed = this_ref.borrow();
        let element = this_borrowed.get_property_value(index.to_string());
        let callback_args: Vec<crate::value::ValueInfo> = vec![
          element.to_value_info(),
          Value::Number(*index as f64).to_value_info(),
          this_value.to_value_info()
        ];
        drop(this_borrowed);
        call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
        let result = call_ctx.ctx.pop_stack_value();
        new_arr_borrowed.define_property(index.to_string(), Property { enumerable: true, value: result });
      }
    }
  }
  new_arr_borrowed.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(len as f64) });
  Ok(new_array)
}

// Array.prototype.forEach
fn array_for_each(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let callback = if args.len() > 0 {
    &args[0]
  } else {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.forEach requires a callback function"), 0, 0))
  };

  let callback_func = match callback {
    Value::Function(func) => Rc::clone(func),
    _ => return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.forEach callback must be a function"), 0, 0))
  };

  let this_array_obj = get_array_object_from_this(&call_ctx.this);
  let len = if let Some(this_ref) = this_array_obj {
    let this = this_ref.borrow();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      len as u64
    } else {
      return Ok(Value::Undefined);
    }
  } else {
    return Ok(Value::Undefined);
  };

  let this_value = call_ctx.this.clone();
  let this_ref = get_array_object_from_this(&call_ctx.this).unwrap();

  // For sparse arrays, only iterate over actual existing indices
  if len <= 1000 {
    for index in 0..len {
      let this_borrowed = this_ref.borrow();
      let element = this_borrowed.get_property_value(index.to_string());
      let callback_args: Vec<crate::value::ValueInfo> = vec![
        element.to_value_info(),
        Value::Number(index as f64).to_value_info(),
        this_value.to_value_info()
      ];
      drop(this_borrowed);
      call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
      call_ctx.ctx.pop_stack_value();
    }
  } else {
    let indices = {
      let this_borrowed = this_ref.borrow();
      get_array_indices(&this_borrowed)
    };
    for index in indices.iter() {
      if *index < len {
        let this_borrowed = this_ref.borrow();
        let element = this_borrowed.get_property_value(index.to_string());
        let callback_args: Vec<crate::value::ValueInfo> = vec![
          element.to_value_info(),
          Value::Number(*index as f64).to_value_info(),
          this_value.to_value_info()
        ];
        drop(this_borrowed);
        call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
        call_ctx.ctx.pop_stack_value();
      }
    }
  }
  Ok(Value::Undefined)
}

// Array.prototype.filter
fn array_filter(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let callback = if args.len() > 0 {
    &args[0]
  } else {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.filter requires a callback function"), 0, 0))
  };

  let callback_func = match callback {
    Value::Function(func) => Rc::clone(func),
    _ => return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.filter callback must be a function"), 0, 0))
  };

  let this_array_obj = get_array_object_from_this(&call_ctx.this);
  let len = if let Some(this_ref) = this_array_obj {
    let this = this_ref.borrow();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      len as u64
    } else {
      return Ok(create_array(call_ctx.ctx, 0));
    }
  } else {
    return Ok(create_array(call_ctx.ctx, 0));
  };

  let this_value = call_ctx.this.clone();
  let new_array = create_array(call_ctx.ctx, 0);
  let new_array_clone = match &new_array {
    Value::Array(arr) => Rc::clone(arr),
    _ => return Err(JSIError::new(JSIErrorType::TypeError, format!("Failed to create array"), 0, 0))
  };
  let mut new_arr_borrowed = new_array_clone.borrow_mut();
  let mut new_index: u64 = 0;

  let this_ref = get_array_object_from_this(&call_ctx.this).unwrap();

  // For sparse arrays, only iterate over actual existing indices
  if len <= 1000 {
    for index in 0..len {
      let this_borrowed = this_ref.borrow();
      let element = this_borrowed.get_property_value(index.to_string());
      let callback_args: Vec<crate::value::ValueInfo> = vec![
        element.to_value_info(),
        Value::Number(index as f64).to_value_info(),
        this_value.to_value_info()
      ];
      let element_clone = element.clone();
      drop(this_borrowed);
      call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
      let result = call_ctx.ctx.pop_stack_value();
      if result.to_boolean(call_ctx.ctx) {
        new_arr_borrowed.define_property(new_index.to_string(), Property { enumerable: true, value: element_clone });
        new_index += 1;
      }
    }
  } else {
    let indices = {
      let this_borrowed = this_ref.borrow();
      get_array_indices(&this_borrowed)
    };
    for index in indices.iter() {
      if *index < len {
        let this_borrowed = this_ref.borrow();
        let element = this_borrowed.get_property_value(index.to_string());
        let callback_args: Vec<crate::value::ValueInfo> = vec![
          element.to_value_info(),
          Value::Number(*index as f64).to_value_info(),
          this_value.to_value_info()
        ];
        let element_clone = element.clone();
        drop(this_borrowed);
        call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
        let result = call_ctx.ctx.pop_stack_value();
        if result.to_boolean(call_ctx.ctx) {
          new_arr_borrowed.define_property(new_index.to_string(), Property { enumerable: true, value: element_clone });
          new_index += 1;
        }
      }
    }
  }
  new_arr_borrowed.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(new_index as f64) });
  Ok(new_array)
}

// Array.prototype.includes
fn array_includes(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let search_element = if args.len() > 0 {
    &args[0]
  } else {
    return Ok(Value::Boolean(false));
  };

  let mut from_index: u64 = 0;
  if args.len() > 1 {
    if let Some(pos) = args[1].to_number(call_ctx.ctx) {
      let pos_i64 = pos as i64;
      if pos_i64 < 0 {
        // Handle negative from_index later when we have len
        from_index = 0; // Will be adjusted below
      } else {
        from_index = pos_i64 as u64;
      }
    }
  }

  let this_array_obj = get_array_object_from_this(&call_ctx.this);
  if let Some(this_ref) = this_array_obj {
    let this = this_ref.borrow();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      let len_u64 = len as u64;

      // 处理负数的 from_index
      if args.len() > 1 {
        if let Some(pos) = args[1].to_number(call_ctx.ctx) {
          let pos_i64 = pos as i64;
          if pos_i64 < 0 {
            let neg_start = len_u64 as i64 + pos_i64;
            from_index = if neg_start < 0 { 0 } else { neg_start as u64 };
          }
        }
      }

      // For sparse arrays, only check actual existing indices
      if len_u64 <= 1000 {
        for index in from_index..len_u64 {
          let element = this.get_property_value(index.to_string());
          if element.is_equal_to(call_ctx.ctx, search_element, true) {
            return Ok(Value::Boolean(true));
          }
          if element.is_nan() && search_element.is_nan() {
            return Ok(Value::Boolean(true));
          }
        }
      } else {
        let indices = get_array_indices_in_range(&this, from_index, len_u64);
        for index in indices.iter() {
          let element = this.get_property_value(index.to_string());
          if element.is_equal_to(call_ctx.ctx, search_element, true) {
            return Ok(Value::Boolean(true));
          }
          if element.is_nan() && search_element.is_nan() {
            return Ok(Value::Boolean(true));
          }
        }
      }
    }
  }

  Ok(Value::Boolean(false))
}

// Array.prototype.indexOf
fn array_index_of(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let search_element = if args.len() > 0 {
    &args[0]
  } else {
    return Ok(Value::Number(-1f64));
  };

  let mut from_index: u64 = 0;
  if args.len() > 1 {
    if let Some(pos) = args[1].to_number(call_ctx.ctx) {
      let pos_i64 = pos as i64;
      if pos_i64 < 0 {
        from_index = 0; // Will be adjusted below
      } else {
        from_index = pos_i64 as u64;
      }
    }
  }

  let this_array_obj = get_array_object_from_this(&call_ctx.this);
  if let Some(this_ref) = this_array_obj {
    let this = this_ref.borrow();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      let len_u64 = len as u64;

      // 处理负数的 from_index
      if args.len() > 1 {
        if let Some(pos) = args[1].to_number(call_ctx.ctx) {
          let pos_i64 = pos as i64;
          if pos_i64 < 0 {
            let neg_start = len_u64 as i64 + pos_i64;
            from_index = if neg_start < 0 { 0 } else { neg_start as u64 };
          }
        }
      }

      // For sparse arrays, only check actual existing indices
      if len_u64 <= 1000 {
        for index in from_index..len_u64 {
          let element = this.get_property_value(index.to_string());
          if element.is_equal_to(call_ctx.ctx, search_element, true) {
            return Ok(Value::Number(index as f64));
          }
        }
      } else {
        let indices = get_array_indices_in_range(&this, from_index, len_u64);
        for index in indices.iter() {
          let element = this.get_property_value(index.to_string());
          if element.is_equal_to(call_ctx.ctx, search_element, true) {
            return Ok(Value::Number(*index as f64));
          }
        }
      }
    }
  }

  Ok(Value::Number(-1f64))
}

// Array.prototype.fill
// arr.fill(value[, start[, end]])
fn array_fill(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let fill_value = if args.len() > 0 {
    args[0].clone()
  } else {
    Value::Undefined
  };

  let this_array_obj = get_array_object_from_this(&call_ctx.this);
  if let Some(this_ref) = this_array_obj {
    let len = {
      let this = this_ref.borrow();
      let len = this.get_property_value(String::from("length"));
      if let Value::Number(len) = len {
        len as i32
      } else {
        return Ok(call_ctx.this.clone());
      }
    };

    // Parse start and end parameters
    let mut start: i32 = 0;
    let mut end: i32 = len;

    if args.len() > 1 {
      if let Some(s) = args[1].to_number(call_ctx.ctx) {
        start = s as i32;
        // Handle negative start
        if start < 0 {
          start = len + start;
          if start < 0 {
            start = 0;
          }
        }
        if start > len {
          start = len;
        }
      }
    }

    if args.len() > 2 {
      if let Some(e) = args[2].to_number(call_ctx.ctx) {
        end = e as i32;
        // Handle negative end
        if end < 0 {
          end = len + end;
          if end < 0 {
            end = 0;
          }
        }
        if end > len {
          end = len;
        }
      }
    }

    // Fill the array
    let mut this = this_ref.borrow_mut();
    for index in start..end {
      this.define_property(index_to_string(index), Property { enumerable: true, value: fill_value.clone() });
    }
  }

  Ok(call_ctx.this.clone())
}

// Array.prototype.find
// arr.find(callback[, thisArg])
fn array_find(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let callback = if args.len() > 0 {
    &args[0]
  } else {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.find requires a callback function"), 0, 0))
  };

  let callback_func = match callback {
    Value::Function(func) => Rc::clone(func),
    _ => return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.find callback must be a function"), 0, 0))
  };

  let this_array_obj = get_array_object_from_this(&call_ctx.this);
  let len = if let Some(this_ref) = this_array_obj {
    let this = this_ref.borrow();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      len as u64
    } else {
      return Ok(Value::Undefined);
    }
  } else {
    return Ok(Value::Undefined);
  };

  let this_value = call_ctx.this.clone();
  let this_ref = get_array_object_from_this(&call_ctx.this).unwrap();

  // For sparse arrays, only iterate over actual existing indices
  if len <= 1000 {
    for index in 0..len {
      let this_borrowed = this_ref.borrow();
      let element = this_borrowed.get_property_value(index.to_string());
      let callback_args: Vec<crate::value::ValueInfo> = vec![
        element.to_value_info(),
        Value::Number(index as f64).to_value_info(),
        this_value.to_value_info()
      ];
      let element_clone = element.clone();
      drop(this_borrowed);
      call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
      let result = call_ctx.ctx.pop_stack_value();
      if result.to_boolean(call_ctx.ctx) {
        return Ok(element_clone);
      }
    }
  } else {
    let indices = {
      let this_borrowed = this_ref.borrow();
      get_array_indices(&this_borrowed)
    };
    for index in indices.iter() {
      if *index < len {
        let this_borrowed = this_ref.borrow();
        let element = this_borrowed.get_property_value(index.to_string());
        let callback_args: Vec<crate::value::ValueInfo> = vec![
          element.to_value_info(),
          Value::Number(*index as f64).to_value_info(),
          this_value.to_value_info()
        ];
        let element_clone = element.clone();
        drop(this_borrowed);
        call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
        let result = call_ctx.ctx.pop_stack_value();
        if result.to_boolean(call_ctx.ctx) {
          return Ok(element_clone);
        }
      }
    }
  }

  Ok(Value::Undefined)
}

// Array.prototype.findIndex
// arr.findIndex(callback[, thisArg])
fn array_find_index(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let callback = if args.len() > 0 {
    &args[0]
  } else {
    return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.findIndex requires a callback function"), 0, 0))
  };

  let callback_func = match callback {
    Value::Function(func) => Rc::clone(func),
    _ => return Err(JSIError::new(JSIErrorType::TypeError, format!("Array.prototype.findIndex callback must be a function"), 0, 0))
  };

  let this_array_obj = get_array_object_from_this(&call_ctx.this);
  let len = if let Some(this_ref) = this_array_obj {
    let this = this_ref.borrow();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      len as u64
    } else {
      return Ok(Value::Number(-1f64));
    }
  } else {
    return Ok(Value::Number(-1f64));
  };

  let this_value = call_ctx.this.clone();
  let this_ref = get_array_object_from_this(&call_ctx.this).unwrap();

  // For sparse arrays, only iterate over actual existing indices
  if len <= 1000 {
    for index in 0..len {
      let this_borrowed = this_ref.borrow();
      let element = this_borrowed.get_property_value(index.to_string());
      let callback_args: Vec<crate::value::ValueInfo> = vec![
        element.to_value_info(),
        Value::Number(index as f64).to_value_info(),
        this_value.to_value_info()
      ];
      drop(this_borrowed);
      call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
      let result = call_ctx.ctx.pop_stack_value();
      if result.to_boolean(call_ctx.ctx) {
        return Ok(Value::Number(index as f64));
      }
    }
  } else {
    let indices = {
      let this_borrowed = this_ref.borrow();
      get_array_indices(&this_borrowed)
    };
    for index in indices.iter() {
      if *index < len {
        let this_borrowed = this_ref.borrow();
        let element = this_borrowed.get_property_value(index.to_string());
        let callback_args: Vec<crate::value::ValueInfo> = vec![
          element.to_value_info(),
          Value::Number(*index as f64).to_value_info(),
          this_value.to_value_info()
        ];
        drop(this_borrowed);
        call_ctx.ctx.call_function_with_bytecode(callback_func.clone(), None, None, callback_args)?;
        let result = call_ctx.ctx.pop_stack_value();
        if result.to_boolean(call_ctx.ctx) {
          return Ok(Value::Number(*index as f64));
        }
      }
    }
  }

  Ok(Value::Number(-1f64))
}

// Array.prototype.pop
// arr.pop()
fn array_pop(call_ctx: &mut CallContext, _args: Vec<Value>) -> JSIResult<Value> {
  let this_array_obj = get_array_object_from_this(&call_ctx.this);

  if let Some(this_ref) = this_array_obj {
    let mut this = this_ref.borrow_mut();
    let len_opt = this.get_property_value(String::from("length")).to_number(call_ctx.ctx);
    if let Some(len) = len_opt {
      let len = len as i32;
      if len == 0 {
        this.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(0f64) });
        return Ok(Value::Undefined);
      }
      let last_index = len - 1;
      let last_value = this.get_property_value(index_to_string(last_index));
      this.property.remove(&index_to_string(last_index));
      this.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(last_index as f64) });
      return Ok(last_value);
    }
    return Ok(Value::Undefined);
  }
  Ok(Value::Undefined)
}

// Array.prototype.reverse
// arr.reverse()
fn array_reverse(call_ctx: &mut CallContext, _args: Vec<Value>) -> JSIResult<Value> {
  let this_array_obj = get_array_object_from_this(&call_ctx.this);

  if let Some(this_ref) = this_array_obj {
    let mut this = this_ref.borrow_mut();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      let len_u64 = len as u64;

      // For sparse arrays, only process actual existing indices
      if len_u64 <= 1000 {
        // Collect all elements first
        let mut elements: Vec<(u64, Value)> = vec![];
        for index in 0..len_u64 {
          let value = this.get_property_value(index.to_string());
          elements.push((index, value));
        }

        // Reverse and write back
        for (new_index, (_, value)) in elements.into_iter().enumerate() {
          let old_index = len_u64 - 1 - new_index as u64;
          this.define_property(old_index.to_string(), Property { enumerable: true, value: value.clone() });
        }
      } else {
        // For sparse arrays, get actual indices and swap pairs
        let indices = get_array_indices(&this);
        // Swap each pair of indices
        let mid = indices.len() / 2;
        for i in 0..mid {
          let left_idx = indices[i];
          let right_idx = indices[indices.len() - 1 - i];
          let left_value = this.get_property_value(left_idx.to_string());
          let right_value = this.get_property_value(right_idx.to_string());
          // Swap: left index gets right value at mirrored position
          let new_left = len_u64 - 1 - right_idx;
          let new_right = len_u64 - 1 - left_idx;
          this.property.remove(&left_idx.to_string());
          this.property.remove(&right_idx.to_string());
          this.define_property(new_left.to_string(), Property { enumerable: true, value: right_value });
          this.define_property(new_right.to_string(), Property { enumerable: true, value: left_value });
        }
        // Handle odd number of elements
        if indices.len() % 2 == 1 {
          let mid_idx = indices[mid];
          let new_mid = len_u64 - 1 - mid_idx;
          let value = this.get_property_value(mid_idx.to_string());
          this.property.remove(&mid_idx.to_string());
          this.define_property(new_mid.to_string(), Property { enumerable: true, value });
        }
      }
    }
  }

  Ok(call_ctx.this.clone())
}

// Array.prototype.shift
// arr.shift()
fn array_shift(call_ctx: &mut CallContext, _args: Vec<Value>) -> JSIResult<Value> {
  let this_array_obj = get_array_object_from_this(&call_ctx.this);

  if let Some(this_ref) = this_array_obj {
    let mut this = this_ref.borrow_mut();
    let len_opt = this.get_property_value(String::from("length")).to_number(call_ctx.ctx);
    if let Some(len) = len_opt {
      let len_u64 = len as u64;
      if len_u64 == 0 {
        this.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(0f64) });
        return Ok(Value::Undefined);
      }

      // Get first element
      let first_value = this.get_property_value(String::from("0"));

      // For sparse arrays, only shift actual existing indices
      if len_u64 <= 1000 {
        // Shift all elements
        for index in 1..len_u64 {
          let value = this.get_property_value(index.to_string());
          this.define_property((index - 1).to_string(), Property { enumerable: true, value });
        }

        // Remove last element
        this.property.remove(&(len_u64 - 1).to_string());
      } else {
        // Get actual indices and shift them
        let indices = get_array_indices(&this);
        // Remove index 0 if it exists
        this.property.remove(&String::from("0"));
        // Shift all other indices left by 1
        for index in indices.iter() {
          if *index > 0 && *index < len_u64 {
            let value = this.get_property_value(index.to_string());
            this.property.remove(&index.to_string());
            this.define_property((*index - 1).to_string(), Property { enumerable: true, value });
          }
        }
      }

      this.define_property(String::from("length"), Property { enumerable: false, value: Value::Number((len_u64 - 1) as f64) });

      return Ok(first_value);
    }
    return Ok(Value::Undefined);
  }
  Ok(Value::Undefined)
}

// Array.prototype.unshift
// arr.unshift(element1[, ...[, elementN]])
fn array_unshift(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let this_array_obj = get_array_object_from_this(&call_ctx.this);

  if let Some(this_ref) = this_array_obj {
    let mut this = this_ref.borrow_mut();
    let len_opt = this.get_property_value(String::from("length")).to_number(call_ctx.ctx);
    if let Some(len) = len_opt {
      let len_u64 = len as u64;
      let add_count = args.len() as u64;
      let new_len = len_u64 + add_count;

      // For sparse arrays, only shift actual existing indices
      if len_u64 <= 1000 {
        // Shift existing elements to the right (from end to start)
        for index in (0..len_u64).rev() {
          let value = this.get_property_value(index.to_string());
          this.define_property((index + add_count).to_string(), Property { enumerable: true, value });
        }

        // Clear old positions that were shifted
        for index in 0..add_count.min(len_u64) {
          this.property.remove(&index.to_string());
        }
      } else {
        // Get actual indices and shift them right
        let indices = get_array_indices(&this);
        // Process in reverse order
        for index in indices.iter().rev() {
          if *index < len_u64 {
            let value = this.get_property_value(index.to_string());
            this.property.remove(&index.to_string());
            this.define_property((*index + add_count).to_string(), Property { enumerable: true, value });
          }
        }
      }

      // Insert new elements at the beginning
      for (i, value) in args.iter().enumerate() {
        this.define_property(i.to_string(), Property { enumerable: true, value: value.clone() });
      }

      this.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(new_len as f64) });
      return Ok(Value::Number(new_len as f64));
    }
    return Ok(Value::Number(args.len() as f64));
  }
  Ok(Value::Number(args.len() as f64))
}

// Array.prototype.sort
// arr.sort([compareFunction])
fn array_sort(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let this_array_obj = get_array_object_from_this(&call_ctx.this);

  if let Some(this_ref) = this_array_obj {
    let mut this = this_ref.borrow_mut();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      let len_u64 = len as u64;

      // For sparse arrays, only collect actual existing elements
      if len_u64 <= 1000 {
        // Collect elements into a vector
        let mut elements: Vec<Value> = Vec::with_capacity(len_u64 as usize);
        for index in 0..len_u64 {
          elements.push(this.get_property_value(index.to_string()));
        }

        // Sort the elements (same logic as before)
        if args.len() > 0 {
          let compare_func = &args[0];
          if let Value::Function(_func) = compare_func {
            elements.sort_by(|a, b| {
              let a_str = match a {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                _ => String::from("[object]"),
              };
              let b_str = match b {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                _ => String::from("[object]"),
              };
              a_str.cmp(&b_str)
            });
          } else {
            elements.sort_by(|a, b| {
              let a_str = match a {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                _ => String::from("[object]"),
              };
              let b_str = match b {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                _ => String::from("[object]"),
              };
              a_str.cmp(&b_str)
            });
          }
        } else {
          elements.sort_by(|a, b| {
            let a_str = match a {
              Value::Number(n) => n.to_string(),
              Value::String(s) => s.clone(),
              _ => String::from("[object]"),
            };
            let b_str = match b {
              Value::Number(n) => n.to_string(),
              Value::String(s) => s.clone(),
              _ => String::from("[object]"),
            };
            a_str.cmp(&b_str)
          });
        }

        // Write sorted elements back
        for (index, value) in elements.into_iter().enumerate() {
          this.define_property(index.to_string(), Property { enumerable: true, value });
        }
      } else {
        // For sparse arrays, collect only actual existing elements with their indices
        let indices = get_array_indices(&this);
        let mut elements_with_indices: Vec<(u64, Value)> = Vec::new();
        for index in indices.iter() {
          if *index < len_u64 {
            let value = this.get_property_value(index.to_string());
            elements_with_indices.push((*index, value));
          }
        }

        // Sort the elements (same logic as before)
        elements_with_indices.sort_by(|(_, a), (_, b)| {
          let a_str = match a {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            _ => String::from("[object]"),
          };
          let b_str = match b {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            _ => String::from("[object]"),
          };
          a_str.cmp(&b_str)
        });

        // Remove old properties
        for index in indices.iter() {
          this.property.remove(&index.to_string());
        }

        // Write sorted elements back at indices 0..n
        for (new_index, (_, value)) in elements_with_indices.into_iter().enumerate() {
          this.define_property(new_index.to_string(), Property { enumerable: true, value });
        }
      }
    }
  }

  Ok(call_ctx.this.clone())
}

// Array.prototype.slice
// arr.slice([begin[, end]])
fn array_slice(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let this_array_obj = get_array_object_from_this(&call_ctx.this);

  let len = if let Some(this_ref) = this_array_obj {
    let this = this_ref.borrow();
    let len = this.get_property_value(String::from("length"));
    if let Value::Number(len) = len {
      len as u64
    } else {
      return Ok(create_array(call_ctx.ctx, 0));
    }
  } else {
    return Ok(create_array(call_ctx.ctx, 0));
  };

  // Parse begin
  let mut begin: u64 = 0;
  if args.len() > 0 {
    if let Some(b) = args[0].to_number(call_ctx.ctx) {
      let b_i64 = b as i64;
      if b_i64 < 0 {
        let neg_start = len as i64 + b_i64;
        begin = if neg_start < 0 { 0 } else { neg_start as u64 };
      } else {
        begin = if b_i64 > len as i64 { len } else { b_i64 as u64 };
      }
    }
  }

  // Parse end
  let mut end: u64 = len;
  if args.len() > 1 {
    if let Some(e) = args[1].to_number(call_ctx.ctx) {
      let e_i64 = e as i64;
      if e_i64 < 0 {
        let neg_end = len as i64 + e_i64;
        end = if neg_end < 0 { 0 } else { neg_end as u64 };
      } else {
        end = if e_i64 > len as i64 { len } else { e_i64 as u64 };
      }
    }
  }

  // Create new array
  let new_array = create_array(call_ctx.ctx, 0);
  if let Value::Array(arr) = &new_array {
    let mut arr_mut = arr.borrow_mut();
    let this_ref = get_array_object_from_this(&call_ctx.this).unwrap();
    let this_borrowed = this_ref.borrow();

    let mut new_index: u64 = 0;
    // For sparse arrays, only iterate over actual existing indices in range
    if (end - begin) <= 1000 {
      for index in begin..end {
        if index < len {
          let value = this_borrowed.get_property_value(index.to_string());
          arr_mut.define_property(new_index.to_string(), Property { enumerable: true, value });
          new_index += 1;
        }
      }
    } else {
      let indices = get_array_indices_in_range(&this_borrowed, begin, end);
      for index in indices.iter() {
        let value = this_borrowed.get_property_value(index.to_string());
        arr_mut.define_property(new_index.to_string(), Property { enumerable: true, value });
        new_index += 1;
      }
    }
    arr_mut.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(new_index as f64) });
  }

  Ok(new_array)
}

// Array.prototype.splice
// array.splice(start[, deleteCount[, item1[, item2[, ...]]]])
fn array_splice(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let this_array_obj = get_array_object_from_this(&call_ctx.this);

  if let Some(this_ref) = this_array_obj {
    let len = {
      let this = this_ref.borrow();
      let len = this.get_property_value(String::from("length"));
      if let Value::Number(len) = len {
        len as u64
      } else {
        return Ok(create_array(call_ctx.ctx, 0));
      }
    };

    // Parse start
    let mut start: u64 = 0;
    if args.len() > 0 {
      if let Some(s) = args[0].to_number(call_ctx.ctx) {
        let s = s as i64;
        if s < 0 {
          let neg_start = len as i64 + s;
          start = if neg_start < 0 { 0 } else { neg_start as u64 };
        } else {
          start = if s > len as i64 { len } else { s as u64 };
        }
      }
    }

    // Parse deleteCount
    let mut delete_count: u64 = len - start;
    if args.len() > 1 {
      if let Some(dc) = args[1].to_number(call_ctx.ctx) {
        let dc = dc as i64;
        if dc < 0 {
          delete_count = 0;
        } else {
          delete_count = if dc > (len - start) as i64 { len - start } else { dc as u64 };
        }
      }
    }

    // Get items to insert
    let items: Vec<Value> = if args.len() > 2 {
      args[2..].to_vec()
    } else {
      vec![]
    };

    // Collect deleted elements - only collect actual existing indices
    let deleted_array = create_array(call_ctx.ctx, 0);
    let indices_to_delete: Vec<u64>;
    {
      let this_borrowed = this_ref.borrow();
      // Calculate end of deletion range, handling potential overflow
      let delete_end = if start.checked_add(delete_count).is_some() {
        start + delete_count
      } else {
        len // Cap at array length
      };
      indices_to_delete = get_array_indices_in_range(&this_borrowed, start, delete_end);

      if let Value::Array(deleted_arr) = &deleted_array {
        let mut deleted_mut = deleted_arr.borrow_mut();
        for (i, index) in indices_to_delete.iter().enumerate() {
          let value = this_borrowed.get_property_value(index.to_string());
          deleted_mut.define_property(i.to_string(), Property { enumerable: true, value });
        }
        deleted_mut.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(indices_to_delete.len() as f64) });
      }
    }

    // Perform splice
    let mut this = this_ref.borrow_mut();

    // Calculate new length
    let items_len = items.len() as u64;
    let new_len = len - delete_count + items_len;

    // Remove deleted properties
    for index in indices_to_delete.iter() {
      this.property.remove(&index.to_string());
      this.property_list.retain(|key| key != &index.to_string());
    }

    // Get all indices that need to be shifted (those >= start + delete_count)
    let shift_indices = {
      let shift_start = start + delete_count;
      get_array_indices_in_range(&this, shift_start, len)
    };

    // Shift elements that are after the deleted range
    // Move from old position to new position
    let delta = items_len as i64 - delete_count as i64;
    if delta != 0 {
      if delta < 0 {
        // Shifting left (deleting more than inserting)
        for old_index in shift_indices.iter() {
          let new_index = *old_index as i64 + delta;
          if new_index >= 0 {
            let value = this.get_property_value(old_index.to_string());
            // Remove old position
            this.property.remove(&old_index.to_string());
            this.property_list.retain(|key| key != &old_index.to_string());
            // Set new position
            this.define_property(new_index.to_string(), Property { enumerable: true, value });
          }
        }
      } else {
        // Shifting right (inserting more than deleting)
        // Process in reverse order to avoid overwriting
        for old_index in shift_indices.iter().rev() {
          let new_index = *old_index + delta as u64;
          let value = this.get_property_value(old_index.to_string());
          // Remove old position
          this.property.remove(&old_index.to_string());
          this.property_list.retain(|key| key != &old_index.to_string());
          // Set new position
          this.define_property(new_index.to_string(), Property { enumerable: true, value });
        }
      }
    }

    // Insert new items
    for (i, item) in items.iter().enumerate() {
      this.define_property((start + i as u64).to_string(), Property { enumerable: true, value: item.clone() });
    }

    // Update length
    this.define_property(String::from("length"), Property { enumerable: false, value: Value::Number(new_len as f64) });

    return Ok(deleted_array);
  }

  Ok(create_array(call_ctx.ctx, 0))
}