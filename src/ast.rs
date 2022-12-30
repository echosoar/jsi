// AST
// mod super::token::TokenKeywords;
use std::io;

use crate::{ast_token::{get_token_keyword, Token, get_token_literal}, ast_node::{ Expression, NumberLiteral, LetVariableStatement, StringLiteral, LetVariableDeclaration, Statement, IdentifierLiteral, ExpressionStatement, PropertyAccessExpression, BinaryExpression}, ast_utils::{get_hex_number_value, chars_to_string}};
const AST_PRIORITY_MAX: i32 = 20;
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
  // 当前标识符优先级
  token_priority: i32,
  // 当前字面量
  literal: String,
  // 当前表达式
  cur_expr: Expression,
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
      token_priority: 0,
      literal: String::from(""),
      cur_expr: Expression::Unknown,
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
    let body = self.parse_statements();
    // 关闭一个作用域
    self.close_scope();
    Program {
      body
    }
  }
  // 开启一个作用域
  fn open_scope(&mut self) {

  }

   // 关闭一个作用域
   fn close_scope(&mut self) {

   }

  fn parse_statements(&mut self) -> Vec<Statement> {
    let mut statements: Vec<Statement> = vec![];
    loop {
      
      if let Token::EOF = self.token {
        // end of file
        break;
      }
      let statement = self.parse_statement();
      println!("statement: {:?}", statement);
      if let Statement::Unknown = statement  {
        // TODO: unknown statement
        break;
      }
      statements.push(statement);
    }
    return statements;
  }

  // 解析生成 statement
  fn parse_statement(&mut self) -> Statement {
    match self.token {
        Token::Let => self.parse_let_statement(),
        _ => {
          let expression = self.parse_expression();
          match  expression {
              Expression::Unknown => {
                Statement::Unknown
              },
              _ => {
                Statement::Expression(ExpressionStatement{
                  expression
                })
              }
          }
          
        },
    }
  }

  // 解析 let 
  fn parse_let_statement(&mut self) -> Statement {
    self.check_token_and_next(Token::Let);
    let mut let_statement = LetVariableStatement {
      list: vec![],
    };
    loop {
      let expression = self.parse_variable_declaration();
      let_statement.list.push(expression);
      // let a= 1, b = 2;
      if self.token != Token::Comma {
        break;
      }
      self.next();
    }
    self.semicolon();
    return Statement::Let(let_statement);
  }

  // 解析变量定义 a = 123,b,c = true 
  fn parse_variable_declaration(&mut self) -> Expression {
    if Token::Identifier != self.token {
      // TODO: throw error 需要一个identifier
      return Expression::Unknown;
    }
    let literal = self.literal.clone();
    self.next();
    let mut node = LetVariableDeclaration{
      name: literal,
      initializer: Box::new(Expression::Undefined),
    };

    if self.token == Token::Assign {
      self.next();
      node.initializer = Box::new(self.parse_expression());
    }
    return Expression::Let(node)
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
    self.token_priority = scan_res.2;
    println!("next: >{:?}<, >{}<, >{}<", self.token, self.literal, self.char);
  }

  // 扫描获取符号
  pub fn scan(&mut self) -> (Token, String, i32) {
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
                return (Token::Identifier, literal, 0)
              },
              _ => {
                // 非字面量:null、true和false
                return (token_literal, literal, 0)
              },
            }
          },
          _ => {
            // 关键字
            return (token, literal, 0)
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
        return (Token::EOF, String::from(""), 0);
      }
      
      let cur_char = self.char;
      let cur_char_string = String::from(cur_char);
      self.read();
      let (token, literal, priority) =  match cur_char {
        '+' => (Token::Plus, cur_char_string, 12),
        '-' => (Token::Minus, cur_char_string, 12),
        '>' => (Token::Greater, cur_char_string, 10),
        '<' => (Token::Less, cur_char_string, 10),
        '=' => (Token::Assign, cur_char_string, 2),
        ':' => (Token::Colon, cur_char_string, 0),
        '.' => (Token::Period, cur_char_string, 18),
        ',' => (Token::Comma, cur_char_string, 1),
        ';' => (Token::Semicolon, cur_char_string, 0),
        '(' => (Token::LeftParenthesis, cur_char_string, 18),
        ')' => (Token::RightParenthesis, cur_char_string, 18),
        '[' => (Token::LeftBracket, cur_char_string, 18),
        ']' => (Token::RightBracket, cur_char_string, 18),
        '{' => (Token::LeftBrace, cur_char_string, 0),
        '}' => (Token::RightBrace, cur_char_string, 0),
        '?' => (Token::QuestionMark, cur_char_string, 3),
        _ => (Token::ILLEGAL, cur_char_string, 0),
      };
     
      return (token, literal, priority);
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
    // println!("read:{}, {},  {}", self.char, self.cur_char_index,self.next_char_index)
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
  fn scan_number(&mut self) -> (Token, String, i32) {
    // 十进制
    // 八进制 0777 |0o777 | 0O777
    // 二进制 0b | 0B
    // 十六进制 0x | 0X
    let start_index = self.cur_char_index;
    if self.char == '0' {
      self.read();
      match self.char {
        'x' | 'X' => {
          // TODO: 十六进制
          // 自动读取下一个字符
         self.read_number(16);
         let number_len = self.cur_char_index - start_index;

         if number_len <= 2 {
          self.error_common("Illegal hex characters");
         }

        },
        'b' | 'B' => {
          // TODO: 二进制
        },
        '.' => {
          // TODO: 浮点数
        },
        _ => {

        }
      }
    }
    // 十进制
    self.read_number(10);
    // 浮点数
    if self.char == '.' {
      self.read();
      self.read_number(10);
    }
    return (Token::Number, chars_to_string(&self.code, start_index, self.cur_char_index), 0)
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
  fn scan_string(&mut self) -> (Token, String, i32) {
    let start_index = self.cur_char_index;
    let str_start = self.char.clone();
    self.read();
    while self.char != str_start {
      // TODO: '\'aa\''
      self.read();
    }
    let literal = chars_to_string(&self.code, start_index, self.next_char_index);
    self.read();
    return (Token::String, literal, 0);
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
  // 解析表达式
  fn parse_expression(&mut self) -> Expression  {
    return self.parse_relationship_expression();
  }
  // 解析关系运算符 > /< 优先级 10
  fn parse_relationship_expression(&mut self) -> Expression {
    let left = self.parse_access_expression();
    if self.token == Token::Less || self.token == Token::Greater {
      return Expression::Binary(BinaryExpression {
        left: Box::new(left),
        operator: self.token.clone(),
        right: Box::new(self.parse_access_expression())
      })
    }
    return left;
  }


  // 解析访问(.、[])语法 优先级 18
  fn parse_access_expression(&mut self) -> Expression {
    let mut left = self.parse_literal_expression();
    loop {
      self.cur_expr = left.clone();
      let new_left = match self.token {
        Token::Period => self.parse_property_access_expression(),
        Token::LeftParenthesis => self.parse_call_expression(),
        _ => Expression::Unknown,
      };
      if let  Expression::Unknown = new_left {
        break;
      }
      left = new_left;
    }
    return left;
  }
  // 解析属性访问(.)语法 优先级 18
  fn parse_property_access_expression(&mut self) -> Expression {
    self.next();
    let literal = self.literal.clone();
    self.next();
    return Expression::PropertyAccess(PropertyAccessExpression{
      expression: Box::new(self.cur_expr.clone()),
      name: IdentifierLiteral { literal }
    });
  }

  // 解析属方法调用语法 优先级 18
  fn parse_call_expression(&mut self) -> Expression {
    // 1. 解析参数
    self.parse_arguments();
    // CallExpression {}
    return Expression::Unknown;
  }

  // 解析字面量 优先级 20 最后处理
  fn parse_literal_expression(&mut self) -> Expression {
    let literal = self.literal.clone();
    match self.token {
      Token::Identifier => {
        self.next();
        Expression::Identifier(IdentifierLiteral{
          literal
        })
      },
      Token::Number => {
        let value = self.parse_number_literal_expression();
        self.next();
        return Expression::Number(NumberLiteral {
          literal,
          value,
        })
      },
      Token::String => {
        let str_len = literal.len();
        let slice = String::from(&self.literal[1..str_len-1]);
        self.next();
        Expression::String(StringLiteral{
          literal,
          value: slice
        })
      },
      _ => Expression::Unknown,
    }
  }

  // 解析参数
  fn parse_arguments(&mut self) {
    self.check_token_and_next(Token::LeftParenthesis);
    let arguments:Vec<i32> = vec![];
    while self.token != Token::RightParenthesis && self.token != Token::EOF {
      println!("arguments expr pre {:?}", self.token);
      let expr = self.parse_expression();
      println!("arguments expr:{:?}", expr);
      if self.token != Token::Comma {
				break
			}
      // self.next()
    }
    println!("arguments:{:?}", arguments)
  }

  fn parse_number_literal_expression(&mut self) -> f64 {
    // 检测是否是 float
    self.literal.parse::<f64>().unwrap()
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
    println!("unexpected_token {:?} {:?}", expected, self.token)
  }

  fn error_common(&mut self, error_msg: &str) {
    println!("Error: {:?}", error_msg)
  }
}
#[derive(Debug)]
pub struct Program {
  pub body: Vec<Statement>
}


impl Program {}