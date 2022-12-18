use std::str::Chars;

// AST
pub struct AST<'a> {
  pub char: char,
  pub offset: usize,
  pub code: Chars<'a>,
  pub length: usize, 
}

impl<'a> AST<'a> {
  pub fn new(code: String) -> AST<'a> {
    let chars = code.chars();
    let len = chars.count();
    AST {
      char: ' ',
      offset: 0,
      code: chars,
      length: len,
    }
  }

  // 解析生成 Program
  pub fn parse(&self) -> Program {
    self.next();
    Program {  }
  }

  // 获取下一个符号
  fn next(&self) {
    self.scan()
  }

  // 扫描获取符号
  pub fn scan(&self) {

    fn isx(ch: char) -> bool {
      if ch == ' ' {
        return true
      }
      return false
    }
    loop {
      
    }
  }

  // 读取下一个字符
  pub fn read(&mut self) {
    if self.offset < self.length {
      self.offset = self.offset + 1;
      self.char = self.code.nth(self.offset).unwrap();
    } else {
      self.offset = self.length
    }
  }

  // 查看是否是 标识符
  pub fn charIsIdentifier() {

  }
}
#[derive(Debug)]
pub struct Program {}


impl Program {}