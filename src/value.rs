use std::collections::HashMap;
use std::rc::Weak;
use crate::ast_node::FunctionDeclarationStatement;

#[derive(Debug,Clone)]
pub enum Value {
  String(String),
  Number(f64),
  Boolean(bool),
  Null,
  Object(Object),
  Undefined,
  NAN,
  Function(FunctionDeclarationStatement),
  CycleRefObject(Weak<Object>),
}

impl Value {
  pub fn is_string(&self) -> bool {
    if let Value::String(_) = self {
      return true
    }
    return false
  }
  pub fn to_string(&self) -> String {
    match self {
      Value::String(str) => str.clone(),
      Value::Number(number) => number.to_string(),
      Value::Boolean(bool) => {
        if *bool {
          String::from("true")
        } else {
          String::from("false")
        }
      },
      Value::NAN => String::from("NaN"),
      _ => String::from(""),
    }
  }
  pub fn is_number(&self) -> bool {
    if let Value::Number(_) = self {
      return true
    }
    return false
  }

  pub fn is_infinity(&self) -> bool {
    if let Value::Number(number) = self {
      return *number == f64::INFINITY || *number == -f64::INFINITY;
    }
    return false
  }

  pub fn is_nan(&self) -> bool {
    if let Value::NAN = self {
      return true
    }
    return false
  }

  pub fn to_number(&self) -> f64 {
    match self {
      Value::String(str) => str.parse::<f64>().unwrap(),
      Value::Number(number) => *number,
      Value::Boolean(bool) => {
        if *bool {
          1f64
        } else {
          0f64
        }
      },
      _ => 0f64,
    }
  }
  pub fn is_boolean(&self) -> bool {
    if let Value::Boolean(_) = self {
      return true
    }
    return false
  }

}

#[derive(Debug,Clone)]
pub struct Object {
  // 构造此对象的构造函数
  // 比如函数的 constructor 就是 Function
  // constructor
  property: HashMap<String, Property>,
  // 属性列表，对象的属性列表需要次序
  property_list: Vec<String>,
  // 原型对象，用于查找原型链
  pub prototype: Option<Box<Object>>,
  // 对象的值
  value: Option<Box<Value>>,
}


impl Object {

  pub fn new() -> Object {
    Object {
      property: HashMap::new(),
      property_list: vec![],
      prototype: None,
      value: None,
    }
  }

  pub fn set_value(&mut self, value: Option<Box<Value>>) -> bool {
    self.value = value;
    return true;
  }

  pub fn get_value(&self) -> Option<Box<Value>> {
    self.value.clone()
  }

  // TODO: descriptor
  pub fn define_property_by_value(&mut self, name: String, value: Value) -> bool {
    self.define_property(name, Property { value });
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
}

#[derive(Debug,Clone)]
pub struct Property {
  pub value: Value,
  // TODO: 属性的描述符 descriptor writable ，是否可枚举等
}