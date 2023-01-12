use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Weak, Rc};
use crate::ast_node::{Statement, IdentifierLiteral};
use crate::scope::Scope;


#[derive(Debug)]
pub struct ValueInfo {
  pub name: Option<String>,
  pub value: Value,
  pub reference: Option<Value>
}

impl ValueInfo {
  pub fn set_value(&mut self, value: Value) -> Option<String> {
    if self.name == None {
      return  None;
    }
    let name = match &self.name {
        Some(name) => name.clone(),
        _ => String::from(""),
    };
    if let Some(reference) = &self.reference {
      match reference {
          Value::Object(object) => {
            object.borrow_mut().define_property_by_value( name.clone(), value);
            None
          },
          Value::Scope(scope) => {
            scope.borrow_mut().set_value( name.clone(), value);
            None
          },
          _ => Some(name.clone())
      }
    } else {
      return Some(name.clone())
    }
  }
}

#[derive(Debug)]
pub enum Value {
  // 5种基本数据类型
  String(String),
  Number(f64),
  Boolean(bool),
  Null,
  Undefined,
  // 3 种引用类型
  Object(Rc<RefCell<Object>>),
  Function(Rc<RefCell<Object>>),
  Array,
  // 其他
  NAN,
  RefObject(Weak<RefCell<Object>>),
  Scope(Rc<RefCell<Scope>>)
}

#[derive(PartialEq)]
pub enum ValueType {
  // 5种基本数据类型
  String,
  Number,
  Boolean,
  Null,
  Undefined,
  // 3 种引用类型
  Object,
  Function,
  Array,
  // 其他
  NAN,
}

impl PartialEq for Value {
  fn eq(&self, other: &Value) -> bool {
      match (self, other) {
          (Value::String(a), Value::String(b)) => *a == *b,
          (Value::Number(a), Value::Number(b)) => *a == *b,
          (Value::Boolean(a), Value::Boolean(b)) => *a == *b,
          (Value::Null, Value::Null) | (Value::Undefined, Value::Undefined) => true,
          _ => false,
      }
  }
}

impl Clone for Value {
  fn clone(&self) -> Value {
    match self {
      Value::Object(rc_value) => {
        Value::Object(Rc::clone(rc_value))
      },
      Value::Function(rc_value) => {
        Value::Function(Rc::clone(rc_value))
      },
      Value::String(str) => Value::String(str.clone()),
      Value::Number(num) => Value::Number(*num),
      Value::Boolean(bool) => Value::Boolean(*bool),
      Value::Null => Value::Null,
      Value::Undefined => Value::Undefined,
      _ => Value::Undefined,
    }
  }
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



  pub fn to_number(&self) -> Option<f64> {
    match self {
      Value::String(str) => {
        match str.parse::<f64>() {
            Ok(num) => Some(num),
            _ => None,
        }
      },
      Value::Number(number) => Some(*number),
      Value::Boolean(bool) => {
        if *bool {
          Some(1f64)
        } else {
          Some(0f64)
        }
      },
      _ => {
        // TODO: throw error
       None
      }
    }
  }
  pub fn is_boolean(&self) -> bool {
    if let Value::Boolean(_) = self {
      return true
    }
    return false
  }

  pub fn to_object(&self) -> Rc<RefCell<Object>> {
    // TODO: more type
    match self {
      Value::Object(obj) => Rc::clone(obj),
      Value::Function(function) => Rc::clone(function),
      _ => {
        // TODO: throw error
        Rc::new(RefCell::new(Object::new()))
      }
    }
  }
  pub fn get_value_type(&self) -> ValueType {
    match self {
      Value::Object(_) => ValueType::Object,
      Value::Function(_) => ValueType::Function,
      Value::Array => ValueType::Array,
      Value::String(_) => ValueType::String,
      Value::Number(_) => ValueType::Number,
      Value::Boolean(_) => ValueType::Boolean,
      Value::Null => ValueType::Null,
      Value::Undefined => ValueType::Undefined,
      _ => {
        // TODO: more
        ValueType::NAN
      },
    }
  }

  pub fn is_equal_to(&self, other_value: &Value, is_check_type: bool) -> bool {
    let self_type = self.get_value_type();
    let other_type = other_value.get_value_type();
    let is_same_type = self_type == other_type;
    if is_check_type && !is_same_type {
      return false;
    }
    if is_same_type {
      if self_type == ValueType::Boolean || self_type == ValueType::Number || self_type == ValueType::String || self_type == ValueType::Null || self_type == ValueType::Undefined {
        return self == other_value;
      }
    }

    if (self_type == ValueType::Null || self_type == ValueType::Undefined) && (other_type == ValueType::Null || other_type == ValueType::Undefined) {
      return true;
    }

    if self_type == ValueType::Object || other_type == ValueType::Object {
      // TODO: to primary value Number(123) => 123
      return false;
    }
    return self.to_number() == other_value.to_number();
  }

  // 匿名方法，需要绑定name
  pub fn bind_name(&mut self, name: String) {
    match self {
      Value::Function(function) => {
        let mut function_define =function.borrow_mut();
        let mut value = function_define.get_value().unwrap();
        match *value {
            Statement::Function(func) => {
              if func.is_anonymous {
                let mut new_func = func.clone();
                new_func.name = IdentifierLiteral {
                  literal: name.clone()
                };
                value = Box::new(Statement::Function(new_func));
                function_define.set_value(Some(value));
                function_define.define_property_by_value(String::from("name"), Value::String(String::from(name)));
              }
            },
            _ => {}
        };
      },
      _ => {}
    }
  }

}

#[derive(Debug,Clone,PartialEq)]
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
  value: Option<Box<Statement>>,
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

  pub fn set_value(&mut self, value: Option<Box<Statement>>) -> bool {
    self.value = value;
    return true;
  }

  pub fn get_value(&self) -> Option<Box<Statement>> {
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

  pub fn get_property(&self, name: String) -> Value {
    let prop = self.property.get(&name);
    if let Some(prop) = prop {
      prop.value.clone()
    } else {
      Value::Undefined
    }
  }
}

#[derive(Debug,Clone,PartialEq)]
pub struct Property {
  pub value: Value,
  // TODO: 属性的描述符 descriptor writable ，是否可枚举等
}