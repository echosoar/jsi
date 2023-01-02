use crate::{ast::Program, ast_node::Statement, ast_node::{Expression, CallExpression, Keywords, BinaryExpression}, value::Value, scope::Scope, ast_token::Token};

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
    pub fn run(&mut self, code: String) {
      let mut ast = AST::new(code);
      let program = ast.parse();
      self.call(program);
    }

    fn call(&mut self, program: Program) {
      // 创建全局作用域
      // TODO：绑定函数声明
      // 函数声明需要添加 prototype、constrctor、__proto__
      // 绑定变量声明
      // 执行 statement
      for statement in program.body.iter() {
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
          }
          _ => {}
        }
      }
      // 关闭全局作用域
    }

    fn execute_expression(&self, expression: &Expression) -> Value {
      println!("expression: {:?}", expression);
      match expression {
        Expression::Binary(binary) => {
          self.execute_binary_expression(binary)
        },
        Expression::Call(call) => {
          self.execute_call_expression(call)
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
    fn execute_binary_expression(&self, expression: &BinaryExpression) -> Value {
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
    fn execute_call_expression(&self, expression: &CallExpression) -> Value {
      let callee = self.execute_expression(expression.expression.as_ref());
      let mut arguments: Vec<Value> = vec![];
      for arg in expression.arguments.iter() {
        arguments.push(self.execute_expression(arg));
      }
      println!("call {:?} args:{:?}", callee, arguments);

      Value::Undefined
    }

    fn new_function(&self) {
      /*
      let function = Object {}

      */
      // function_value.defineProperty("name", expression.name)
      // function_value.defineProperty("length", expression.arguments.length)
      // function_value.defineProperty("prototype")
      // function_value.defineProperty("constructor")
      // function_value.defineOwnProperty("caller", )
      // function_value._prototype = global.FunctionPrototype = { constructor: ''};
    }
} 