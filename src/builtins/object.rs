use std::borrow::BorrowMut;
use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use super::array::create_array;
// use super::array::new_array;
use super::function::builtin_function;
use super::global::get_global_object;
use crate::ast_node::{Statement, CallContext, ClassType};
use crate::value::Value;
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

  // 从当前属性和原型链上面寻找值
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
      if let Some(constructor) = &self.constructor {
        let constructor_rc = constructor.upgrade();
        if let Some(cur) = constructor_rc {
          let mut cur = cur;
          loop {
            let broow = Rc::clone(&cur);
            let cur_mut = broow.borrow();
            let prop = cur_mut.property.get(&name);
            if let Some(prop) = prop {
              return prop.value.clone()
            } else {
              if let Some(constructor) = &cur_mut.prototype {
                cur = Rc::clone(constructor);
              } else {
                break;
              }
            }
          }
        }
      }
    }
    Value::Undefined
  }

  pub fn call(ctx: &mut CallContext, name: String, arguments:Vec<Value>) -> Value {
    let this = ctx.this.upgrade().unwrap();
    let fun = {
      //  处理临时借用
      let this_mut = (*this).borrow_mut();
      this_mut.get_value(name)
    };
   
    if let Value::Function(function_define) = &fun {
      // 获取 function 定义
      let fun_clone = Rc::clone(function_define);
      let fun_obj = (*fun_clone).borrow_mut();
      let function_define_value = fun_obj.get_initializer().unwrap();
      
      // 内置方法
      if let Statement::BuiltinFunction(builtin_function) = function_define_value.as_ref() {
        // let mut ctx = CallContext{ global: ctx.global, this: Rc::downgrade(&function_define) };
        return (builtin_function)( ctx, arguments);
      }
    }
    Value::Undefined
  }
}

#[derive(Debug,Clone,PartialEq)]
pub struct Property {
  // 是否可枚举
  pub enumerable: bool,
  pub value: Value,
  // TODO: 属性的描述符 descriptor writable ，是否可枚举等
}


// 实例化对象
pub fn create_object(global: &Rc<RefCell<Object>>, obj_type: ClassType, value: Option<Box<Statement>>) -> Rc<RefCell<Object>> {
  let object = Rc::new(RefCell::new(Object::new(obj_type, value)));
  let object_clone = Rc::clone(&object);
  let mut object_mut = (*object_clone).borrow_mut();
  // 绑定 obj.constructor = global.Object
  let global_object = get_global_object(global, String::from("Object"));
  object_mut.constructor = Some(Rc::downgrade(&global_object));
  object
}



pub fn bind_global_object(global: &Rc<RefCell<Object>>) {
  let obj_rc = get_global_object(global, String::from("Object"));
  let mut obj = (*obj_rc).borrow_mut();
  let property = obj.property.borrow_mut();
  // Object.keys
  let name = String::from("keys");
  property.insert(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 1f64, object_keys) });

  if let Some(prop)= &obj.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = (*prototype_rc).borrow_mut();
    // Object.prototype.toString
    let name = String::from("toString");
    prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 0f64, to_string) });
  }
 
}



// Object.keys()
fn object_keys(ctx: &mut CallContext, args: Vec<Value>) -> Value {
  let hangle_global = ctx.global.upgrade().unwrap();
  let array = create_array(&hangle_global, 0);
  let array_obj = match array {
    Value::Array(arr) => Some(arr),
    _ => None
  }.unwrap();

  if args.len() < 1 {
    return Value::Array(array_obj);
  }
  let weak = Rc::downgrade(&array_obj);
  let call_ctx = &mut CallContext {
    global: Rc::downgrade(&hangle_global),
    this: weak,
    reference: None,
  };
  let obj_rc= args[0].to_object(&hangle_global);
  let obj = obj_rc.borrow();
  for key in obj.property_list.iter() {
    let prop = obj.property.get(key);
    if let Some(property) = prop {
      if property.enumerable {
        Object::call(call_ctx, String::from("push"), vec![Value::String(key.clone())]);
      }
    }
  }
  return Value::Array(array_obj);
}


// Object.keys()
fn to_string(ctx: &mut CallContext, _: Vec<Value>) -> Value {
  let this_origin = ctx.this.upgrade();
  let this_rc = this_origin.unwrap();
  let this = this_rc.borrow();
  let mut obj_type : String = "[object ".to_owned();
  obj_type.push_str(this.class_type.to_string().as_str());
  obj_type.push(']');
  Value::String(obj_type)
}