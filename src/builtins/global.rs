use std::cell::{RefCell};
use std::rc::{Rc};

use crate::ast_node::ClassType;
use crate::constants::{GLOBAL_OBJECT_NAME_LIST, GLOBAL_OBJECT_NAME, PROTO_PROPERTY_NAME, GLOBAL_ERROR_NAME, GLOBAL_TYPE_ERROR_NAME};
use crate::value::Value;
use crate::context::{Context};
use super::array::bind_global_array;
use super::boolean::{bind_global_boolean};
use super::error::{bind_global_error};
use super::function::{bind_global_function};
use super::number::bind_global_number;
use super::object::{Object, Property, bind_global_object};
use super::string::bind_global_string;
use super::promise::bind_global_promise;

pub const IS_GLOABL_OBJECT: &str = "isGlobal";

// 创建全局构造函数对象 (如 Array, Promise 等)
pub fn new_global_constructor() -> Rc<RefCell<Object>> {
  let object = Rc::new(RefCell::new(Object::new(ClassType::Function, None)));
  let object_clone = Rc::clone(&object);
  let mut object_mut = (*object_clone).borrow_mut();

  // 创建原型对象 prototype
  let prototype =  Rc::new(RefCell::new(Object::new(ClassType::Object, None)));
  let prototype_clone = Rc::clone(&prototype);
  let mut prototype_mut = prototype_clone.borrow_mut();
  prototype_mut.define_property(String::from("constructor"), Property {
    enumerable: false,
    value: Value::RefObject(Rc::downgrade(&object)),
  });
  object_mut.prototype = Some(prototype);
  object
}

pub fn new_global_object() -> Rc<RefCell<Object>> {
  let object = Rc::new(RefCell::new(Object::new(ClassType::Object, None)));
  let object_clone = Rc::clone(&object);
  let mut object_mut = (*object_clone).borrow_mut();

  // 创建原型对象 prototype
  // Object.prototype 是所有对象的原型
  // 原型上面的方法，通过 bind_global_object 挂载
  let prototype =  Rc::new(RefCell::new(Object::new(ClassType::Object, None)));
  let prototype_clone = Rc::clone(&prototype);
  let mut prototype_mut = prototype_clone.borrow_mut();
  prototype_mut.define_property(String::from("constructor"), Property {
    enumerable: false,
    value: Value::RefObject(Rc::downgrade(&object)),
  });
  object_mut.prototype = Some(prototype);
  object
}

// 全局对象
pub fn new_global_this() -> Rc<RefCell<Object>> {
  // 先创建全局 Object，以及 Object.prototype
  let first_obj = new_global_object();
  let first_obj_clone = Rc::clone(&first_obj);
  let mut first_obj_borrow = (*first_obj_clone).borrow_mut();
  first_obj_borrow.set_inner_property_value(IS_GLOABL_OBJECT.to_string(), Value::Boolean(true));
  first_obj_borrow.set_inner_property_value(String::from("name"), Value::String(GLOBAL_OBJECT_NAME.to_string()));
  // native function
  let native_function = new_global_object();
  {
    let native_function_rc = Rc::clone(&native_function);
    let mut native_borrow = native_function_rc.borrow_mut();
    // 绑定 native_function的原型到全局 Object.prototype
    // Object['__proto__'] === native_function
    first_obj_borrow.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&native_function)));
    // native_function.__proto__ === Object['__proto__'].__proto__ === Object.prototype
    if let Some(prop) = &first_obj_borrow.prototype {
      native_borrow.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(prop)));
    }
  }
  
  // Global
  let global = new_global_object();
  let global_clone = Rc::clone(&global);
  {
    let mut global_obj = global_clone.borrow_mut();
    global_obj.property.insert(GLOBAL_OBJECT_NAME.to_string(), Property { enumerable: true, value: Value::Object(Rc::clone(&first_obj))});
    // 创建并绑定全局对象
    for name in GLOBAL_OBJECT_NAME_LIST.iter() {
      if name == &GLOBAL_OBJECT_NAME {
        continue;
      }
      
      // 构造函数应该被创建为 Function 而不是 Object
      let object = new_global_constructor();
      let object_rc = Rc::clone(&object);
      let mut object_borrow = object_rc.borrow_mut();
      // 绑定当前对象的原型
      object_borrow.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::Object(Rc::clone(&native_function)));

      // 标记是全局对象
      object_borrow.set_inner_property_value(IS_GLOABL_OBJECT.to_string(), Value::Boolean(true));
      // 添加对象 name
      object_borrow.set_inner_property_value(String::from("name"), Value::String(name.to_string()));
      // 存储为 Function 而不是 Object
      global_obj.property.insert(name.to_string(), Property { enumerable: true, value: Value::Function(Rc::clone(&object))});
    }
  }
  
  return global;
}

pub fn bind_global(ctx: &mut Context) {
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
  // 绑定  Promise 的 静态方法 和 原型链方法
  bind_global_promise(ctx);
  // 绑定  Error 的 静态方法 和 原型链方法
  bind_global_error(ctx, GLOBAL_ERROR_NAME);
  bind_global_error(ctx, GLOBAL_TYPE_ERROR_NAME);

  let obj_rc = get_global_object(ctx, GLOBAL_OBJECT_NAME.to_string());
  let obj_rc =  obj_rc.borrow();
  let obj_prototype_rc = &obj_rc.prototype;
  if let Some(obj_prototype) = obj_prototype_rc {
    // 绑定 prototype.[[Property]]
    for name in GLOBAL_OBJECT_NAME_LIST.iter() {
      if name == &GLOBAL_OBJECT_NAME {
        continue;
      }
      let global_item_rc =  get_global_object(ctx, name.to_string());
      let global_item_ref = global_item_rc.borrow();
      if let Some(prop)= &global_item_ref.prototype {

        let prototype_rc = Rc::clone(prop);
        let mut prototype = (*prototype_rc).borrow_mut();
    
        // 除 Object 外，其他的原型对象的原型 [[Property]] 都是 Object 的原型对象
        prototype.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&obj_prototype)));

      }
    }
  }
}

pub fn get_global_object(ctx: &mut Context, name: String) -> Rc<RefCell<Object>> {

  let value = {
    let clone_global_mut = ctx.global.borrow_mut();
    clone_global_mut.get_value(name.clone())
  };

  let obj = value.to_object(ctx);
  return obj;
}

pub fn get_global_object_by_name(ctx: &mut Context, name: &str) -> Rc<RefCell<Object>> {
  let value = {
    let clone_global_mut = ctx.global.borrow_mut();
    clone_global_mut.get_value(name.to_string().clone())
  };
  let obj = value.to_object(ctx);
  return obj;
}
// 获取全局对象的 prototype
pub fn get_global_object_prototype_by_name(ctx: &mut Context, name: &str) -> Rc<RefCell<Object>> {
  let obj = get_global_object_by_name(ctx, name);
  let obj_clone = Rc::clone(&obj);
  let obj_borrow = obj_clone.borrow_mut();
  let proto = (obj_borrow.prototype.as_ref()).unwrap();
  return Rc::clone(&proto);
}