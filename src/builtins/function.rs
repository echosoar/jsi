use std::rc::Rc;

use crate::{ast_node::{Statement, FunctionDeclaration}, value::Value};

use super::object::new_base_object;

 // ref: https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#prod-FunctionDeclaration
pub fn new_function(function_declaration: &FunctionDeclaration) -> Value {
  let function = new_base_object(Some(Box::new(Statement::Function((*function_declaration).clone()))));
  let function_clone = Rc::clone(&function);
  let mut function_mut = (*function_clone).borrow_mut();
  // TODO: function.prototype = global.function_prototype;
  // TODO: function name https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-setfunctionname
  function_mut.define_property_by_value(String::from("name"),  Value::String(function_declaration.name.literal.clone()));
  function_mut.define_property_by_value(String::from("length"), Value::Number(function_declaration.parameters.len() as f64));

  Value::Function(function)
}