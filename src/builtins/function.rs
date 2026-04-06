use std::{rc::{Rc, Weak}, cell::RefCell};
use crate::{ast_node::{BlockStatement, IdentifierLiteral, Parameter}, bytecode::ByteCode, constants::{GLOBAL_FUNCTION_NAME, PROTO_PROPERTY_NAME}, context::Context, error::{JSIError, JSIErrorType}};
use crate::{ast_node::{Statement, FunctionDeclaration, BuiltinFunction, ClassType, CallContext}, value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, scope::Scope, error::JSIResult};

use super::{object::{create_object, Property, Object}, global::{get_global_object_prototype_by_name, get_global_object_by_name}, array::create_list_from_array_list};

// 初始化一个方法
// ref: https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#prod-FunctionDeclaration
 pub fn create_function(ctx: &mut Context, function_declaration: &FunctionDeclaration, define_scope: Rc<RefCell<Scope>>) -> Value {
  let global_function = get_global_object_by_name(ctx, GLOBAL_FUNCTION_NAME);
  let function = create_object(ctx,ClassType::Function, Some(Box::new(Statement::Function((*function_declaration).clone()))));
  
  let function_clone = Rc::clone(&function);
  let mut function_mut = (*function_clone).borrow_mut();
  // 绑定 fun.constructor = global.Function
  function_mut.constructor = Some(Rc::downgrade(&global_function));
  // fun.name
  function_mut.define_property(String::from("name"), Property {
    enumerable: false,
    value: Value::String(function_declaration.name.literal.clone()),
  });
  // fun.length
  function_mut.define_property(String::from("length"), Property {
    enumerable: false,
    value: Value::Number(function_declaration.parameters.len() as f64)
  });
  
  let global_prototype = get_global_object_prototype_by_name(ctx, GLOBAL_FUNCTION_NAME);
  function_mut.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&global_prototype)));

  // define_scope
  function_mut.set_inner_property_value(String::from("define_scope"), Value::Scope(define_scope));
  function_mut.set_inner_property_value(String::from("bytecode"), Value::ByteCode(function_declaration.bytecode.clone()));

  // function prototype
  let prototype =  Rc::new(RefCell::new(Object::new(ClassType::Object, None)));
  let prototype_clone = Rc::clone(&prototype);
  let mut prototype_mut = prototype_clone.borrow_mut();
  // function.prototype.constructor 指向自己
  prototype_mut.define_property(String::from("constructor"), Property {
    enumerable: false,
    value: Value::RefObject(Rc::downgrade(&function)),
  });
  function_mut.prototype = Some(prototype);
  Value::Function(function)
}


pub fn create_function_with_bytecode(ctx: &mut Context, name: String, parameters: Vec<Parameter>, bytecode: Vec<ByteCode>, define_scope: Rc<RefCell<Scope>>) -> Value {
  let function_declaration = FunctionDeclaration {
    is_anonymous: false,
    is_arrow: false,
    is_async: false,
    name: IdentifierLiteral {
      literal: name,
    },
    parameters,
    body: BlockStatement {
      statements: vec![],
    },
    declarations: vec![],
    bytecode
  };
  create_function(ctx, &function_declaration, define_scope)
}

// 构建内置方法
pub fn builtin_function(ctx: &mut Context, name: String, length: f64, fun: BuiltinFunction) -> Value {
  let global_function = get_global_object_by_name(ctx, GLOBAL_FUNCTION_NAME);
  let function = create_object(ctx, ClassType::Function, Some(Box::new(Statement::BuiltinFunction(fun))));
  let function_clone = Rc::clone(&function);
  let mut function_mut = (*function_clone).borrow_mut();
  // 绑定 fun.constructor = global.Function
  function_mut.constructor = Some(Rc::downgrade(&global_function));
  // fun.name
  function_mut.define_property(String::from("name"), Property {
    enumerable: false,
    value: Value::String(name),
  });
  // fun.length
  function_mut.define_property(String::from("length"), Property {
    enumerable: false,
    value: Value::Number(length)
  });

  let global_prototype = get_global_object_prototype_by_name(ctx, GLOBAL_FUNCTION_NAME);
  function_mut.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&global_prototype)));
  Value::Function(function)
}


pub fn bind_global_function(ctx: &mut Context) {

  // 绑定实例化方法
  let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, function_create);

  let apply_fun = builtin_function(ctx, String::from("apply"), 1f64, function_apply);
  let bind_fun = builtin_function(ctx, String::from("bind"), 1f64, function_bind);
  let call_fun = builtin_function(ctx,  String::from("call"), 1f64, function_call);
  let fun_rc = get_global_object_by_name(ctx, GLOBAL_FUNCTION_NAME);
  let mut fun = (*fun_rc).borrow_mut();
  fun.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
  if let Some(prop)= &fun.prototype {
    let prototype_rc = Rc::clone(prop);
    let mut prototype = prototype_rc.borrow_mut();
    // let name = String::from("toString");
    // prototype.define_property(name.clone(), Property { enumerable: true, value: builtin_function(global, name, 0f64, function_to_string) });
    let name = String::from("apply");
    prototype.define_property(name.clone(), Property { enumerable: true, value: apply_fun});
    let name = String::from("call");
    prototype.define_property(name.clone(), Property { enumerable: true, value: call_fun});
    let name = String::from("bind");
    prototype.define_property(name.clone(), Property { enumerable: true, value: bind_fun});
  }
}

// Function constructor: new Function(arg1, arg2, ..., argN, functionBody)
fn function_create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  // Last argument is the function body, rest are parameter names
  if args.is_empty() {
    // Create an empty function
    let function_declaration = FunctionDeclaration {
      is_anonymous: true,
      is_arrow: false,
      is_async: false,
      name: IdentifierLiteral {
        literal: String::from("anonymous"),
      },
      parameters: vec![],
      body: BlockStatement {
        statements: vec![],
      },
      declarations: vec![],
      bytecode: vec![]
    };
    return Ok(create_function(call_ctx.ctx, &function_declaration, Rc::clone(&call_ctx.ctx.cur_scope)));
  }

  // Extract parameter names and body
  let body = args[args.len() - 1].to_string(call_ctx.ctx);
  let param_names: Vec<String> = if args.len() > 1 {
    args[0..args.len() - 1].iter().map(|arg| arg.to_string(call_ctx.ctx)).collect()
  } else {
    vec![]
  };

  // Construct function string
  let params_str = param_names.join(", ");
  let function_str = format!("function anonymous({}) {{ {} }}", params_str, body);

  // Parse and compile the function
  let program = call_ctx.ctx.parse(function_str)?;

  // Find the function declaration in the program
  for statement in &program.body {
    if let Statement::Function(func_decl) = statement {
      return Ok(create_function(call_ctx.ctx, &func_decl, Rc::clone(&call_ctx.ctx.cur_scope)));
    }
  }

  Err(JSIError::new(JSIErrorType::SyntaxError, String::from("Failed to create function"), 0, 0))
}

// Function.prototype.apply
fn function_apply(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut this = Value::Undefined;
  let mut new_args: Vec<Value> = vec![];
  if args.len() > 0 {
    this = args[0].clone();
    if args.len() > 1 {
      new_args = create_list_from_array_list(call_ctx, &args[1])?;
    }
  }
  let new_fun = function_bind(call_ctx, vec![this])?;
  let call_function_define = match new_fun {
    Value::Function(function) => Some(function),
    _ => None,
  }.unwrap();
  return call_ctx.call_function(call_function_define, None, None, new_args);
}

// Function.prototype.call
fn function_call(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut this = Value::Undefined;
  let mut new_args: Vec<Value> = vec![];
  if args.len() > 0 {
    this = args[0].clone();
    new_args = args[1..].to_vec();
  }
  let new_fun = function_bind(call_ctx, vec![this])?;
  let call_function_define = match new_fun {
    Value::Function(function) => Some(function),
    _ => None,
  }.unwrap();
  return call_ctx.call_function(call_function_define, None, None, new_args);
}

// Function.prototype.bind
fn function_bind(ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut this = Value::Undefined;
  if args.len() > 0 {
    this = args[0].clone();
  }
  if let Value::Function(function_object) = &ctx.this {
    let fun_obj = function_object.borrow();
    let mut new_fun = fun_obj.force_copy();
    new_fun.set_inner_property_value(String::from("this"), this);
    Ok(Value::Function(Rc::new(RefCell::new(new_fun))))
  } else {
    Err(JSIError::new(JSIErrorType::TypeError, format!("Bind must be called on a function
    "), 0, 0))
  }
}

pub fn get_function_this(ctx: &mut Context, func: Rc<RefCell<Object>>)-> Rc<RefCell<Object>> {
  let bind_this = func.borrow().get_inner_property_value(String::from("this"));
  if let Some(this) = bind_this {
    return this.to_object(ctx)
  }
  return func
}

pub fn get_builtin_function_name(ctx: &mut Context, function_define: &Rc<RefCell<Object>>) -> String {
  let fun_clone = Rc::clone(function_define);
    let fun_obj = (*fun_clone).borrow_mut();
    let name = fun_obj.get_property_value(String::from("name"));
    if let Value::String(name) = name {
      return name;
    }
    return String::from("")
}