// AST
// mod super::token::TokenKeywords;
use std::io;

use crate::{ast_token::{get_token_keyword, Token, get_token_literal}, ast_node::{ Expression, NumberLiteral, LetVariableStatement}, ast_utils::{get_hex_number_value, chars_to_string}};
pub struct AST {
  // 当前字符
  char: char,
  // 下一个字符的索引
  next_char_index: usize,
  // 当前字符索引
  cur_char_index: usize,
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
      next_char_index: 0,
      cur_char_index: 0,
      code: chars,
      length: len,
      token: Token::Identifier,
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
    let mut i = 0;
    loop {
      
      if let Token::EOF = self.token  {
        // end of file
        break;
      } else {
        self.parse_statement()
      }
      i = i + 1;
      if i > 1000 {
        break;
      }
    }
  }

  // 解析生成 statement
  fn parse_statement(&mut self) {
    match self.token {
        Token::Let => self.parse_let_statement(),
        _ => {},
    }
  }

  // 解析 let 
  fn parse_let_statement(&mut self) {
    self.check_token_and_next(Token::Let);
    loop {
      self.parse_variable_declaration();
      println!("let {:?} {}", self.token, self.literal);
      // let a= 1, b = 2;
      if self.token != Token::Comma {
        break;
      }
      self.next();
    }
    self.semicolon();
  }

  // 解析变量定义 a = 123,b,c = true 
  fn parse_variable_declaration(&mut self) {
    if Token::Identifier != self.token {
      // TODO: throw error 需要一个identifier
      return;
    }
    let literal = self.literal.clone();
    self.next();
    println!("nexg:{}", self.char);
    let node = LetVariableStatement{
      name: literal
    };

    if self.token == Token::Assign {
      self.next();
      self.parse_assignment_expression();
    }
    println!("literal:{:?}", node);
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

  fn semicolon(&mut self) {
    self.check_token_and_next(Token::Semicolon)
  }

  // 获取下一个符号
  fn next(&mut self) {
    let scan_res = self.scan();
    self.token = scan_res.0;
    self.literal = scan_res.1;
    println!("next:{:?}, {}", self.token, self.literal)
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
                return (Token::Identifier, literal)
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
      // 字符串
      if self.char == '"' || self.char == '\'' {
        return self.scan_string();
      }

      if self.next_char_index == self.length {
        return (Token::EOF, String::from(""));
      }
      
      let cur_char = self.char;
      let cur_char_string = String::from(cur_char);
      let (token, literal) =  match cur_char {
        '=' => (Token::Assign, cur_char_string),
        ':' => (Token::Colon, cur_char_string),
        '.' => (Token::Period, cur_char_string),
        ',' => (Token::Comma, cur_char_string),
        ';' => (Token::Semicolon, cur_char_string),
        '(' => (Token::LeftParenthesis, cur_char_string),
        ')' => (Token::RightParenthesis, cur_char_string),
        '[' => (Token::LeftBracket, cur_char_string),
        ']' => (Token::RightBracket, cur_char_string),
        '{' => (Token::LeftBrace, cur_char_string),
        '}' => (Token::RightBrace, cur_char_string),
        _ => (Token::ILLEGAL, cur_char_string),
      };
      self.read();
      return (token, literal);
    }
  }

  // 读取下一个字符
  pub fn read(&mut self) {
    if self.next_char_index < self.length {
      self.cur_char_index = self.next_char_index;
      self.char = self.code.get(self.cur_char_index).unwrap().clone();
      self.next_char_index = self.next_char_index + 1;
    } else {
      self.next_char_index = self.length
    }
    println!("read:{}, {},  {}", self.char, self.cur_char_index,self.next_char_index)
  }

  // 获取标识符
  fn get_identifier(&mut self) -> Result<String, io::Error> {
    let start_index = self.cur_char_index;
    loop {
      if self.char_is_identifier_part() {
        self.read()
      } else {
        break;
      }
    }
    let identifier: String = chars_to_string(&self.code, start_index, self.cur_char_index);
    return Ok(identifier);
  }

  // 扫描数字字面量
  fn scan_number(&mut self) -> (Token, String) {
    // 十进制
    // 八进制 0777 |0o777 | 0O777
    // 二进制 0b | 0B
    // 十六进制 0x | 0X
    let start_index = self.cur_char_index;
    if self.char == '0' {
      self.read();
      match self.char {
        'x' | 'X' => {
          // 十六进制
          // 自动读取下一个字符
         self.read_number(16);
         let number_len = self.cur_char_index - start_index;

         if number_len <= 2 {
          self.error_common("Illegal hex characters");
         }

        },
        'b' | 'B' => {
          // 二进制
        },
        _ => {

        }
      }
    }
    // 十进制
    self.read_number(10);
    println!("number{}", start_index);
    return (Token::Number, chars_to_string(&self.code, start_index, self.next_char_index))
  }

  fn read_number(&mut self, binary: i32) {
    loop {
      if get_hex_number_value(self.char) < binary {
        self.read()
      } else {
        break;
      }
    }
  }

  // 扫描字符串字面量
  fn scan_string(&mut self) -> (Token, String) {
    let start_index = self.cur_char_index;
    let str_start = self.char.clone();
    self.read();
    let mut i = 0;
    while self.char != str_start {
      if i > 10 {
        break;
      }
      i = i + 1;
      self.read();
    }
    self.read();
    return (Token::String, chars_to_string(&self.code, start_index, self.next_char_index))
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
    if self.char >= '0' && self.char <= '9' {
      return true;
    }
    return false;
  }

  // 解析赋值表达式
  fn parse_assignment_expression(&mut self) -> Expression {
    // 解析三目表达式 bool ? a: b;
    let left = self.parse_conditional_expression();
    let operator = match self.token {
      Token::Assign => {
        Token::Assign
      },
      _ => Token::ILLEGAL,
    };
    // 获取右值
    self.next();
    if operator == Token::ILLEGAL {
      return left;
    }
    return left

  }

  fn parse_conditional_expression(&mut self) -> Expression {
    return self.parse_literal_expression();
  }

  // 解析字面量
  fn parse_literal_expression(&mut self) -> Expression {
    let literal = self.literal.clone();
    match self.token {
      Token::Number => {
        println!("literal {}", literal);
        return Expression::Number(NumberLiteral {

        })
      },
      _ => ()
    }
    return Expression::Number(NumberLiteral {

    })
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

  fn error_common(&mut self, error_msg: &str) {
    println!("Error: {:?}", error_msg)
  }
}
#[derive(Debug)]
pub struct Program {}


impl Program {}