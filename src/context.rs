use std::{rc::{Rc}, cell::RefCell};

use crate::{ast::Program, ast_node::{Statement, Declaration, ObjectLiteral, AssignExpression, CallContext}, ast_node::{Expression, CallExpression, Keywords, BinaryExpression}, value::{Value, ValueInfo}, scope::{Scope, get_value_and_scope}, ast_token::Token, builtins::{object::{Object, Property, create_object}, function::{create_function}, global::Global}};

use super::ast::AST;
pub struct Context {
  scope: Rc<RefCell<Scope>>,
  global: Global,
  cur_scope: Rc<RefCell<Scope>>
}

impl Context {
    pub fn new() -> Context {
      let scope = Rc::new(RefCell::new(Scope::new()));
      let cur_scope = Rc::clone(&scope);
      let global = Global::new();
      let mut ctx = Context {
        global,
        scope,
        cur_scope,
      };
      ctx.init();
      return ctx;
    }
    
    // 运行一段 JS 代码
    pub fn run(&mut self, code: String) -> Value {
      let program = self.parse(code);
      self.call(program)
    }

    // 运行一段 JS 代码
    pub fn parse(&mut self, code: String) -> Program {
      let mut ast = AST::new(code);
      ast.parse()
    }

    fn call(&mut self, program: Program) -> Value {
      self.call_block(&program.declarations, &program.body).1
    }

    fn call_block(&mut self, declarations: &Vec<Declaration>, body: &Vec<Statement>) -> (Value, Value) {
       // 绑定函数声明
       for declaration in declarations.iter() {
        match  declaration {
            Declaration::Function(function_statement) => {
              let function = create_function(&self.global, &function_statement);
             (*self.scope).borrow_mut().set_value(function_statement.name.literal.clone(), function)
            }
        };
      }
      // 函数声明需要添加 prototype、constrctor、__proto__
      // 绑定变量声明
      // 执行 statement
      let mut result_value = Value::Undefined;
      let mut last_statement_value = Value::Undefined;
      for statement in body.iter() {
        match statement {
          Statement::Var(var_statement) => {
            for variable in var_statement.list.iter() {
              if let Expression::Var(let_var) = variable {
                let name = let_var.name.clone();
                let mut value = self.execute_expression(&let_var.initializer);
                value.bind_name(name.clone());
                last_statement_value = value.clone();
                (*self.scope).borrow_mut().set_value(name, value);
              }
            }
          },
          Statement::Expression(expression) => {
            last_statement_value = self.execute_expression(&expression.expression);
          },
          Statement::Return(return_statement) => {
            result_value = self.execute_expression(&return_statement.expression);
            last_statement_value = result_value.clone()
          },
          Statement::Function(_) => {
            // skip, 因为函数声明前置了
          },
          _ => {
            println!("unknown statement {:?}", statement);
          }
        }
      }
      (result_value, last_statement_value)
    }

    fn execute_expression(&mut self, expression: &Expression) -> Value {
      self.execute_expression_info(expression).value
    }

    fn execute_expression_info(&mut self, expression: &Expression) -> ValueInfo {
      // println!("expression: {:?}", expression);
      match expression {
        Expression::Binary(binary) => {
          ValueInfo { value: self.execute_binary_expression(binary), name: None, reference: None }
        },
        Expression::Call(call) => {
          ValueInfo { value: self.execute_call_expression(call), name: None, reference: None }
        },
        Expression::Object(object) => {
          ValueInfo { value: self.new_object(object), name: None, reference: None }
        },
        Expression::Function(function_declaration) => {
          ValueInfo { value: create_function(&self.global, function_declaration), name: None, reference: None }
        },
        Expression::PropertyAccess(property_access) => {
          // expression.name
          let left = self.execute_expression(&property_access.expression);
          let left_obj = left.to_object();
          let right = &property_access.name.literal;
          let value = (*left_obj).borrow().get_value(right.clone());
          ValueInfo { value, name: Some(right.clone()), reference: Some(Value::Object(left_obj)) }
        },
        Expression::ElementAccess(element_access) => {
          // expression[argument]
          let left = self.execute_expression(&element_access.expression);
          let left_obj = left.to_object();
          let right = self.execute_expression(&element_access.argument).to_string();
          let value = (*left_obj).borrow().get_value(right.clone());
          ValueInfo { value, name: Some(right.clone()), reference: Some(Value::Object(left_obj)) }
        },
        Expression::Identifier(identifier) => {
          let name = identifier.literal.clone();
          let (value, scope) = get_value_and_scope(Rc::clone(&self.scope), name.clone());
          if let Some(val) = value {
            ValueInfo{ value: val, name: Some(name.clone()), reference: Some(Value::Scope(Rc::clone(&scope))) }
          } else {
            ValueInfo{ value: Value::Undefined, name: Some(name.clone()), reference: Some(Value::Scope(Rc::clone(&scope))) }
          }
        },
        Expression::Assign(assign) => {
          ValueInfo{ value: self.execute_assign_expression(assign), name: None, reference: None }
        },
        Expression::String(string) => {
          ValueInfo {value: Value::String(string.value.clone()), name: None, reference: None }
        },
        Expression::Number(number) => {
          return ValueInfo {value: Value::Number(number.value), name: None, reference: None }
        },
        Expression::Keyword(keyword) => {
          ValueInfo {
            value: match *keyword {
              Keywords::False => Value::Boolean(false),
              Keywords::True => Value::Boolean(true),
              Keywords::Null => Value::Null,
              _ => Value::Undefined,
            },
            name: None,
            reference: None,
          }
        },
        _ => {
          println!("expression: {:?}", expression);
          ValueInfo {
            value: Value::Undefined,
            name: None,
            reference: None,
          }
        },
      }
    }

    // 执行基础四则运算
    fn execute_binary_expression(&mut self, expression: &BinaryExpression) -> Value {
      let left = self.execute_expression(expression.left.as_ref());
      let right = self.execute_expression(expression.right.as_ref());
      match expression.operator {
        Token::Equal => {
          return Value::Boolean(left.is_equal_to(&right, false));
        },
        Token::StrictEqual => {
          return Value::Boolean(left.is_equal_to(&right, true));
        },
        Token::Plus | Token::Subtract | Token::Multiply | Token::Slash |Token::Remainder => {
          // 数字处理
          if left.is_nan() || right.is_nan() {
            return Value::NAN;
          }
          
          // 加法的特殊处理
          if expression.operator == Token::Plus {
            // 如果有一个是字符串，那就返回字符串
            if left.is_string() || right.is_string() {
              return Value::String(left.to_string() + right.to_string().as_str())
            }
          }

          // 除法的特殊处理
          if expression.operator == Token::Slash {
            if left.is_infinity() && right.is_infinity() {
              return Value::NAN;
            }
          }

          // 计算数字运算
          self.execute_number_operator_expression(&left, &right, &expression.operator)
        },
        _ =>  Value::Undefined
      }

    }

    // 执行方法调用表达式
    fn execute_number_operator_expression(&mut self, left: &Value, right: &Value, operator: &Token) -> Value {
      let left_number: f64;
      let right_number: f64;
      if let Some(num) = left.to_number() {
        left_number = num;
      } else {
        return Value::NAN;
      }
      if let Some(num) = right.to_number() {
        right_number = num;
      } else {
        return Value::NAN;
      }
      match operator {
        Token::Plus => Value::Number(left_number + right_number),
        Token::Subtract => Value::Number(left_number - right_number),
        Token::Multiply => Value::Number(left_number * right_number),
        Token::Slash => Value::Number(left_number / right_number),
        Token::Remainder => Value::Number(left_number % right_number),
        _=> Value::NAN,
      }
    }

    // 执行方法调用表达式
    fn execute_call_expression(&mut self, expression: &CallExpression) -> Value {
      let callee = self.execute_expression_info(expression.expression.as_ref());
      let mut arguments: Vec<Value> = vec![];
      for arg in expression.arguments.iter() {
        arguments.push(self.execute_expression(arg));
      }
      if let Value::Function(function_object) = callee.value {
        return self.call_function_object(function_object, callee.reference, arguments);
      }
      // TODO: throw error,非函数
      Value::Undefined
    }

    // 执行赋值表达式
    fn execute_assign_expression(&mut self, expression: &AssignExpression) -> Value {
      let mut left_info = self.execute_expression_info(&expression.left);
      let right_value = self.execute_expression(&expression.right);
      // TODO: more operator
      if expression.operator == Token::Assign {
        left_info.set_value(right_value.clone());
        right_value
      } else {
        Value::Undefined
      }
    }

    fn new_object(&mut self, expression: &ObjectLiteral) -> Value {
      // 获取 object 实例
      let object = create_object(&self.global, None);
      let object_clone = Rc::clone(&object);
      let mut object_mut = (*object_clone).borrow_mut();
      // TODO:
      // object.prototype = global.object_prototype;
      // 绑定属性
      for property_index in 0..expression.properties.len() {
        let property = &expression.properties[property_index];
        let name = self.execute_expression(&property.name).to_string();
        let mut initializer = self.execute_expression(&property.initializer);
        initializer.bind_name(name.clone());
        object_mut.define_property(name, Property {
          enumerable: true,
          value: initializer,
        });
      }
      Value::Object(object)
    }

    fn call_function_object(&mut self, function_define: Rc<RefCell<Object>>, call_this: Option<Value>, arguments: Vec<Value>) -> Value {
      // 获取 function 定义
      let function_define_value = (*function_define).borrow_mut().get_initializer().unwrap();
      // 内置方法
      if let Statement::BuiltinFunction(builtin_function) = *function_define_value {
        let mut this_obj = function_define;
        if let Some(call_this_value) = call_this {
          if let Value::Object(obj) = call_this_value {
            this_obj = obj;
          } else if let Value::Array(obj) = call_this_value {
            this_obj = obj;
          } else if let Value::Function(obj) = call_this_value {
            this_obj = obj;
          }
        }
        let mut ctx = CallContext{ global: &self.global, this: Rc::downgrade(&this_obj) };
        let result = (builtin_function)(&mut ctx, arguments);
        return result;
      }

      let function_declaration =  match *function_define_value {
        Statement::Function(function_declaration) => Some(function_declaration),
        _ => None,
      }.unwrap();
      // 创建新的作用域
      self.new_scope();
      // 绑定参数
      for parameter_index in 0..function_declaration.parameters.len() {
        if parameter_index < arguments.len() {
          // TODO: 参数引用
          (*self.scope).borrow_mut().set_value(function_declaration.parameters[parameter_index].name.literal.clone(), arguments[parameter_index].clone());
        } else {
          (*self.scope).borrow_mut().set_value(function_declaration.parameters[parameter_index].name.literal.clone(), Value::Undefined);
        }
      }
      // 执行 body
      let result = self.call_block(&function_declaration.declarations, &function_declaration.body.statements);
      self.close_scope();
      result.0
    }

    // 进入作用域
    fn new_scope(&mut self) {
      let mut new_scope = Scope::new();
      new_scope.parent = Some(Rc::clone(&self.cur_scope));
      let scope_rc = Rc::new(RefCell::new(new_scope));
      let rc = Rc::clone(&scope_rc);
      (*self.cur_scope).borrow_mut().childs.push(scope_rc);
      self.cur_scope = rc;
    }

    // 退出作用域
    fn close_scope(&mut self) {
      if let Some(parent_scope_rc) = &self.cur_scope.borrow().parent {
        let len = (*parent_scope_rc).borrow_mut().childs.len();
        let mut cur_scope_index = len;
        for index in 0..len {
          if (*parent_scope_rc).borrow_mut().childs[index].borrow().id == self.cur_scope.borrow().id {
            cur_scope_index = index;
          }
        }
        if cur_scope_index != len {
          (*parent_scope_rc).borrow_mut().childs.remove(cur_scope_index);
        }
      }
    }

    // 初始化，主要是挂载全局对象
    fn init(&mut self) {
      // 挂载全局对象
      let mut global_scope = self.scope.borrow_mut();
      global_scope.set_value(String::from("Object"), Value::Object(Rc::clone(&self.global.object)));
    }
} 