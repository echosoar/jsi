use std::{rc::Rc, cell::RefCell};

use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}};

use super::{object::{create_object, Object, Property}, global::get_global_object, function::builtin_function};

 pub fn create_string(global: &Rc<RefCell<Object>>, init: Value) -> Value {
  let global_string = get_global_object(global, String::from("String"));
  let string = create_object(global, ClassType::String, None);
  let string_clone = Rc::clone(&string);
  let mut string_mut = (*string_clone).borrow_mut();
  string_mut.constructor = Some(Rc::downgrade(&global_string));
  string_mut.set_inner_property_value(String::from("value"), init);
  Value::StringObj(string)
}

pub fn bind_global_string(global:  &Rc<RefCell<Object>>) {
  let string_rc = get_global_object(global, String::from("String"));
  let mut string = (*string_rc).borrow_mut();
  let create_function = builtin_function(global, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
  string.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  if let Some(prop)= &string.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = (*prototype_rc).borrow_mut();
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 0f64, to_string) });
    let name = String::from("valueOf");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 1f64, to_string) });
  }
}


fn create(ctx: &mut CallContext, args: Vec<Value>) -> Value {
  let mut param = Value::Undefined;
  if args.len() > 0 {
    param = args[0].clone();
  }
  let global = ctx.global.upgrade();
  if let Some(global) = &global {
    create_string(global, param)
  } else {
    Value::Undefined
  }
}

// String.prototype.toString
fn to_string(ctx: &mut CallContext, _: Vec<Value>) -> Value {
  let global = ctx.global.upgrade().unwrap();
  let this_origin = ctx.this.upgrade();
  let this_rc = this_origin.unwrap();
  let init = this_rc.borrow().get_inner_property_value(String::from("value"));
  if let Some(value) = init {
    return Value::String(value.to_string(&global))
  }
  Value::String(String::from(""))
}