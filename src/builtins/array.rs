use std::{rc::Rc};

use crate::{value::Value, ast_node::{Statement, CallContext}};

use super::object::{new_base_object, Property, Object};


 // ref: https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#prod-FunctionDeclaration
 pub fn new_array(length: usize) -> Value {
  let array = new_base_object(None);
  let array_clone = Rc::clone(&array);
  let mut array_mut = (*array_clone).borrow_mut();
  array_mut.define_property_by_value(String::from("length"),  Value::Number(length as f64));

  let to_string = new_base_object(Some(Box::new(Statement::BuiltinFunction(array_to_string))));
  array_mut.inner_property.insert(String::from("to_string"), Property {
    enumerable: false,
    value: Value::Function(to_string)
  });
  array_mut.inner_property.insert(String::from("initialed"), Property {
    enumerable: false,
    value: Value::Boolean(false)
  });

  Value::Array(array)
}


 // Array 最根层的全局Array (global)
 pub fn global_array(length: i32) -> Value {
  let array = new_base_object(None);
  let array_clone = Rc::clone(&array);
  let mut array_mut = (*array_clone).borrow_mut();
  
 // new Array
 let new_array = new_base_object(Some(Box::new(Statement::BuiltinFunction(array_constructor))));
  array_mut.inner_property.insert(String::from("constructor"), Property {
    enumerable: false,
    value: Value::Function(new_array)
  });
  // Array.prototype.length
  array_mut.define_prototype_property(String::from("length"),  Property { enumerable: false, value: Value::Number(f64::from(length)) });
  // Array.prototype.push
  let push = new_base_object(Some(Box::new(Statement::BuiltinFunction(array_push))));
  array_mut.define_prototype_property(String::from("push"),  Property { enumerable: false, value: Value::Function(push) });
  Value::Array(array)
}

fn array_to_string(ctx: &mut CallContext, _: Vec<Value>) -> Value {
  array_join(ctx, vec![])
}

fn array_join(ctx: &mut CallContext, args: Vec<Value>) -> Value {
  let mut join = ",";
  if args.len() > 0 {
    if let Value::String(join_param) = &args[0] {
      join = join_param;
    }
  }
  let mut string_list: Vec<String> = vec![];
  let iter = |_: i32, value: &Value| {
    string_list.push(value.to_string());
  };
  array_iter_mut(ctx, iter);
  Value::String(string_list.join(join))
}

fn array_iter_mut<F: FnMut(i32, &Value)>(ctx: &mut CallContext, mut callback: F) {
  let this_rc = ctx.this.upgrade().unwrap();
  let this = this_rc.borrow_mut();
  let len = this.get_property_value(String::from("length"));
  if let Value::Number(len) = len {
    for index in 0..(len as i32) {
      (callback)(index, &this.get_property_value(index.to_string()));
    }
  }
}

// new Array
fn array_constructor(_: &mut CallContext, args: Vec<Value>) -> Value {
  let mut len = args.len();
  let mut need_to_init_value = true;
  if len == 1 {
    if let Value::Number(num) = args[0] {
      len = num as usize;
      need_to_init_value = false;
    }
  }
  let arr = new_array(len);
  if need_to_init_value {
    if let Value::Array(arr) = &arr {
      let ctx = &mut CallContext {
        this: Rc::downgrade(arr),
      };
      Object::call_builtin(String::from("push"), args, ctx);
    }
  };
  return arr
}

// Array.prototype.push
fn array_push(ctx: &mut CallContext, args: Vec<Value>) -> Value {
  // 插入值
  let this_rc = ctx.this.upgrade().unwrap();
  let mut this = this_rc.borrow_mut();
  let mut len = this.get_property_value(String::from("length")).to_number().unwrap() as usize;
  for value in args.iter() { 
    this.define_property(len.to_string(), Property { enumerable: true, value: value.clone() });
    len += 1
  }
  let new_length = Value::Number(len as f64);
  this.define_property_by_value(len.to_string(),  new_length.clone());

  return new_length
}