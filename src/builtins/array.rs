use std::{rc::Rc, cell::RefCell, borrow::BorrowMut};

use crate::{value::Value, ast_node::{Statement, BuiltinFunctionDeclaration}};

use super::object::{new_base_object, Property, Object};


 // ref: https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#prod-FunctionDeclaration
 pub fn new_array(length: i32) -> Value {
  let array = new_base_object(None);
  let array_clone = Rc::clone(&array);
  let mut array_mut = (*array_clone).borrow_mut();
  array_mut.define_property_by_value(String::from("length"),  Value::Number(f64::from(length)));

  let to_string = new_base_object(Some(Box::new(Statement::BuiltinFunction(BuiltinFunctionDeclaration {
    call: array_to_string
  }))));
  array_mut.inner_property.insert(String::from("to_string"), Property {
    enumerable: false,
    value: Value::Function(to_string)
  });

  Value::Array(array)
}

fn array_to_string(this: Option<Rc<RefCell<Object>>>,_: Vec<Value>) -> Value {
  array_join(this, vec![])
}

fn array_join(this: Option<Rc<RefCell<Object>>>, args: Vec<Value>) -> Value {
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
  array_iter_mut(this.unwrap(), iter);
  Value::String(string_list.join(join))
}

fn array_iter_mut<F: FnMut(i32, &Value)>(this: Rc<RefCell<Object>>,mut callback: F) {
  let len = this.borrow().get_property_value(String::from("length"));
  if let Value::Number(len) = len {
    for index in 0..(len as i32) {
      (callback)(index, &this.borrow().get_property_value(index.to_string()));
    }
  }
}