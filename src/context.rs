use super::ast::AST;
pub struct Context {
}

impl Context {
    pub fn new() -> Context {
      Context{}
    }
    // 运行一段 JS 代码
    pub fn run(&self, code: String) {
      let ast = AST::new(code);
      let program = ast.parse();

      println!("{:?}", program)
    }
}