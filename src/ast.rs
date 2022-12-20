// AST
// mod super::token::TokenKeywords;
use std::io;

use crate::{ast_token::{get_token_keyword, Token, get_token_literal}, ast_node::ASTNodeLetVariableExpression};
pub struct AST {
  // 当前字符
  char: char,
  // 下一个字符的索引
  index: usize,
  // 当前字符索引
  pos: usize,
  // 代码字符列表
  code: Vec<char>,
  // 代码总字符数
  length: usize,
  // 当前标识符
  token: Token,
  // 当前字面量
  literal: String,
}

impl AST{
  pub fn new(code: String) -> AST{
    let chars: Vec<char> = code.chars().collect();
    let len = chars.len();
    AST {
      char: ' ',
      index: 0,
      pos: 0,
      code: chars,
      length: len,
      token: Token::IDENTIFIER,
      literal: String::from(""),
    }
  }

  // 解析生成 Program
  pub fn parse(&mut self) -> Program {
    self.next();
    return self.parse_program()
  }

  // 解析生成 program
  fn parse_program(&mut self) -> Program {
    // 开启一个作用域
    self.open_scope();
    self.parse_statements();
    // 关闭一个作用域
    self.close_scope();
    Program {}
  }
  // 开启一个作用域
  fn open_scope(&mut self) {

  }

   // 关闭一个作用域
   fn close_scope(&mut self) {

   }

  fn parse_statements(&mut self) {
    loop {
      if let Token::EOF = self.token  {
        // end of file
      } else {
        self.parse_statement()
      }
    }
  }

  // 解析生成 statement
  fn parse_statement(&mut self) {
    match self.token {
        Token::LET => self.parse_let_statement(),
        _ => {},
    }
  }

  // 解析 let 
  fn parse_let_statement(&mut self) {
    self.check_token_and_next(Token::LET);
    loop {
      self.parse_variable_declaration();
    }
  }

  // 解析变量定义 a = 123,b,c = true 
  fn parse_variable_declaration(&mut self) {
    if Token::IDENTIFIER != self.token {
      // TODO: throw error 需要一个identifier
      return;
    }
    let literal = self.literal.clone();
    self.next();

    let node = ASTNodeLetVariableExpression{
      name: literal
    };

    if self.token == Token::ASSIGN {
      
      // 解析赋值语句
      self.next()
    }
    println!("literal{:?}", node);
    return

  }

  fn check_token_and_next(&mut self, token: Token) {
    if token == self.token {
      self.next()
    } else {
      self.error_unexpected_token(token)
    }
    // TODO: 类型不匹配，需要报错
  }

  // 获取下一个符号
  fn next(&mut self) {
    let scan_res = self.scan();
    self.token = scan_res.0;
    self.literal = scan_res.1;
    println!("scan:{:?}, {}, {}", self.token, self.literal, self.index)
  }

  // 扫描获取符号
  pub fn scan(&mut self) -> (Token, String) {
    // TODO: 严格模式
    let is_strict = true;
    loop {
      self.skip_white_space();
      if self.char_is_identifier_first() {
        let literal = self.get_identifier().unwrap();
        
        let token = get_token_keyword(&literal, is_strict);
        match token {
          Token::ILLEGAL => {
            let token_literal = get_token_literal(&literal);
            match token_literal {
              Token::ILLEGAL => {
                // 其他非字面量
                return (Token::IDENTIFIER, literal)
              },
              _ => {
                // 非字面量:null、true和false
                return (token_literal, literal)
              },
            }
          },
          _ => {
            // 关键字
            return (token, literal)
          },
        };
      }
      // 数字
      if self.char >= '0' && self.char <= '9' {
        return self.scan_number();
      }
      // TODO: more operator
      if self.char == '=' {
        return (Token::ASSIGN, String::from(self.char))
      }

      self.read();
      if self.index == self.length {
       return (Token::EOF, String::from(""));
      }
    }
  }

  // 读取下一个字符
  pub fn read(&mut self) {
    if self.index < self.length {
      self.pos = self.index;
      self.char = self.code.get(self.pos).unwrap().clone();
      self.index = self.index + 1;
    } else {
      self.index = self.length
    }
  }

  // 获取标识符
  fn get_identifier(&mut self) -> Result<String, io::Error> {
    let start_index = self.pos;
    loop {
      if self.char_is_identifier_part() {
        self.read()
      } else {
        break;
      }
    }
    let identifier: String = self.code[start_index..self.pos].iter().collect();
    return Ok(identifier);
  }

  fn scan_number(&mut self) -> (Token, String) {
    return (Token::NUMBER, String::from(""))
  }

  // 查看是否是 标识符的首字符
  // 标识符需要时 大小写字母、$ 或 _ 开头
  // ref: https://developer.mozilla.org/zh-CN/docs/Glossary/Identifier
  fn char_is_identifier_first(&mut self) -> bool {
    if self.char >= 'a' && self.char <= 'z' || self.char >= 'A' && self.char <= 'Z' {
      return true;
    }
    match self.char {
      '$' | '_' => true,
      _ => false   
    }
  }

  // 查看是否是 标识符的一部分
  // 标识符需要时 大小写字母、$、_ 或数字
  fn char_is_identifier_part(&mut self) -> bool {
    if self.char_is_identifier_first() {
      return true;
    }
    if self.char >= '0' && self.char <= '0' {
      return true;
    }
    return false;
  }

  // 跳过空白字符
  fn skip_white_space(&mut self) {
    loop {
      match self.char {
        //  跳过空格
          ' '| '\t' => {
            self.read();
          },
          _ => {
            break;
          }
      }
    }
  }

  // TODO: 抛出错误，未预期的标识符
  fn error_unexpected_token(&mut self, expected: Token) {
    println!("unexpected_token {:?}", expected)
  }

  
}
#[derive(Debug)]
pub struct Program {}


impl Program {}