use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::{Rc};
use super::array::new_array;
use crate::ast_node::{Statement, CallContext};
use crate::value::Value;
#[derive(Debug,Clone)]
pub struct Object {
  // 构造此对象的构造函数
  // 比如函数的 constructor 就是 Function
  // constructor
  pub property: HashMap<String, Property>,
  // 内置属性
  pub inner_property: HashMap<String, Property>,
  // 属性列表，对象的属性列表需要次序
  pub property_list: Vec<String>,
  // 原型对象，用于查找原型链
  pub prototype: Option<Box<Object>>,
  // 对象的值
  value: Option<Box<Statement>>,
}

impl Object {
  pub fn new(value: Option<Box<Statement>>) -> Object {
    Object {
      property: HashMap::new(),
      inner_property: HashMap::new(),
      property_list: vec![],
      prototype: None,
      value,
    }
  }

  pub fn set_value(&mut self, value: Option<Box<Statement>>) -> bool {
    self.value = value;
    return true;
  }

  pub fn get_value(&self) -> Option<Box<Statement>> {
    self.value.clone()
  }

  // TODO: descriptor
  pub fn define_property_by_value(&mut self, name: String, value: Value) -> bool {
    self.define_property(name, Property { value, enumerable: false });
    return true;
  }

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

  // 定义原型链上面的属性
  pub fn define_prototype_property(&mut self, name: String, property: Property) -> bool {
    let proto = self.prototype.as_mut().unwrap();
    proto.define_property(name, property)
  }

  // [静态]调用内置方法
  pub fn call_builtin(method_name: String, args: Vec<Value>, ctx: &mut CallContext) -> Value {
    let this_rc =  ctx.this.upgrade().unwrap();
    let this = this_rc.borrow_mut();
    if let Some(value) = this.inner_property.get(&method_name) {
      if let Value::Function(fun) = &value.value {
        let fun_value = fun.borrow().get_value();
        if let Some(fun_value) = fun_value {
          if let Statement::BuiltinFunction(fun_value) = *fun_value {
            return (fun_value)(ctx, args);
          }
        }
      }
    }
    Value::Undefined
  }

  // 转换为字符串
  pub fn to_string(&self, ctx: &mut CallContext) -> String {
    let value = Object::call_builtin(String::from("to_string"), vec![], ctx);
    if let Value::String(str) = value {
      return str;
    }
    String::from("")
  }
}

#[derive(Debug,Clone,PartialEq)]
pub struct Property {
  // 是否可枚举
  pub enumerable: bool,
  pub value: Value,
  // TODO: 属性的描述符 descriptor writable ，是否可枚举等
}


// 基础对象，绑定好原型链
pub fn new_base_object(value: Option<Box<Statement>>) -> Rc<RefCell<Object>> {
  let object = Rc::new(RefCell::new(Object::new(value)));
  let object_clone = Rc::clone(&object);
  let mut object_mut = (*object_clone).borrow_mut();
  let prototype =  Rc::new(RefCell::new(Object::new(None)));
  // constructor 弱引用
  (*prototype).borrow_mut().define_property_by_value(String::from("constructor"), Value::RefObject(Rc::downgrade(&object)));
  object_mut.define_property_by_value(String::from("prototype"), Value::Object(prototype));
  object
}


// Object 
pub fn global_object() -> Rc<RefCell<Object>> {
  let obj = new_base_object(None);
  let obj_mut = Rc::clone(&obj);
  let object_keys_method = new_base_object(Some(Box::new(Statement::BuiltinFunction(object_keys))));
  obj_mut.borrow_mut().define_property_by_value(String::from("keys"), Value::Function(object_keys_method));
  obj
}


// Object.keys()
fn object_keys(_: &mut CallContext, args: Vec<Value>) -> Value {
  let array =  new_array(0);
  let result = match array {
    Value::Array(arr) => Some(arr),
    _ => None
  }.unwrap();
  if args.len() < 1 {
    return Value::Array(result);
  }
  let obj = args[0].to_object();
  let obj = obj.borrow();
  let mut index = 0;
  for key in obj.property_list.iter() {
    let prop = obj.property.get(key);
    if let Some(property) = prop {
      if property.enumerable {
        result.borrow_mut().define_property_by_value(index.to_string(), Value::String(key.clone()));
        index += 1;
      }
    }
    
  }
  // TODO: using array.push
  result.borrow_mut().define_property_by_value(String::from("length"),  Value::Number(index as f64));
  return Value::Array(result)
}