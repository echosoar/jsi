use std::{rc::Rc, cell::RefCell};

use crate::{value::Value, ast_node::{ClassType, CallContext}};

use super::{object::{create_object, Object, Property}, global::get_global_object, function::builtin_function};

 pub fn create_number(global: &Rc<RefCell<Object>>, init: Value) -> Value {
  let global_number = get_global_object(global, String::from("Number"));
  let number = create_object(global, ClassType::Number, None);
  let number_clone = Rc::clone(&number);
  let mut number_mut = (*number_clone).borrow_mut();
  number_mut.constructor = Some(Rc::downgrade(&global_number));
  number_mut.set_inner_property_value(String::from("value"), init);
  Value::NumberObj(number)
}

pub fn bind_global_number(global:  &Rc<RefCell<Object>>) {
  let number_rc = get_global_object(global, String::from("Number"));
  let bool = (*number_rc).borrow_mut();
  if let Some(prop)= &bool.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = (*prototype_rc).borrow_mut();
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 0f64, to_string) });
    let name = String::from("valueOf");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 1f64, value_of) });
  }
}

// Number.prototype.toString
fn to_string(ctx: &mut CallContext, _: Vec<Value>) -> Value {
  let value = value_of(ctx, vec![]);
  let global = ctx.global.upgrade().unwrap();
  Value::String(value.to_string(&global))
}


// Number.prototype.valueOf
fn value_of(ctx: &mut CallContext, _: Vec<Value>) -> Value {
  let global = ctx.global.upgrade().unwrap();
  let this_origin = ctx.this.upgrade();
  let this_rc = this_origin.unwrap();
  let init = this_rc.borrow().get_inner_property_value(String::from("value"));
  if let Some(value) = init {
    return Value::Number(value.to_number(&global).unwrap())
  }
  Value::Number(0f64)
}