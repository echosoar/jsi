use std::cell::{RefCell};
use std::rc::{Rc};

use crate::ast_node::ClassType;
use crate::constants::{GLOBAL_OBJECT_NAME_LIST, GLOBAL_OBJECT_NAME, PROTO_PROPERTY_NAME};
use crate::value::Value;
use crate::context::{Context};
use super::array::bind_global_array;
use super::boolean::{bind_global_boolean};
use super::error::{bind_global_error};
use super::function::{bind_global_function};
use super::number::bind_global_number;
use super::object::{Object, Property, bind_global_object};
use super::string::bind_global_string;

pub const IS_GLOABL_OBJECT: &str = "isGlobal";

pub fn new_global_object() -> Rc<RefCell<Object>> {
  let object = Rc::new(RefCell::new(Object::new(ClassType::Object, None)));
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


// 全局对象
pub fn new_global_this() -> Rc<RefCell<Object>> {
  // TODO： 需要是一个 functcion
  let empty_native_function = Value::Undefined;
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
      // 绑定当前对象的原型
      object_borrow.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), empty_native_function.clone());
      object_borrow.set_inner_property_value(IS_GLOABL_OBJECT.to_string(), Value::Boolean(true));
      global_obj.property.insert(name.to_string(), Property { enumerable: true, value: Value::Object(Rc::clone(&object))});
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
  // 绑定  Error 的 静态方法 和 原型链方法
  bind_global_error(ctx);

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

pub fn get_global_object_prototype_by_name(ctx: &mut Context, name: &str) -> Rc<RefCell<Object>> {
  let obj = get_global_object_by_name(ctx, name);
  let obj_clone = Rc::clone(&obj);
  let obj_borrow = obj_clone.borrow_mut();
  let proto = (obj_borrow.prototype.as_ref()).unwrap();
  return Rc::clone(&proto);
}