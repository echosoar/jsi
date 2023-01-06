use std::rc::{Rc};

use crate::{ast::Program, ast_node::{Statement, Declaration, FunctionDeclarationStatement}, ast_node::{Expression, CallExpression, Keywords, BinaryExpression}, value::Value, value::{Object}, scope::{Scope, get_value_by_scope}, ast_token::Token};

use super::ast::AST;
pub struct Context {
  scope: Scope,
}

impl Context {
    pub fn new() -> Context {
      let ctx = Context {
        scope: Scope::new(),
      };
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
      self.call_block(&program.declarations, &program.body)
    }

    fn call_block(&mut self, declarations: &Vec<Declaration>, body: &Vec<Statement>) -> Value {
       // 绑定函数声明
       for declaration in declarations.iter() {
        match  declaration {
            Declaration::Function(function_statement) => {
              let function = self.new_function(&function_statement);
              println!("function:{:?}", function);
              self.scope.set_value(function_statement.name.literal.clone(), function)
            }
        };
      }
      // 函数声明需要添加 prototype、constrctor、__proto__
      // 绑定变量声明
      // 执行 statement
      let mut result_value = Value::Undefined;
      for statement in body.iter() {
        match statement {
          Statement::Let(let_statement) => {
            for variable in let_statement.list.iter() {
              if let Expression::Let(let_var) = variable {
                let value = self.execute_expression(&let_var.initializer);
                self.scope.set_value(let_var.name.clone(), value);
              }
            }
          },
          Statement::Expression(expression) => {
            let value = self.execute_expression(&expression.expression);
            println!("expression result: {:?}", value);
          },
          Statement::Return(return_statement) => {
            result_value = self.execute_expression(&return_statement.expression);
          }
          _ => {}
        }
      }
      result_value
    }

    fn execute_expression(&mut self, expression: &Expression) -> Value {
      println!("expression: {:?}", expression);
      match expression {
        Expression::Binary(binary) => {
          self.execute_binary_expression(binary)
        },
        Expression::Call(call) => {
          self.execute_call_expression(call)
        },
        Expression::Identifier(identifier) => {
          if let Some(value) = get_value_by_scope(&self.scope, identifier.literal.clone()) {
            (*value).clone()
          } else {
            Value::Undefined
          }
        },
        Expression::String(string) => {
          return Value::String(string.value.clone());
        },
        Expression::Number(number) => {
          return Value::Number(number.value);
        },
        Expression::Keyword(keyword) => {
          match *keyword {
            Keywords::False => Value::Boolean(false),
            Keywords::True => Value::Boolean(true),
            Keywords::Null => Value::Null,
            _ => Value::Undefined,
          }
        },
        _ => Value::Undefined,
      }
    }

    // 执行基础四则运算
    fn execute_binary_expression(&mut self, expression: &BinaryExpression) -> Value {
      let left = self.execute_expression(expression.left.as_ref());
      let right = self.execute_expression(expression.right.as_ref());
      println!("binary {:?} {:?} {:?}", left, expression.operator, right);
      if left.is_nan() || right.is_nan() {
        return Value::NAN;
      }
      // 加法
      if expression.operator == Token::Plus {
        // 如果有一个是字符串，那就返回字符串
        if left.is_string() || right.is_string() {
          return Value::String(left.to_string() + right.to_string().as_str())
        }
        return Value::Number(left.to_number() + right.to_number())
      }
      // 减法
      if expression.operator == Token::Minus {
        return Value::Number(left.to_number() - right.to_number())
      }

      // 乘法
      if expression.operator == Token::Multiply {
        return Value::Number(left.to_number() * right.to_number())
      }
    
      // 除法
      if expression.operator == Token::Slash {
        if left.is_infinity() && right.is_infinity() {
          return Value::NAN;
        }
        let left_value = left.to_number();
        let right_value = right.to_number();
        return Value::Number(left_value / right_value)
      }

       // 取余
       if expression.operator == Token::Remainder {
        return Value::Number(left.to_number() % right.to_number())
      }

      Value::Undefined
    }

    // 执行方法调用表达式
    fn execute_call_expression(&mut self, expression: &CallExpression) -> Value {
      let callee = self.execute_expression(expression.expression.as_ref());
      let mut arguments: Vec<Value> = vec![];
      for arg in expression.arguments.iter() {
        arguments.push(self.execute_expression(arg));
      }
      if let Value::Object(function_object) = callee {
        return self.call_function_object(function_object, arguments);
      }

      Value::Undefined
    }

    fn new_function(&self, function_statement: &FunctionDeclarationStatement) -> Value {
      let mut function = Object::new();
      function.set_value(Some(Box::new(Value::Function((*function_statement).clone()))));
      // TODO:
      // function.prototype = global.function_prototype;
      function.define_property_by_value(String::from("name"),  Value::String(function_statement.name.literal.clone()));
      function.define_property_by_value(String::from("length"), Value::Number(function_statement.parameters.len() as f64));


      let prototype =  Rc::new(Object::new()); 
      // TODO: 定义循环结构，constructor 需要指向 function
      function.define_property_by_value(String::from("prototype"), Value::CycleRefObject(Rc::downgrade(&prototype)));
      
      Value::Object(function)
    }

    fn call_function_object(&mut self, function_define: Object, arguments: Vec<Value>) -> Value {
      // 获取 function 定义
      let function_define_value = *function_define.get_value().unwrap();
      let function_statement =  match function_define_value {
        Value::Function(function_statement) => Some(function_statement),
        _ => None,
      }.unwrap();
      // 创建新的作用域
      self.new_scope();
      // 绑定参数
      for parameter_index in 0..function_statement.parameters.len() {
        if parameter_index < arguments.len() {
          // TODO: 参数引用
          self.scope.set_value(function_statement.parameters[parameter_index].name.literal.clone(), arguments[parameter_index].clone());
        } else {
          self.scope.set_value(function_statement.parameters[parameter_index].name.literal.clone(), Value::Undefined);
        }
      }
      println!("call function {:?}", arguments);
      // 执行 body
      let result = self.call_block(&function_statement.declarations, &function_statement.body.statements);
      println!("call function result {:?}", result);
      self.close_scope();
      result
    }

    // 进入作用域
    fn new_scope(&mut self) {
      let mut scope = Scope::new();
      scope.parent = Some(Box::new(self.scope.clone()));
      self.scope = scope;
    }

    // 退出作用域
    fn close_scope(&mut self) {
      if let Some(parent) = self.scope.parent.clone() {
        self.scope = *parent
      }
    }
} 