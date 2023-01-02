mod context;
mod ast;
mod ast_token;
mod ast_node;
mod ast_utils;
mod object;
mod value;
mod scope;

use context::Context;
pub struct JSI {
  context: Context,
}

impl JSI {
  pub fn new() -> JSI {
      JSI {
          context: Context::new()
      }
  }
  pub fn run(&mut self, code: String) {
      return self.context.run(code)
  }
}
