use std::{rc::Rc, cell::RefCell};

use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, constants::GLOBAL_BOOLEAN_NAME};

use super::{object::{create_object, Object, Property}, global::get_global_object, function::builtin_function};

 // ref:https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-boolean-objects
 pub fn create_boolean(global: &Rc<RefCell<Object>>, init: Value) -> Value {
  let global_boolean = get_global_object(global, GLOBAL_BOOLEAN_NAME.to_string());
  let boolean = create_object(global, ClassType::Boolean, None);
  let boolean_clone = Rc::clone(&boolean);
  let mut boolean_mut = (*boolean_clone).borrow_mut();
  boolean_mut.constructor = Some(Rc::downgrade(&global_boolean));
  boolean_mut.set_inner_property_value(String::from("value"), init);
  Value::BooleanObj(boolean)
}

pub fn bind_global_boolean(global:  &Rc<RefCell<Object>>) {
  let bool_rc = get_global_object(global, GLOBAL_BOOLEAN_NAME.to_string());
  let mut bool = (*bool_rc).borrow_mut();
  let create_function = builtin_function(global, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
  bool.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  if let Some(prop)= &bool.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 0f64, boolean_to_string) });
    let name = String::from("valueOf");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 1f64, value_of) });
  }
}

// Boolean.prototype.toString
fn boolean_to_string(ctx: &mut CallContext, _: Vec<Value>) -> Value {
  let value = value_of(ctx, vec![]);
  let global = ctx.global.upgrade().unwrap();
  Value::String(value.to_string(&global))
}

// Boolean.prototype.valueOf
fn value_of(ctx: &mut CallContext, _: Vec<Value>) -> Value {
  let global = ctx.global.upgrade().unwrap();
  let this_origin = ctx.this.upgrade();
  let this_rc = this_origin.unwrap();
  let init = this_rc.borrow().get_inner_property_value(String::from("value"));
  if let Some(value) = init {
    return Value::Boolean(value.to_boolean(&global))
  }
  Value::Boolean(false)
}

fn create(ctx: &mut CallContext, args: Vec<Value>) -> Value {
  let mut param = Value::Undefined;
  if args.len() > 0 {
    param = args[0].clone();
  }
  let global = ctx.global.upgrade();
  if let Some(global) = &global {
    create_boolean(global, param)
  } else {
    Value::Undefined
  }
}