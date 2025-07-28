use std::cell::RefCell;
use std::rc::{Weak, Rc};
use crate::ast_node::{Statement, IdentifierLiteral, ClassType, CallContext, Expression};
use crate::ast_token::Token;
use crate::builtins::boolean::create_boolean;
use crate::builtins::function::get_builtin_function_name;
use crate::builtins::number::create_number;
use crate::builtins::object::{Object, Property};
use crate::builtins::string::create_string;
use crate::bytecode::ByteCode;
use crate::context::{Context};
use crate::error::{JSIResult, JSIError, JSIErrorType};
use crate::scope::{self, Scope};


#[derive(Debug, Clone)]
pub struct ValueInfo {
  // 变量名
  pub name: Option<String>,
  // 值
  pub value: Value,
  // 访问的路径
  pub access_path: String,
  // a.c，a 就是 reference，c 就是 name 变量名
  pub reference: Option<Value>,
  pub is_const: bool,
}

impl ValueInfo {
  pub fn set_value(&mut self, ctx: &mut Context,value: Value) -> JSIResult<Option<String>> {
    if self.name == None {
      return  Err(JSIError::new(JSIErrorType::SyntaxError, format!("Invalid left-hand side in assignment"), 0, 0));
    }
    if self.is_const {
      return  Err(JSIError::new(JSIErrorType::TypeError, format!("Assignment to constant variable"), 0, 0));
    }
    self.value = value.clone();
    let name = match &self.name {
        Some(name) => name.clone(),
        _ => String::from(""),
    };
    if let Some(reference) = &self.reference {
      match reference {
          Value::Scope(scope) => {
            let scope_rc = scope.upgrade();
            if let Some(scope)= scope_rc {
              scope.borrow_mut().set_value( name.clone(), value, false);
            }
            Ok(None)
          },
          _ => {
            let object = reference.to_object(ctx);
            object.borrow_mut().define_property( name.clone(), Property {
              enumerable: false,
              value: value,
            });
            Ok(None)
          }
      }
    } else {
      // no reference set value
      if name.len() > 0 {
        ctx.cur_scope.borrow_mut().set_value(name.clone(), value, self.is_const);
      }
      return Ok(Some(name.clone()))
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
  // 4 种引用类型
  Object(Rc<RefCell<Object>>),
  Function(Rc<RefCell<Object>>),
  Array(Rc<RefCell<Object>>),
  Promise(Rc<RefCell<Object>>),
  // 3 种基础数据类型的包装对象
  StringObj(Rc<RefCell<Object>>),
  NumberObj(Rc<RefCell<Object>>),
  BooleanObj(Rc<RefCell<Object>>),
  // 其他
  NAN,
  // 一般是指向自己等情况
  RefObject(Weak<RefCell<Object>>),
  Scope(Weak<RefCell<Scope>>),
  // 中断
  Interrupt(Token,Expression),
  // bytecode
  ByteCode(Vec<ByteCode>),
}

#[derive(PartialEq, Debug)]
pub enum ValueType {
  // 5种基本数据类型
  String,
  Number,
  Boolean,
  Null,
  Undefined,
  // 4 种引用类型
  Object,
  Function,
  Array,
  Promise,
  // 其他
  NAN,
}

impl PartialEq for Value {
  // 仅做简单比较，不能处理 Number(1.2) == 1.2 这种，需要通过 value.is_equal_to
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
      Value::Array(rc_value) => {
        Value::Array(Rc::clone(rc_value))
      },
      Value::Function(rc_value) => {
        Value::Function(Rc::clone(rc_value))
      },
      Value::Promise(rc_value) => {
        Value::Promise(Rc::clone(rc_value))
      },
      Value::StringObj(rc_value) => {
        Value::StringObj(Rc::clone(rc_value))
      },
      Value::NumberObj(rc_value) => {
        Value::NumberObj(Rc::clone(rc_value))
      },
      Value::BooleanObj(rc_value) => {
        Value::BooleanObj(Rc::clone(rc_value))
      },
      Value::String(str) => Value::String(str.clone()),
      Value::Number(num) => Value::Number(*num),
      Value::Boolean(bool) => Value::Boolean(*bool),
      Value::Null => Value::Null,
      Value::Undefined => Value::Undefined,
      Value::RefObject(obj) => {
        return Value::RefObject(obj.clone());
      },
      Value::Scope(obj) => {
        return Value::Scope(obj.clone());
      },
      Value::Interrupt(token, expr) => {
        return Value::Interrupt(token.clone(), expr.clone());
      },
      Value::ByteCode(bytecode) => {
        return Value::ByteCode(bytecode.clone());
      },
      _ => Value::Undefined,
    }
  }
}

impl Value {
  pub fn is_string(&self) -> bool {
    if let Value::String(_) = self {
      return true
    }
    if let Value::StringObj(_) = self {
      return true
    }
    return false
  }
  
  pub fn is_number(&self) -> bool {
    if let Value::Number(_) = self {
      return true
    }
    if let Value::NumberObj(_) = self {
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

  pub fn to_string(&self, ctx: &mut Context) -> String {
    
    let mut self_value = self;
    let primitive_value = self.to_primitive_value(ctx);
    if let Some(value) = &primitive_value {
      self_value = value;
    }
    match self_value {
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
      _ => {
        let call_this = match self {
          Value::Object(_) | Value::Array(_) | Value::Function(_) | Value::Promise(_) => Some(self.clone()),
          Value::RefObject(_ref) => {
            let origin = _ref.upgrade();
            if let Some(obj)= &origin {
              let obj = Rc::clone(obj);
              Some(Value::Object(obj))
            } else {
              None
            }
          }
          _=> None
        };
        if let Some(this) = call_this {
          let call_ctx = &mut CallContext {
            ctx,
            this,
            reference: None,
            func_name: String::from("toString"),
          };
          let value = Object::call(call_ctx, String::from("toString"), vec![]);
          if let Ok(value) = value {
            return value.to_string(ctx)
          }
          let value = Object::call(call_ctx, String::from("valueOf"), vec![]);
          if let Ok(value) = value {
            return value.to_string(ctx)
          }
        }
        return String::from("");
      },
    }
  }

  pub fn to_number(&self, ctx: &mut Context) -> Option<f64> {
    let mut self_value = self;
    let primitive_value = self.to_primitive_value(ctx);
    if let Some(value) = &primitive_value {
      self_value = value;
    }
    match self_value {
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
  pub fn to_boolean(&self, ctx: &mut Context) -> bool {
    let mut self_value = self;
    let primitive_value = self.to_primitive_value(ctx);
    if let Some(value) = &primitive_value {
      self_value = value;
    }
    match self_value {
        Value::Undefined | Value::Null => false,
        Value::String(str) => {
          return str.len() > 0;
        },
        Value::Number(num) => {
          return num.to_owned() != 0f64;
        },
        Value::Boolean(boolean) => {
          return boolean.to_owned();
        },
        _ => {
          true
        }
    }
  }

  // 实例化对象
  pub fn instantiate_object(&self, ctx: &mut Context, args: Vec<Value>) -> JSIResult<Value> {
    let rc_obj = self.to_weak_rc_object();
    if let Some(wrc) = rc_obj {
      let rc = wrc.upgrade();
      if let Some(obj)= &rc {
        let obj = Rc::clone(obj);
        let create_method = obj.borrow().get_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string());
        if let Some(create_method) = &create_method {
          if let Value::Function(function_define) = create_method {
            // 获取 function 定义
            let func_name = get_builtin_function_name(ctx, function_define);
            let fun_clone = Rc::clone(function_define);
            let fun_obj = (*fun_clone).borrow_mut();
            let function_define_value = fun_obj.get_initializer().unwrap();
            // 内置方法
            if let Statement::BuiltinFunction(builtin_function) = function_define_value.as_ref() {
              let mut call_ctx = CallContext{
                ctx,
                this: Value::Function(Rc::clone(function_define)),
                reference: None,
                func_name,
              };
              return (builtin_function)(&mut call_ctx, args);
            }
          }
        }
      }
    }
    Err(JSIError::new(JSIErrorType::Unknown, format!("todo: unsupported global Type"), 0, 0))
  }

  pub fn to_object(&self, ctx: &mut Context) -> Rc<RefCell<Object>> {
    // TODO: Cannot convert undefined or null to object
    let obj_value = self.to_object_value(ctx);
    match obj_value {
      Value::StringObj(obj) => {
        return obj;
      },
      Value::NumberObj(obj) => {
        return obj;
      },
      Value::BooleanObj(obj) => {
        return obj;
      },
      _ => {
        let rc_obj = self.to_weak_rc_object();
        if let Some(wrc) = rc_obj {
          let rc = wrc.upgrade();
          if let Some(obj)= &rc {
            return Rc::clone(obj);
          }
        }
        Rc::new(RefCell::new(Object::new(ClassType::Object,None)))
      }
    }
    
  }

  pub fn to_object_value(&self, ctx: &mut Context) -> Value {
    match self {
      Value::String(string) => {
        create_string(ctx, Value::String(string.to_owned()))
      },
      Value::Number(number) => {
        create_number(ctx, Value::Number(number.to_owned()))
      },
      Value::Boolean(boolean) => {
        create_boolean(ctx, Value::Boolean(boolean.to_owned()))
      },
      _ => {
        self.clone()
      }
    }
  }

  pub fn is_primitive_value(&self) -> bool {
    match self {
      Value::String(_) => true,
      Value::StringObj(_) => true,
      Value::Number(_) => true,
      Value::NumberObj(_) => true,
      Value::Boolean(_) => true,
      Value::BooleanObj(_) => true,
      Value::Undefined => true,
      Value::Null => true,
      _ => false
    }
  }

  // check value is null of undefined
  pub fn is_not_strict_null(&self) -> bool {
    match self {
      Value::Undefined => true,
      Value::Null => true,
      _ => false
    }
  }

   // check value is Object
   pub fn is_object(&self) -> bool {
    match self {
      Value::StringObj(_) => true,
      Value::NumberObj(_) => true,
      Value::BooleanObj(_) => true,
      Value::Function(_) => true,
      Value::Array(_) => true,
      Value::Object(_) => true,
      Value::Promise(_) => true,
      _ => false
    }
  }

  // 到原始值，也就是 Boolean(false) => false 等
  pub fn to_primitive_value(&self, ctx: &mut Context) -> Option<Value> {
    let base_type_obj: Option<(ValueType, &Rc<RefCell<Object>>)> = match self {
      Value::StringObj(obj) => Some((ValueType::String,obj)),
      Value::NumberObj(obj) => Some((ValueType::Number,obj)),
      Value::BooleanObj(obj) => Some((ValueType::Boolean,obj)),
      _ => None
    };

    if let Some(type_obj) = base_type_obj {
      let mut call_ctx = CallContext{
        ctx,
        this: self.clone(),
        reference: None,
        func_name: String::from("valueOf"),
      };
      // 不会出错
      let value = Object::call(&mut call_ctx, String::from("valueOf"), vec![]).unwrap();
      match type_obj.0 {
        ValueType::Number => {
          return Some(Value::Number(value.to_number(ctx).unwrap()));
        },
        ValueType::Boolean => {
          return Some(Value::Boolean(value.to_boolean(ctx)));
        },
        ValueType::String => {
          return Some(Value::String(value.to_string(ctx)));
        },
        _ => {}
      }
    }

    return None;
  }

  pub fn to_value_info(&self) -> ValueInfo {
    return ValueInfo { name: None, value: self.clone(), access_path: String::from(""), reference: None, is_const: false }
  }


  pub fn to_weak_rc_object(&self) -> Option<Weak<RefCell<Object>>> {
    match self {
      Value::Object(obj) => Some(Rc::downgrade(obj)),
      Value::Function(function) => Some(Rc::downgrade(function)),
      Value::Array(array) => Some(Rc::downgrade(array)),
      Value::Promise(promise) => Some(Rc::downgrade(promise)),
      Value::StringObj(obj) => Some(Rc::downgrade(obj)),
      Value::NumberObj(obj) => Some(Rc::downgrade(obj)),
      Value::BooleanObj(obj) => Some(Rc::downgrade(obj)),
      Value::RefObject(obj) => Some(obj.clone()),
      _ => None
    }
  }

  pub fn type_of(&self) -> String {
    match self {
      Value::Boolean(_) => String::from("boolean"),
      Value::Number(_) => String::from("number"),
      Value::NAN => String::from("number"),
      Value::String(_) => String::from("string"),
      Value::Undefined => String::from("undefined"),
      Value::Function(_) => String::from("function"),
      _ => String::from("object")
    }
  }

  fn get_value_type(&self) -> ValueType {
    match self {
      Value::Object(_) => ValueType::Object,
      Value::Function(_) => ValueType::Function,
      Value::Array(_) => ValueType::Array,
      Value::Promise(_) => ValueType::Object,
      Value::String(_) => ValueType::String,
      Value::StringObj(_) => ValueType::String,
      Value::Number(_) => ValueType::Number,
      Value::NumberObj(_) => ValueType::Number,
      Value::Boolean(_) => ValueType::Boolean,
      Value::BooleanObj(_) => ValueType::Boolean,
      Value::Null => ValueType::Null,
      Value::Undefined => ValueType::Undefined,
      Value::RefObject(refobj) => {
        let origin = refobj.upgrade();
        if let Some(origin) = &origin {
          let origin_clone = Rc::clone(&origin);
          let origin_borrow = origin_clone.borrow();
          return match &origin_borrow.class_type {
            // TODO: more type
            ClassType::Function => ValueType::Function,
            _ => ValueType::Object
          }
        }
        ValueType::Object
      },
      _ => {
        // TODO: more
        ValueType::NAN
      },
    }
  }

  pub fn is_equal_to(&self, ctx: &mut Context, other: &Value, is_check_type: bool) -> bool {
    let self_type = self.get_value_type();
    let other_type = other.get_value_type();
    let is_same_type = self_type == other_type;
    if is_check_type && !is_same_type {
      return false;
    }
    let mut self_value = self;
    let primitive_value = self.to_primitive_value(ctx);
    if let Some(value) = &primitive_value {
      self_value = value;
    }

    let mut other_value = other;
    let other_primitive_value = other.to_primitive_value(ctx);
    if let Some(value) = &other_primitive_value {
      other_value = value;
    }
    match (self_value, other_value) {
        (Value::String(a), Value::String(b)) => *a == *b,
        (Value::Number(a), Value::Number(b)) => *a == *b,
        (Value::Boolean(a), Value::Boolean(b)) => *a == *b,
        (Value::Null, Value::Null) | (Value::Undefined, Value::Undefined) => true,
        _ => {
          if self_value.is_primitive_value() && other_value.is_primitive_value() {
            return self_value.to_number(ctx) == other_value.to_number(ctx);
          }
          let left_obj_id = self_value.to_object(ctx).borrow().get_id();
          let right_obj_id: Rc<RefCell<Object>> = other_value.to_object(ctx);
          
          return left_obj_id == right_obj_id.borrow().get_id();
        },
    }
  }

  // 匿名方法，需要绑定name
  pub fn bind_name(&mut self, name: String) {
    match self {
      Value::Function(function) => {
        let mut function_define =function.borrow_mut();
        let mut value = function_define.get_initializer().unwrap();
        match *value {
            Statement::Function(func) => {
              if func.is_anonymous {
                let mut new_func = func.clone();
                new_func.name = IdentifierLiteral {
                  literal: name.clone()
                };
                value = Box::new(Statement::Function(new_func));
                function_define.set_value(Some(value));
                function_define.define_property(String::from("name"), Property { enumerable: false, value: Value::String(String::from(name)) });
              }
            },
            _ => {}
        };
      },
      _ => {}
    }
  }

}

pub struct CallStatementOptions {
  pub label: Option<String>,
}

pub const INSTANTIATE_OBJECT_METHOD_NAME: &str = "instantiate_object_method";