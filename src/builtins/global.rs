use std::cell::{RefCell};
use std::rc::{Rc};

use crate::value::Value;

use super::array::bind_global_array;
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
pub struct Global {
  pub object: Rc<RefCell<Object>>,
  pub array: Rc<RefCell<Object>>,
  pub function: Rc<RefCell<Object>>,
}

#[derive(Debug, Clone)]
pub enum ClassType {
  Object,
  Array,
  Function,
  String,
  Boolean,
  Number,
  Null,
}

impl Global {
  pub fn new() -> Global {
    // Object
    let object = new_global_object();
    let array = new_global_object();
    let function = new_global_object();
    let global = Global{
      object,
      array,
      function,
    };
    // 绑定 Object 的 静态方法 和 原型链方法
    bind_global_object(&global);
    // 绑定 Array 的 静态方法 和 原型链方法
    bind_global_array(&global);
    return global;
  }
}