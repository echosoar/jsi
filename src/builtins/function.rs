use std::rc::Rc;

use crate::{ast_node::{Statement, FunctionDeclaration, BuiltinFunction}, value::{Value}};

use super::{object::{create_object, Property}, global::{Global, ClassType}};

// 初始化一个方法
// ref: https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#prod-FunctionDeclaration
 pub fn create_function(global: &Global, function_declaration: &FunctionDeclaration) -> Value {
  let function = create_object(global,ClassType::Function, Some(Box::new(Statement::Function((*function_declaration).clone()))));
  let function_clone = Rc::clone(&function);
  let mut function_mut = (*function_clone).borrow_mut();
  // 绑定 fun.constructor = global.Function
  function_mut.constructor = Some(Rc::clone(&global.function));
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
pub fn builtin_function(global: &Global, name: String, length: f64, fun: BuiltinFunction) -> Value {
  let function = create_object(global, ClassType::Function, Some(Box::new(Statement::BuiltinFunction(fun))));
  let function_clone = Rc::clone(&function);
  let mut function_mut = (*function_clone).borrow_mut();
  // 绑定 fun.constructor = global.Function
  function_mut.constructor = Some(Rc::clone(&global.function));
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