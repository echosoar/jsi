use std::cell::{RefCell};
use std::rc::{Rc};

use crate::ast_node::ClassType;
use crate::value::Value;

use super::array::bind_global_array;
use super::function::bind_global_function;
use super::object::{Object, Property, bind_global_object};
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
  // Object
  let object = new_global_object();
  let array = new_global_object();
  let function = new_global_object();
  let global_clone = Rc::clone(&global);
  {
    let mut global_obj = global_clone.borrow_mut();
    global_obj.property.insert(String::from("Object"), Property { enumerable: true, value: Value::Object(Rc::clone(&object))});
    global_obj.property.insert(String::from("Array"), Property { enumerable: true, value: Value::Object(Rc::clone(&array))});
    global_obj.property.insert(String::from("Function"), Property { enumerable: true, value: Value::Object(Rc::clone(&function))});
  }

  // let global = Global{
  //   object,
  //   array,
  //   function,
  // };
  // 绑定 Object 的 静态方法 和 原型链方法
  bind_global_object(&global_clone);
  // 绑定 Function 的 静态方法 和 原型链方法
  bind_global_function(&global_clone);
  // 绑定 Array 的 静态方法 和 原型链方法
  bind_global_array(&global_clone);
  return global;
}

pub fn get_global_object(global: &Rc<RefCell<Object>>, name: String) -> Rc<RefCell<Object>> {
  let clone_global_mut = (*global).borrow_mut();
  let obj = clone_global_mut.get_value(name.clone()).to_object();
  return obj;
}