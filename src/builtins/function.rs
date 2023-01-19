use std::{rc::Rc, cell::RefCell};

use crate::{ast_node::{Statement, FunctionDeclaration, BuiltinFunction, ClassType, CallContext}, value::{Value}};

use super::{object::{create_object, Property, Object}, global::{get_global_object}};

// 初始化一个方法
// ref: https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#prod-FunctionDeclaration
 pub fn create_function(global: &Rc<RefCell<Object>>, function_declaration: &FunctionDeclaration) -> Value {
  let global_function = get_global_object(global, String::from("Function"));
  let function = create_object(global,ClassType::Function, Some(Box::new(Statement::Function((*function_declaration).clone()))));
  let function_clone = Rc::clone(&function);
  let mut function_mut = (*function_clone).borrow_mut();
  // 绑定 fun.constructor = global.Function
  function_mut.constructor = Some(Rc::downgrade(&global_function));
  // fun.name
  function_mut.define_property(String::from("name"), Property {
    enumerable: false,
    value: Value::String(function_declaration.name.literal.clone()),
  });
  // fun.length
  function_mut.define_property(String::from("length"), Property {
    enumerable: false,
    value: Value::Number(function_declaration.parameters.len() as f64)
  });
  Value::Function(function)
}

// 构建内置方法
pub fn builtin_function(global: &Rc<RefCell<Object>>, name: String, length: f64, fun: BuiltinFunction) -> Value {
  let global_function = get_global_object(global, String::from("Function"));
  let function = create_object(global, ClassType::Function, Some(Box::new(Statement::BuiltinFunction(fun))));
  let function_clone = Rc::clone(&function);
  let mut function_mut = (*function_clone).borrow_mut();
  // 绑定 fun.constructor = global.Function
  function_mut.constructor = Some(Rc::downgrade(&global_function));
  // fun.name
  function_mut.define_property(String::from("name"), Property {
    enumerable: false,
    value: Value::String(name),
  });
  // fun.length
  function_mut.define_property(String::from("length"), Property {
    enumerable: false,
    value: Value::Number(length)
  });
  Value::Function(function)
}


pub fn bind_global_function(global: &Rc<RefCell<Object>>) {
  let fun_rc = get_global_object(global, String::from("Function"));
  let fun = (*fun_rc).borrow_mut();
  if let Some(prop)= &fun.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    // let name = String::from("toString");
    // prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 0f64, function_to_string) });
    let name = String::from("call");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 1f64, function_call) });
    let name = String::from("bind");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 1f64, function_bind) });
  }
}


// Function.prototype.call
fn function_call(ctx: &mut CallContext, args: Vec<Value>) -> Value {
  let mut this = Value::Undefined;
  if args.len() > 0 {
    this = args[0].clone();
  }
  let new_fun = function_bind(ctx, vec![this]);
  Value::Undefined
}

// Function.prototype.bind
fn function_bind(ctx: &mut CallContext, args: Vec<Value>) -> Value {
  let mut this = Value::Undefined;
  if args.len() > 0 {
    this = args[0].clone();
  }
  let reference = &ctx.reference;
  if let Some(ref_wk) = reference {
    let fun = ref_wk.upgrade().unwrap();
    let fun_obj = fun.borrow();
    let new_fun = fun_obj.force_copy();
    println!("reference: {:?}", new_fun);
  }
  Value::String("call".to_owned())
}