use std::cell::{RefCell};
use std::rc::{Rc};

use crate::ast_node::ClassType;
use crate::value::Value;

use super::array::bind_global_array;
use super::boolean::bind_global_boolean;
use super::error::{bind_global_error, GLOBAL_ERROR_NAME};
use super::function::bind_global_function;
use super::number::bind_global_number;
use super::object::{Object, Property, bind_global_object};
use super::string::bind_global_string;
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
  let global_string = new_global_object();
  let global_number = new_global_object();
  let global_boolean = new_global_object();
  let global_error = new_global_object();
  let global_clone = Rc::clone(&global);
  {
    let mut global_obj = global_clone.borrow_mut();
    global_obj.property.insert(String::from("Object"), Property { enumerable: true, value: Value::Object(Rc::clone(&object))});
    global_obj.property.insert(String::from("Array"), Property { enumerable: true, value: Value::Object(Rc::clone(&array))});
    global_obj.property.insert(String::from("Function"), Property { enumerable: true, value: Value::Object(Rc::clone(&function))});
    global_obj.property.insert(String::from("String"), Property { enumerable: true, value: Value::Object(Rc::clone(&global_string))});
    global_obj.property.insert(String::from("Number"), Property { enumerable: true, value: Value::Object(Rc::clone(&global_number))});
    global_obj.property.insert(String::from("Boolean"), Property { enumerable: true, value: Value::Object(Rc::clone(&global_boolean))});
    global_obj.property.insert(GLOBAL_ERROR_NAME.to_string(), Property { enumerable: true, value: Value::Object(Rc::clone(&global_error))});
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
  // 绑定  String 的 静态方法 和 原型链方法
  bind_global_string(&global_clone);
  // 绑定  Boolean 的 静态方法 和 原型链方法
  bind_global_boolean(&global_clone);
   // 绑定  Number 的 静态方法 和 原型链方法
   bind_global_number(&global_clone);
  // 绑定  Error 的 静态方法 和 原型链方法
  bind_global_error(&global_clone);
  return global;
}

pub fn get_global_object(global: &Rc<RefCell<Object>>, name: String) -> Rc<RefCell<Object>> {
  let clone_global_mut = (*global).borrow_mut();
  let obj = clone_global_mut.get_value(name.clone()).to_object(global);
  return obj;
}

pub fn get_global_object_by_name(global: &Rc<RefCell<Object>>, name: &str) -> Rc<RefCell<Object>> {
  let clone_global_mut = (*global).borrow_mut();
  let obj = clone_global_mut.get_value(name.to_string().clone()).to_object(global);
  return obj;
}