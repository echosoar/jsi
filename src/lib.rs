mod context;
mod ast;
mod ast_token;
mod ast_node;
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
    pub fn run(&self, code: String) {
        return self.context.run(code)
    }
}