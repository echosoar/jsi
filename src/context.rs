use std::{rc::{Rc}, cell::RefCell};

use crate::{ast::Program, ast_node::{Statement, Declaration, ObjectLiteral, FunctionDeclaration}, ast_node::{Expression, CallExpression, Keywords, BinaryExpression}, value::Value, value::{Object}, scope::{Scope, get_value_by_scope}, ast_token::Token};

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
      self.call_block(&program.declarations, &program.body).1
    }

    fn call_block(&mut self, declarations: &Vec<Declaration>, body: &Vec<Statement>) -> (Value, Value) {
       // 绑定函数声明
       for declaration in declarations.iter() {
        match  declaration {
            Declaration::Function(function_statement) => {
              let function = self.new_function(&function_statement);
              self.scope.set_value(function_statement.name.literal.clone(), function)
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
          Statement::Let(let_statement) => {
            for variable in let_statement.list.iter() {
              if let Expression::Let(let_var) = variable {
                let name = let_var.name.clone();
                let mut value = self.execute_expression(&let_var.initializer);
                value.bind_name(name.clone());
                last_statement_value = value.clone();
                self.scope.set_value(name, value);
              }
            }
          },
          Statement::Expression(expression) => {
            last_statement_value = self.execute_expression(&expression.expression);
          },
          Statement::Return(return_statement) => {
            result_value = self.execute_expression(&return_statement.expression);
            last_statement_value = result_value.clone()
          }
          _ => {}
        }
      }
      (result_value, last_statement_value)
    }

    fn execute_expression(&mut self, expression: &Expression) -> Value {
      // println!("expression: {:?}", expression);
      match expression {
        Expression::Binary(binary) => {
          self.execute_binary_expression(binary)
        },
        Expression::Call(call) => {
          self.execute_call_expression(call)
        },
        Expression::Object(object) => {
          self.new_object(object)
        },
        Expression::Function(function_declaration) => {
          self.new_function(function_declaration)
        },
        Expression::PropertyAccess(property_access) => {
          // expression.name
          let left = self.execute_expression(&property_access.expression);
          let left_obj = left.to_object();
          let right = &property_access.name.literal;
          let value = (*left_obj).borrow().get_property(right.clone());
          value
        },
        Expression::ElementAccess(element_access) => {
          // expression[argument]
          let left = self.execute_expression(&element_access.expression);
          let left_obj = left.to_object();
          let right = self.execute_expression(&element_access.argument).to_string();
          let value = (*left_obj).borrow().get_property(right.clone());
          value
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
        _ => {
          println!("expression: {:?}", expression);
          Value::Undefined
        },
      }
    }

    // 执行基础四则运算
    fn execute_binary_expression(&mut self, expression: &BinaryExpression) -> Value {
      let left = self.execute_expression(expression.left.as_ref());
      let right = self.execute_expression(expression.right.as_ref());
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
      if expression.operator == Token::Subtract {
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
      if let Value::Function(function_object) = callee {
        return self.call_function_object(function_object, arguments);
      }

      Value::Undefined
    }
    // 基础对象，绑定好原型链
    fn new_base_object(&mut self) -> Rc<RefCell<Object>> {
      let object = Rc::new(RefCell::new(Object::new()));
      let object_clone = Rc::clone(&object);
      let mut object_mut = (*object_clone).borrow_mut();
      let prototype =  Rc::new(RefCell::new(Object::new()));
      // constructor 弱引用
      (*prototype).borrow_mut().define_property_by_value(String::from("constructor"), Value::RefObject(Rc::downgrade(&object)));
      object_mut.define_property_by_value(String::from("prototype"), Value::Object(prototype));
      object
    }
    fn new_object(&mut self, expression: &ObjectLiteral) -> Value {
      let object = self.new_base_object();
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
        object_mut.define_property_by_value(name, initializer);
      }
      Value::Object(object)
    }

    // ref: https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#prod-FunctionDeclaration
    fn new_function(&mut self, function_declaration: &FunctionDeclaration) -> Value {
      let function = self.new_base_object();
      let function_clone = Rc::clone(&function);
      let mut function_mut = (*function_clone).borrow_mut();
      function_mut.set_value(Some(Box::new(Statement::Function((*function_declaration).clone()))));
      // TODO: function.prototype = global.function_prototype;
      // TODO: function name https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-setfunctionname
      function_mut.define_property_by_value(String::from("name"),  Value::String(function_declaration.name.literal.clone()));
      function_mut.define_property_by_value(String::from("length"), Value::Number(function_declaration.parameters.len() as f64));

      Value::Function(function)
    }

    fn call_function_object(&mut self, function_define: Rc<RefCell<Object>>, arguments: Vec<Value>) -> Value {
      // 获取 function 定义
      let function_define_value = (*function_define).borrow_mut().get_value().unwrap();
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
          self.scope.set_value(function_declaration.parameters[parameter_index].name.literal.clone(), arguments[parameter_index].clone());
        } else {
          self.scope.set_value(function_declaration.parameters[parameter_index].name.literal.clone(), Value::Undefined);
        }
      }
      // 执行 body
      let result = self.call_block(&function_declaration.declarations, &function_declaration.body.statements);
      self.close_scope();
      result.0
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