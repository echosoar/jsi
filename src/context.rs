use crate::{ast::Program, ast_node::Statement, ast_node::{Expression}, value::Value, scope::Scope};

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
      // 绑定函数声明
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
          _ => {}
        }
      }
      // 关闭全局作用域
    }

    fn execute_expression(&self, expression: &Expression) -> Value {
      println!("expression: {:?}", expression);
      match expression {
        Expression::String(string) => {
          return Value::String(string.value.clone());
        },
        Expression::Number(number) => {
          return Value::Number(number.value);
        },
        _ => Value::Undefined,
      }
    }
} 