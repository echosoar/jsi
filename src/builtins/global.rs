use std::cell::{RefCell};
use std::rc::{Rc};

use crate::ast_node::ClassType;
use crate::constants::{GLOBAL_OBJECT_NAME_LIST};
use crate::value::Value;
use crate::context::{Context};
use super::array::bind_global_array;
use super::boolean::{bind_global_boolean};
use super::error::{bind_global_error};
use super::function::bind_global_function;
use super::number::bind_global_number;
use super::object::{Object, Property, bind_global_object};
use super::string::bind_global_string;

pub const IS_GLOABL_OBJECT: &str = "isGlobal";

pub fn new_global_object() -> Rc<RefCell<Object>> {
  let object = Rc::new(RefCell::new(Object::new(ClassType::Object, None)));
  let object_clone = Rc::clone(&object);
  let mut object_mut = (*object_clone).borrow_mut();
  // 创建原型链
  let prototype =  Rc::new(RefCell::new(Object::new(ClassType::Object, None)));
  // constructor 弱引用
  (*prototype).borrow_mut().define_property(String::from("constructor"), Property {
    enumerable: false,
    value: Value::RefObject(Rc::downgrade(&object)),
  });
  object_mut.prototype = Some(prototype);
  object
}


// 全局对象
pub fn new_global_this() -> Rc<RefCell<Object>> {
  // Global
  let global = new_global_object();
  let global_clone = Rc::clone(&global);
  {
    let mut global_obj = global_clone.borrow_mut();
    // 绑定全局对象
    for name in GLOBAL_OBJECT_NAME_LIST.iter() {
      let object = new_global_object();
      let object_rc = Rc::clone(&object);
      let mut object_borrow = object_rc.borrow_mut();
      object_borrow.set_inner_property_value(IS_GLOABL_OBJECT.to_string(), Value::Boolean(true));
      global_obj.property.insert(name.to_string(), Property { enumerable: true, value: Value::Object(Rc::clone(&object))});
    }
  }
  return global;
}

pub fn bind_global(ctx: &Context) {
  // 绑定 Object 的 静态方法 和 原型链方法
  bind_global_object(ctx);
  // 绑定 Function 的 静态方法 和 原型链方法
  bind_global_function(ctx);
  // 绑定 Array 的 静态方法 和 原型链方法
  bind_global_array(ctx);
  // 绑定  String 的 静态方法 和 原型链方法
  bind_global_string(ctx);
  // 绑定  Boolean 的 静态方法 和 原型链方法
  bind_global_boolean(ctx);
   // 绑定  Number 的 静态方法 和 原型链方法
   bind_global_number(ctx);
  // 绑定  Error 的 静态方法 和 原型链方法
  bind_global_error(ctx);
}

pub fn get_global_object(ctx: &Context, name: String) -> Rc<RefCell<Object>> {
  let clone_global_mut = ctx.global.borrow_mut();
  let obj = clone_global_mut.get_value(name.clone()).to_object(ctx);
  return obj;
}

pub fn get_global_object_by_name(ctx: &Context, name: &str) -> Rc<RefCell<Object>> {
  let clone_global_mut = ctx.global.borrow_mut();
  let obj = clone_global_mut.get_value(name.to_string().clone()).to_object(ctx);
  return obj;
}