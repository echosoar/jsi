use std::borrow::BorrowMut;
use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use crate::context::{Context};
use super::array::create_array;
// use super::array::new_array;
use super::function::builtin_function;
use super::global::{get_global_object, get_global_object_prototype_by_name, get_global_object_by_name};
use crate::ast_node::{Statement, CallContext, ClassType};
use crate::constants::{GLOBAL_OBJECT_NAME, PROTO_PROPERTY_NAME};
use crate::error::{JSIResult, JSIError};
use crate::value::{Value, INSTANTIATE_OBJECT_METHOD_NAME};

#[derive(Debug,Clone)]
// 对象
pub struct Object {
  pub class_type: ClassType,
  // 静态属性，比如 Object.keys
  pub property: HashMap<String, Property>,
  // 属性列表，对象的属性列表需要次序
  pub property_list: Vec<String>,
  // 内置属性
  pub inner_property: HashMap<String, Property>,
  // 原型对象，用于查找原型链
  // 如果是构造方法对象，如 Object，则指向一个真实存在的 Object
  // 如：Array.prototype[key] = value
  // 如：Array.prototype.constructor = Array
  pub prototype: Option<Rc<RefCell<Object>>>,
  // 如果是实例，则存在 constructor 值，指向构造方法
  // 如： arr.constructor = Array
  pub constructor: Option<Weak<RefCell<Object>>>,
  // 对象的值
  value: Option<Box<Statement>>,
}

impl Object {
  pub fn new(obj_type: ClassType, value: Option<Box<Statement>>) -> Object {
    Object {
      class_type: obj_type,
      property: HashMap::new(),
      inner_property: HashMap::new(),
      property_list: vec![],
      prototype: None,
      constructor: None,
      value,
    }
  }

  // 强制拷贝
  pub fn force_copy(&self) -> Object {
    Object {
      class_type: self.class_type.clone(),
      property: self.property.clone(),
      inner_property: self.inner_property.clone(),
      property_list: self.property_list.clone(),
      prototype: self.prototype.clone(),
      constructor: self.constructor.clone(),
      value: self.value.clone(),
    }
  }

  pub fn set_value(&mut self, value: Option<Box<Statement>>) -> bool {
    self.value = value;
    return true;
  }

  pub fn get_initializer(&self) -> Option<Box<Statement>> {
    self.value.clone()
  }

  // // TODO: descriptor
  // pub fn define_property_by_value(&mut self, name: String, value: Value) -> bool {
  //   self.define_property(name, Property { value, enumerable: false });
  //   return true;
  // }

  // TODO: descriptor
  pub fn define_property(&mut self, name: String, property: Property) -> bool {
    // 需要实现 descriptpor
    if !self.property_list.contains(&name) {
      self.property_list.push(name.clone());
    }
    self.property.insert(name, property);
    return true;
  }

  pub fn get_property_value(&self, name: String) -> Value {
    let prop = self.property.get(&name);
    if let Some(prop) = prop {
      prop.value.clone()
    } else {
      Value::Undefined
    }
  }

  pub fn get_inner_property_value(&self, name: String) -> Option<Value> {
    let prop = self.inner_property.get(&name);
    if let Some(prop) = prop {
      Some(prop.value.clone())
    } else {
      None
    }
  }

  pub fn set_inner_property_value(&mut self, name: String, value: Value) {
    self.inner_property.insert(name, Property { enumerable: false, value });
  }

  // 获取属性：从当前属性；从构造器的原型链上面寻找值
  pub fn get_value(&self, name: String) -> Value {
    if name == String::from("prototype") {
      if let Some(proto) = &self.prototype {
        return Value::Object(Rc::clone(proto));
      } else {
        return Value::Undefined
      }
    }
    let prop = self.property.get(&name);
    if let Some(prop) = prop {
      return prop.value.clone()
    } else {
      let proto = self.get_inner_property_value(PROTO_PROPERTY_NAME.to_string());
      if let Some(proto_value) = &proto{
        if let Value::RefObject(proto_obj) = proto_value {
          // 获取到 __proto__ == Constroctor.prototype
          let proto_op = proto_obj.upgrade();
          if let Some(proto) =proto_op {
            let mut proto_rc = proto;
            loop {
              let proto_rc_clone = Rc::clone(&proto_rc);
              let proto = proto_rc_clone.borrow();
              let prop = proto.property.get(&name);
              if let Some(property) = prop {
                return property.value.clone()
              } else {
                let new_proto = proto.get_inner_property_value(PROTO_PROPERTY_NAME.to_string());
                if let Some(new_proto) = new_proto {
                  if let Value::RefObject(new_proto_obj) = new_proto {
                    let new_proto_op = new_proto_obj.upgrade();
                    if let Some(proto) = new_proto_op {
                      proto_rc = Rc::clone(&proto);
                      continue;
                    }
                  }
                }
              }
              break;
            }
            // loop {
            //   let broow = Rc::clone(&cur);
            //   let cur_mut = broow.borrow();
            //   let prop = cur_mut.property.get(&name);
            //   if let Some(prop) = prop {
            //     return prop.value.clone()
            //   } else {
            //     if let Some(constructor) = &cur_mut.prototype {
            //       cur = Rc::clone(constructor);
            //     } else {
            //       break;
            //     }
            //   }
            // }
          }
        }
        
      }
    }
    Value::Undefined
  }

  pub fn call(call_ctx: &mut CallContext, name: String, arguments:Vec<Value>) -> JSIResult<Value> {
    let this = call_ctx.this.upgrade().unwrap();
    let fun = {
      //  处理临时借用
      let this_mut = (*this).borrow_mut();
      this_mut.get_value(name.clone())
    };
   
    if let Value::Function(function_define) = &fun {
      // 获取 function 定义
      let function_define_value = {
        let fun_clone = Rc::clone(function_define);
        let fun_obj = (*fun_clone).borrow_mut();
        fun_obj.get_initializer().unwrap()
      };
      
      // 内置方法
      if let Statement::BuiltinFunction(builtin_function) = function_define_value.as_ref() {
        // let mut ctx = CallContext{ global: ctx.global, this: Rc::downgrade(&function_define) };
        return (builtin_function)(call_ctx, arguments);
      }
      if let Statement::Function(_) = function_define_value.as_ref() {
        let call_function_define = Rc::clone(function_define);
        return call_ctx.call_function(call_function_define, Some(Value::Function(Rc::clone(function_define))), None, arguments);
      }
    }
    Err(JSIError::new(crate::error::JSIErrorType::ReferenceError, format!("{:?} method not exists", name), 0, 0))
  }
}

#[derive(Debug,Clone)]
pub struct Property {
  // 是否可枚举
  pub enumerable: bool,
  pub value: Value,
  // TODO: 属性的描述符 descriptor writable ，是否可枚举等
}

// 实例化对象
pub fn create_object(ctx: &mut Context, obj_type: ClassType, value: Option<Box<Statement>>) -> Rc<RefCell<Object>> {
  let object = Rc::new(RefCell::new(Object::new(obj_type, value)));
  let object_clone = Rc::clone(&object);
  let mut object_mut = (*object_clone).borrow_mut();
  // 绑定 obj.constructor = global.Object
  let global_object = get_global_object_by_name(ctx, GLOBAL_OBJECT_NAME);
  let global_prototype = get_global_object_prototype_by_name(ctx, GLOBAL_OBJECT_NAME);
  object_mut.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&global_prototype)));
  let weak_rc = Rc::downgrade(&global_object);
  object_mut.constructor = Some(weak_rc);
  
  object
}

pub fn bind_global_object(ctx: &mut Context) {
  let obj_rc = get_global_object(ctx, GLOBAL_OBJECT_NAME.to_string());

  // 绑定实例化方法
  let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
  let object_keys_fun = builtin_function(ctx, String::from("keys"), 1f64, object_keys);
  let object_get_own_property_names_fun = builtin_function(ctx, String::from("getOwnPropertyNames"), 1f64, object_get_own_property_names);
  let object_get_prototype_of_fun = builtin_function(ctx, String::from("getPrototypeOf"), 1f64, object_get_prototype_of);
  let object_to_string_fun = builtin_function(ctx, String::from("toString"), 0f64, to_string);


  let mut obj = (*obj_rc).borrow_mut();
  obj.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  let property = obj.property.borrow_mut();

  // Object.keys
  let name = String::from("keys");
  property.insert(name.clone(), Property { enumerable: true, value: object_keys_fun  });

  // Object.getOwnPropertyNames
  let name = String::from("getOwnPropertyNames");
  property.insert(name.clone(), Property { enumerable: true, value: object_get_own_property_names_fun });

  // Object.keys
  let name = String::from("getPrototypeOf");
  property.insert(name.clone(), Property { enumerable: true, value: object_get_prototype_of_fun });

  if let Some(prop)= &obj.prototype {

    let prototype_rc = Rc::clone(prop);
    let mut prototype = (*prototype_rc).borrow_mut();

    // 原型对象的原型 [[Property]] 为 null // Object.prototype.__proto__ == null
    prototype.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::Null);

    // Object.prototype.toString
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: object_to_string_fun });
  }
 
}



// Object.keys()
fn object_keys(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let array = create_array(call_ctx.ctx, 0);
  let array_obj = match array {
    Value::Array(arr) => Some(arr),
    _ => None
  }.unwrap();

  if args.len() < 1 {
    return Ok(Value::Array(array_obj));
  }
  let weak = Rc::downgrade(&array_obj);
  let obj_rc= args[0].to_object(call_ctx.ctx);
  let new_call_ctx = &mut CallContext {
    ctx: call_ctx.ctx,
    this: weak,
    reference: None,
  };
  let obj = obj_rc.borrow();
  for key in obj.property_list.iter() {
    let prop = obj.property.get(key);
    if let Some(property) = prop {
      if property.enumerable {
        Object::call(new_call_ctx, String::from("push"), vec![Value::String(key.clone())])?;
      }
    }
  }
  return Ok(Value::Array(array_obj));
}

// Object.getOwnPropertyNames
fn object_get_own_property_names(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let array = create_array(call_ctx.ctx, 0);
  let array_obj = match array {
    Value::Array(arr) => Some(arr),
    _ => None
  }.unwrap();

  if args.len() < 1 {
    return Ok(Value::Array(array_obj));
  }
  let weak = Rc::downgrade(&array_obj);
  let obj_rc= args[0].to_object(call_ctx.ctx);
  let new_call_ctx = &mut CallContext {
    ctx: call_ctx.ctx,
    this: weak,
    reference: None,
  };
  let obj = obj_rc.borrow();
  for key in obj.property_list.iter() {
    Object::call(new_call_ctx, String::from("push"), vec![Value::String(key.clone())])?;
  }
  return Ok(Value::Array(array_obj));
}

// Object.getPrototypeOf
fn object_get_prototype_of(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut value = Value::Undefined;
  // item.creater
  if args.len() > 0 {
    value = args[0].clone();
  }
  let obj = value.to_object(call_ctx.ctx);
  let create_method = obj.borrow().get_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string());
  if let Some(_) = create_method{
    // TODO: function return undefined
    return Ok(Value::Undefined);
  }

  let obj_rc = obj.borrow();
  let inner_proto = obj_rc.get_inner_property_value(PROTO_PROPERTY_NAME.to_string());
  if let Some(proto) = inner_proto {
    return Ok(proto);
  }
  // item.prototype
  Ok(obj_rc.get_value(String::from("prototype")))
}

// Object.prototype.toString
fn to_string(ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
  let this_origin = ctx.this.upgrade();
  let this_rc = this_origin.unwrap();
  let this = this_rc.borrow();
  let mut obj_type : String = "[object ".to_owned();
  obj_type.push_str(this.class_type.to_string().as_str());
  obj_type.push(']');
 Ok( Value::String(obj_type))
}

fn create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  if args.len() > 0 {
    let obj = args[0].to_object_value(call_ctx.ctx);
    return Ok(obj)
  }
  Ok(Value::Object(create_object(call_ctx.ctx, ClassType::Object, None)))
}