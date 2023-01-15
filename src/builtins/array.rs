use std::{rc::Rc};

use crate::{value::Value, ast_node::{CallContext}};

use super::{object::{create_object, Property}, global::Global, function::builtin_function};


 // ref: https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#prod-FunctionDeclaration
 pub fn create_array(global: &Global, length: usize) -> Value {
  let array = create_object(global, None);
  let array_clone = Rc::clone(&array);
  let mut array_mut = (*array_clone).borrow_mut();
  array_mut.define_property(String::from("length"),  Property { enumerable: true, value: Value::Number(length as f64) });
  // 绑定 fun.constructor = global.Array
  array_mut.constructor = Some(Rc::clone(&global.array));
  Value::Array(array)
}

pub fn bind_global_array(global: &Global) {
  let arr = (*global.array).borrow_mut();
  if let Some(prop)= &arr.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 0f64, array_to_string) });
  }
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

// // new Array
// fn array_constructor(_: &mut CallContext, args: Vec<Value>) -> Value {
//   let mut len = args.len();
//   let mut need_to_init_value = true;
//   if len == 1 {
//     if let Value::Number(num) = args[0] {
//       len = num as usize;
//       need_to_init_value = false;
//     }
//   }
//   let arr = new_array(len);
//   if need_to_init_value {
//     if let Value::Array(arr) = &arr {
//       let ctx = &mut CallContext {
//         this: Rc::downgrade(arr),
//       };
//       Object::call_builtin(String::from("push"), args, ctx);
//     }
//   };
//   return arr
// }

// // Array.prototype.push
// fn array_push(ctx: &mut CallContext, args: Vec<Value>) -> Value {
//   // 插入值
//   let this_rc = ctx.this.upgrade().unwrap();
//   let mut this = this_rc.borrow_mut();
//   let mut len = this.get_property_value(String::from("length")).to_number().unwrap() as usize;
//   for value in args.iter() { 
//     this.define_property(len.to_string(), Property { enumerable: true, value: value.clone() });
//     len += 1
//   }
//   let new_length = Value::Number(len as f64);
//   this.define_property_by_value(len.to_string(),  new_length.clone());

//   return new_length
// }