pub mod context;
pub mod ast;
pub mod ast_token;
pub mod ast_node;
pub mod ast_utils;
pub mod value;
pub mod scope;
pub mod error;
pub mod builtins;
pub mod constants;
pub mod bytecode;

use ast::Program;
use context::Context;
use error::JSIResult;
use value::Value;
pub struct JSI {
  context: Context,
}

impl JSI {
  pub fn new() -> JSI {
      let context = Context::new();
      JSI {
          context,
      }
  }

  pub fn set_strict(&mut self,strict: bool) {
    self.context.set_strict(strict);
  }

  pub fn run(&mut self, code: String) -> JSIResult<Value> {
      return self.context.run(code)
  }

  pub fn parse(&mut self, code: String) -> JSIResult<Program> {
    return self.context.parse(code)
  }
}
