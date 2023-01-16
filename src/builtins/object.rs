use std::borrow::BorrowMut;
use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::{Rc};
use super::array::create_array;
// use super::array::new_array;
use super::function::builtin_function;
use super::global::Global;
use crate::ast_node::{Statement, CallContext};
use crate::value::Value;
#[derive(Debug,Clone)]
// 对象
pub struct Object {
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
  pub constructor: Option<Rc<RefCell<Object>>>,
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
      constructor: None,
      value,
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

  // 从当前属性和原型链上面寻找值
  pub fn get_value(&self, name: String) -> Value {
    let prop = self.property.get(&name);
    if let Some(prop) = prop {
      return prop.value.clone()
    } else {
      if let Some(constructor) = &self.constructor {
        let mut cur = Rc::clone(constructor);
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

  // // 定义原型链上面的属性
  // pub fn define_prototype_property(&mut self, name: String, property: Property) -> bool {
  //   // let proto = self.prototype.as_mut().unwrap();
  //   // proto.define_property(name, property)
  // }

  // // [静态]调用内置方法
  // pub fn call_builtin(method_name: String, args: Vec<Value>, ctx: &mut CallContext) -> Value {
  //   println!("method_name {:?}", method_name);
  //   let this =  ctx.this.upgrade().unwrap();
  //   let this = this.borrow_mut();
  //   if let Some(value) = this.inner_property.get(&method_name) {
  //     if let Value::Function(fun) = &value.value {
  //       let fun_value = fun.borrow().get_value();
  //       if let Some(fun_value) = fun_value {
  //         if let Statement::BuiltinFunction(fun_value) = *fun_value {
  //           return (fun_value)(ctx, args);
  //         }
  //       }
  //     }
  //   }
  //   Value::Undefined
  // }

  // // 转换为字符串
  // pub fn to_string(&self, ctx: &mut CallContext) -> String {
  //   let value = Object::call_builtin(String::from("to_string"), vec![], ctx);
  //   if let Value::String(str) = value {
  //     return str;
  //   }
  //   String::from("")
  // }
}

#[derive(Debug,Clone,PartialEq)]
pub struct Property {
  // 是否可枚举
  pub enumerable: bool,
  pub value: Value,
  // TODO: 属性的描述符 descriptor writable ，是否可枚举等
}


// 实例化对象
pub fn create_object(global: &Global, value: Option<Box<Statement>>) -> Rc<RefCell<Object>> {
  let object = Rc::new(RefCell::new(Object::new(value)));
  let object_clone = Rc::clone(&object);
  let mut object_mut = (*object_clone).borrow_mut();
  // 绑定 obj.constructor = global.Object
  object_mut.constructor = Some(Rc::clone(&global.object));
  object
}



// Object.keys()
fn object_keys(ctx: &mut CallContext, args: Vec<Value>) -> Value {
  let array = create_array(ctx.global, 0);
  let array_obj = match array {
    Value::Array(arr) => Some(arr),
    _ => None
  }.unwrap();

  if args.len() < 1 {
    return Value::Array(array_obj);
  }
  let weak = Rc::downgrade(&array_obj);
  let call_ctx = &mut CallContext {
    global: &ctx.global,
    this: weak,
  };
  let obj_rc= args[0].to_object();
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


pub fn bind_global_object(global: &Global) {
  let mut obj = (*global.object).borrow_mut();
 let property = obj.property.borrow_mut();
 let name = String::from("keys");
 property.insert(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 1f64, object_keys) });
}