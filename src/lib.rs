pub mod context;
pub mod ast;
pub mod ast_token;
pub mod ast_node;
pub mod ast_utils;
pub mod value;
pub mod scope;
pub mod builtins;

use ast::Program;
use context::Context;
use value::Value;
pub struct JSI {
  context: Context,
}

impl JSI {
  pub fn new() -> JSI {
      JSI {
          context: Context::new()
      }
  }
  pub fn run(&mut self, code: String) -> Value {
      return self.context.run(code)
  }

  pub fn parse(&mut self, code: String) -> Program {
    return self.context.parse(code)
  }
}
