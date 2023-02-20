// AST
// mod super::token::TokenKeywords;
use std::io;

use crate::ast_token::{get_token_keyword, Token, get_token_literal};
use crate::ast_node::{ Expression, NumberLiteral, StringLiteral, Statement, IdentifierLiteral, ExpressionStatement, PropertyAccessExpression, BinaryExpression, ConditionalExpression, CallExpression, Keywords, Parameter, BlockStatement, ReturnStatement, Declaration, PropertyAssignment, ObjectLiteral, ElementAccessExpression, FunctionDeclaration, PostfixUnaryExpression, PrefixUnaryExpression, AssignExpression, GroupExpression, VariableDeclaration, VariableDeclarationStatement, VariableFlag, ClassDeclaration, ClassMethodDeclaration, ArrayLiteral, ComputedPropertyName, IfStatement, ForStatement, BreakStatement, ContinueStatement, LabeledStatement, SwitchStatement, CaseClause};
use crate::ast_utils::{get_hex_number_value, chars_to_string};
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
  // 当前表达式
  cur_expr: Expression,
  // 当前上下文
  scope: ASTScope,
  // 前一个 token 如果后面存在换行，是否需要添加 semicolon
  pre_token_need_semicolon: bool,
  // 当碰到换行时，是否需要自动添加 semicolon
  auto_semicolon_when_new_line: bool,
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
      cur_expr: Expression::Unknown,
      scope: ASTScope::new(),
      pre_token_need_semicolon: false,
      auto_semicolon_when_new_line: false,
    }
  }

  // 解析生成 Program
  pub fn parse(&mut self) -> Program {
    self.next();
    return self.parse_program()
  }

  // 解析生成 program
  fn parse_program(&mut self) -> Program {
    self.new_scope();
    let body = self.parse_statements();
    let declarations = self.scope.declarations.clone();
    self.close_scope();
    Program {
      body,
      declarations,
    }
  }

  // 创建新的上下文环境，用于存储当前上下文环境中声明的方法和变量
  fn new_scope(&mut self) {
    let mut scope = ASTScope::new();
    scope.parent = Some(Box::new(self.scope.clone()));
    self.scope = scope;
  }

  fn close_scope(&mut self) {
    if let Some(parent) = self.scope.parent.clone() {
      self.scope = *parent
    }
  }

  fn parse_statements(&mut self) -> Vec<Statement> {
    let mut statements: Vec<Statement> = vec![];
    loop {
      if self.token == Token::EOF || self.token == Token::RightBrace {
        // end of file
        // 结束了块级作用域
        break;
      }
      if self.token == Token::Semicolon {
        self.next();
        continue;
      }
      let statement = self.parse_statement();
      if let Statement::Unknown = statement  {
        // TODO: unknown statement
        println!("statement: {:?}", statement);
        break;
      }
      statements.push(statement);
    }
    return statements;
  }

  // 解析生成 statement
  fn parse_statement(&mut self) -> Statement {
    match self.token {
        Token::Var | Token::Let => self.parse_variable_statement(),
        Token::If => self.parse_if_statement(),
        Token::Switch => self.parse_switch_statement(),
        Token::For => self.parse_for_statement(),
        Token::While => self.parse_while_statement(),
        Token::Do => self.parse_do_while_statement(),
        Token::Break => self.parse_break_statement(),
        Token::Continue => self.parse_continue_statement(),
        Token::Function => {
          Statement::Function(self.parse_function(true))
        },
        Token::Return => self.parse_return_statement(),
        Token::Class => {
          // class (ES2015)
          Statement::Class(self.parse_class())
        },
        Token::LeftBrace => {
          // block
          self.parse_block_statement()
        },
        _ => {
          let expression = self.parse_expression();

          // label:
          if self.token == Token::Colon {
            if let Expression::Identifier(identifier) = expression {
              // TODO: 检测当前的作用域是否已经存在这个 lable，如果存在，则报错
              //  let label = identifier.literal;
              self.next();
               let statement = self.parse_statement();
               return Statement::Label(LabeledStatement {
                label: identifier,
                statement: Box::new(statement),
               });
            }
          }
          
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

  // 解析 let / var
  fn parse_variable_statement(&mut self) -> Statement {
    let mut variable_flag = VariableFlag::Var;
    if self.token == Token::Let {
      self.check_token_and_next(Token::Let);
      variable_flag = VariableFlag::Let;
    } else {
      self.check_token_and_next(Token::Var);
    }
    
    let var_statement = VariableDeclarationStatement {
      list: self.parse_variable_declarations(),
      flag: variable_flag,
    };
    self.semicolon();
    return Statement::Var(var_statement);
  }

  fn parse_variable_declarations(&mut self) -> Vec<Expression> {
    let mut list: Vec<Expression> = vec![];
    loop {
      let expression = self.parse_variable_declaration();
      list.push(expression);
      // let a= 1, b = 2;
      if self.token != Token::Comma {
        break;
      }
      self.next();
    }
    list
  }


  // 解析 block statement
  fn parse_block_statement(&mut self) -> Statement {
    // 以左花括号开始
    self.check_token_and_next(Token::LeftBrace);
    let statements = self.parse_statements();
    self.check_token_and_next(Token::RightBrace);
    return Statement::Block(BlockStatement{
      statements,
    })
  }

  // 解析 if/else/else if
  fn parse_if_statement(&mut self)  -> Statement {
    self.check_token_and_next(Token::If);
    self.check_token_and_next(Token::LeftParenthesis);
    let mut statement = IfStatement {
      condition: self.parse_expression(),
      then_statement: Box::new(Statement::Unknown),
      else_statement: Box::new(Statement::Unknown),
    };
    self.check_token_and_next(Token::RightParenthesis);
    // 判断是否是 单行if
    if self.token == Token::LeftBrace {
      statement.then_statement = Box::new(self.parse_block_statement());
    } else {
      statement.then_statement = Box::new(self.parse_statement());
    }

    if self.token == Token::Else {
      self.next();
      statement.else_statement = Box::new(self.parse_statement());
    }
    return Statement::If(statement)
  }


  // 解析 switch case
  fn parse_switch_statement(&mut self)  -> Statement {
    self.check_token_and_next(Token::Switch);
    self.check_token_and_next(Token::LeftParenthesis);
    let condition = self.parse_expression();
    self.check_token_and_next(Token::RightParenthesis);
    self.check_token_and_next(Token::LeftBrace);
    let mut default_index: i32 = -1;
    let mut clauses: Vec<CaseClause> = vec![];
    loop {
      if self.token == Token::EOF || self.token == Token::RightBrace {
        break;
      }
      let mut clause = CaseClause {
        condition: None,
        statements: vec![],
      };
      // parse case
      if self.token == Token::Default {
        if default_index != -1 {
          // TODO: throw new error
        } else {
          default_index = clauses.len() as i32;
          self.next();
        }
      } else {
        self.check_token_and_next(Token::Case);
        clause.condition = Some(self.parse_expression());
      }
      self.check_token_and_next(Token::Colon);
      loop {
        if self.token == Token::EOF || self.token == Token::RightBrace || self.token == Token::Case || self.token == Token::Default {
          break;
        }
        let statement = self.parse_statement();
        clause.statements.push(statement);
      }
      clauses.push(clause);
    }


    self.check_token_and_next(Token::RightBrace);
    Statement::Switch(SwitchStatement {
      condition,
      clauses,
      default_index
    })
  }

  // 解析 for 循环
  // TODO: for in/ of
  fn parse_for_statement(&mut self)  -> Statement {
    self.check_token_and_next(Token::For);
    self.check_token_and_next(Token::LeftParenthesis);
    // 解析 initializer
    // 需要额外处理 var 的情况
    let mut initializer = Statement::Unknown;
    if self.token == Token::Var || self.token == Token::Let {
        initializer = self.parse_variable_statement();
    } else if self.token != Token::Semicolon {
      initializer = Statement::Expression(ExpressionStatement { expression: self.parse_expression() });
      self.check_token_and_next(Token::Semicolon);
    }
    let condition = self.parse_expression();
    self.check_token_and_next(Token::Semicolon);
    let incrementor = self.parse_expression();
    self.check_token_and_next(Token::RightParenthesis);

    let block = self.parse_block_statement();
    let statement = ForStatement {
      initializer: Box::new(initializer),
      condition: condition,
      incrementor: incrementor,
      statement: Box::new(block),
      post_judgment: false,
    };
    return  Statement::For(statement);
  }

  // 解析 while 循环
  fn parse_while_statement(&mut self)  -> Statement {
    self.check_token_and_next(Token::While);
    self.check_token_and_next(Token::LeftParenthesis);
    let condition = self.parse_expression();
    self.check_token_and_next(Token::RightParenthesis);

    let block = self.parse_block_statement();
    let statement = ForStatement {
      initializer: Box::new(Statement::Unknown),
      condition: condition,
      incrementor: Expression::Unknown,
      statement: Box::new(block),
      post_judgment: false,
    };
    return  Statement::For(statement);
  }


  // 解析 do while 循环
  fn parse_do_while_statement(&mut self)  -> Statement {
    self.check_token_and_next(Token::Do);
    let block = self.parse_block_statement();
    self.check_token_and_next(Token::While);
    self.check_token_and_next(Token::LeftParenthesis);
    let condition = self.parse_expression();
    self.check_token_and_next(Token::RightParenthesis);
    let statement = ForStatement {
      initializer: Box::new(Statement::Unknown),
      condition: condition,
      incrementor: Expression::Unknown,
      statement: Box::new(block),
      post_judgment: true,
    };
    return  Statement::For(statement);
  }


  fn parse_break_statement(&mut self) -> Statement {
    self.check_token_and_next(Token::Break);
    let mut semicolon = false;
    // break;
    if self.token == Token::Semicolon {
      self.next();
      semicolon = true;
    }

    // for() { break }
    if semicolon || self.auto_semicolon_when_new_line || self.token == Token::RightBrace {
      /*
      TODO:
      if self.scope.in_iteration || self.scope.in_switch {

      } else {
        // need label, throw error Illegal break statement
      }
      */
      return Statement::Break(BreakStatement {
        label: None
      });
    }

    self.check_token(Token::Identifier);
    return  Statement::Break(BreakStatement {
      label: Some(IdentifierLiteral { literal: self.literal.clone() })
    });
  }

  fn parse_continue_statement(&mut self) -> Statement {
    self.check_token_and_next(Token::Continue);
    let mut semicolon = false;
    // continue;
    if self.token == Token::Semicolon {
      self.next();
      semicolon = true;
    }

    // for() { continue }
    if semicolon || self.token == Token::RightBrace {
      return Statement::Continue(ContinueStatement {
        label: None
      });
    }

    self.check_token(Token::Identifier);
    return  Statement::Continue(ContinueStatement {
      label: Some(IdentifierLiteral { literal: self.literal.clone() })
    });
  }

  // 解析 function statement
  fn parse_function(&mut self, variable_lifting: bool) -> FunctionDeclaration {
    // 如果是 function 关键字，则跳过
    if self.token == Token::Function {
      self.next();
    }
    
    // 解析方法名
    let mut is_anonymous = true;
    let mut name = String::new();
    if self.token == Token::Identifier {
      is_anonymous = false;
      name = self.literal.clone();
      self.next();
    }
    // 解析参数
    // 左括号
    let mut parameters: Vec<Parameter> = vec![];
    self.check_token_and_next(Token::LeftParenthesis);
    while self.token != Token::RightParenthesis && self.token != Token::EOF {
      if self.token == Token::Identifier {
        parameters.push(Parameter{
          name: IdentifierLiteral { literal: self.literal.clone() },
          initializer: Box::new(Expression::Keyword(Keywords::Undefined)),
        });
        self.next()
      } else {
        self.check_token(Token::Identifier);
      }
      if self.token != Token::RightParenthesis {
        self.check_token_and_next(Token::Comma);
      }
    }

    self.check_token_and_next(Token::RightParenthesis);
    // 需要开启一个新的作用域，用来记录 block 里面的 方法定义 和 变量定义，因为方法定义是要提升到作用域最开始的
    self.new_scope();
    // 解析方法体
    let body = match self.parse_block_statement() {
      Statement::Block(block) => block,
      _ => BlockStatement { statements: vec![] }
    };
    let declarations = self.scope.declarations.clone();
    self.close_scope();
    let func = FunctionDeclaration {
      is_anonymous,
      name: IdentifierLiteral { literal: name },
      parameters,
      body,
      declarations,
    };
    if variable_lifting && !is_anonymous {
      self.scope.declare(Declaration::Function(func.clone()));
    }
    return func;
  }

  // 解析 class(ES2015)
  fn parse_class(&mut self) -> ClassDeclaration {
    self.check_token_and_next(Token::Class);
    // class name
    self.check_token(Token::Identifier);
    let name = self.literal.clone();
    self.next();
    // extends
    if self.token == Token::Extends {
      // TODO: 解析 extends
    }
    self.check_token_and_next(Token::LeftBrace);
    let mut members:  Vec<Expression>= vec![];
    while self.token != Token::RightBrace && self.token != Token::EOF {
      let mut modifiers: Vec<Token> = vec![];
      loop {
        match self.token {
          // ES not define Token::Private | Token::Public | Token::Protected |
          Token::Async => {
            modifiers.push(self.token.clone());
            self.next();
            continue;
          },
          _ => {
            break;
          }
        };
      }
     
      if self.token == Token::Identifier {
        if self.literal == String::from("constructor") {
          // constructor
          let constructor = self.parse_function(false);
          members.push(Expression::Constructor(constructor));
        } else if self.next_is('(', true) {
          // method
          let method = self.parse_function(false);
          members.push(Expression::ClassMethod(ClassMethodDeclaration {
            name: method.name.clone(),
            modifiers,
            method: Box::new(method), 
          }));
        } else {
          // TODO: property
          self.next()
        }
        
      } else {
        // TODO: throw error
        self.next()
      }
    }
    ClassDeclaration {
      name: IdentifierLiteral { literal: name },
      members,
      heritage: None,
    }
  }

  fn parse_return_statement(&mut self) -> Statement {
    self.check_token_and_next(Token::Return);
    let mut expression = Expression::Keyword(Keywords::Undefined);
    if  !self.auto_semicolon_when_new_line && self.token != Token::Semicolon && self.token != Token::RightBrace && self.token != Token::EOF {
      expression = self.parse_expression()
    }
    self.semicolon();
    return Statement::Return(ReturnStatement{
      expression
    });
  }

  // 解析变量定义 a = 123,b,c = true 
  fn parse_variable_declaration(&mut self) -> Expression {
    if Token::Identifier != self.token {
      // TODO: throw error 需要一个identifier
      return Expression::Unknown;
    }
    let literal = self.literal.clone();
    self.next();
    let mut node = VariableDeclaration{
      name: literal,
      initializer: Box::new(Expression::Keyword(Keywords::Undefined)),
    };

    if self.token == Token::Assign {
      self.next();
      node.initializer = Box::new(self.parse_expression());
    }
    return Expression::Var(node)
  }

  fn check_token_and_next(&mut self, token: Token) {
    self.check_token(token);
    self.next();
  }
  fn check_token(&mut self, token: Token) {
    // TODO: 类型不匹配，需要报错
    if token != self.token {
      self.error_unexpected_token(token)
    }
    
  }

  fn semicolon(&mut self) {
    // 如果是自动添加的分号，则跳过
    if self.auto_semicolon_when_new_line {
      self.auto_semicolon_when_new_line = false;
    } else {
      self.check_token_and_next(Token::Semicolon)
    }
  }

  // 获取下一个符号
  fn next(&mut self) {
    let scan_res = self.scan();
    self.token = scan_res.0;
    self.literal = scan_res.1;
    // println!("out next: >{:?}<, >{}<, >{}<", self.token, self.literal, self.char);
  }

  // 查看下一次 scan 获取的是不是 token
  fn next_is(&mut self, check_char: char, skip_space: bool) -> bool {
    let mut start_index = self.cur_char_index;
    loop {
      // EOF
      if start_index >= self.length {
        return false;
      }
      let char = self.code[start_index];
      start_index = start_index + 1;
      if skip_space {
        match char {
          // TODO: 更多空白符
          ' ' | '\n' | '\r' | '\t' => {
            continue;
          },
          _ => {}
        }
      }
      return char == check_char; 
    }
  }
  // 扫描获取符号
  pub fn scan(&mut self) -> (Token, String) {
    // TODO: 严格模式
    let is_strict = true;
    // 默认不自动添加分号
    // 自动添加分号的相关规范 ref: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    self.auto_semicolon_when_new_line = false;
    loop {
      self.skip_white_space();
      if self.cur_char_index >= self.length {
        // 扫描结束了
        self.pre_token_need_semicolon = false;
        self.auto_semicolon_when_new_line = true;
        return (Token::EOF, String::from(""));
      }
      if self.char_is_identifier_first() {
        let literal = self.get_identifier().unwrap();
        
        let token = get_token_keyword(&literal, is_strict);
        match token {
          Token::ILLEGAL => {
            // 非关键字
            let token_literal = get_token_literal(&literal);
            match token_literal {
              Token::ILLEGAL => {
                // 其他非字面量，比如用户自定义变量
                self.pre_token_need_semicolon = true;
                return (Token::Identifier, literal)
              },
              _ => {
                // 非字面量:null、true和false
                return (token_literal, literal)
              },
            }
          },
          _ => {
            match token {
              Token::Continue | Token::Break | Token::Return | Token::Yield => {
                // TODO: module
                // 语法限制，如果这些关键字后面有换行，则自动结尾
                /*
                return
                a + b
                ---
                return;
                a + b;
                */
                self.pre_token_need_semicolon = true;
              },
              _ => {}
            };
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
      
      let cur_char = self.char;
      let mut cur_char_string = String::from(cur_char);
      self.read();
      let (token, literal) =  match cur_char {
        '\n' => {
          // 如果前一个 token 碰到换行时，需要添加分号
          if self.pre_token_need_semicolon {
            self.pre_token_need_semicolon = false;
            self.auto_semicolon_when_new_line = true;
          }
          continue;
        },
        '+' => {
          if self.char == '=' {
            // oper: +=
            cur_char_string.push(self.char);
            self.read();
            (Token::AddAssign, cur_char_string)
          } else if self.char == '+' {
            // oper: ++
            self.pre_token_need_semicolon = true;
            cur_char_string.push(self.char);
            self.read();
            (Token::Increment, cur_char_string)
          } else {
            // oper: +
            (Token::Plus, cur_char_string)
          }
        },
        '-' => {
          if self.char == '=' {
            // oper: -=
            cur_char_string.push(self.char);
            self.read();
            (Token::SubtractAssign, cur_char_string)
          } else if self.char == '-' {
            // oper: --
            self.pre_token_need_semicolon = true;
            cur_char_string.push(self.char);
            self.read();
            (Token::Decrement, cur_char_string)
          } else {
            // oper: -
            (Token::Subtract, cur_char_string)
          }
        },
        '*' => {
          if self.char == '=' {
            // oper: *=
            cur_char_string.push(self.char);
            self.read();
            (Token::MultiplyAssign, cur_char_string)
          } else if self.char == '*' {
            cur_char_string.push(self.char);
            self.read();
            if self.char == '=' {
              // oper: **=
              cur_char_string.push(self.char);
              self.read();
              (Token::ExponentiationAssign, cur_char_string)
            } else {
              // oper ** 幂运算 Exponentiation Operator（ES2017）
              (Token::Exponentiation, cur_char_string)
            }
            
          } else {
            // oper: *
            (Token::Multiply, cur_char_string)
          }
        },
        '/' => {
          if self.char == '/' {
            // 单行注释
            loop {
              self.read();
              match self.char {
                '\n' => {
                  self.read();
                  break;
                },
                _ => {
                  // EOF
                  if self.cur_char_index >= self.length  {
                    break;
                  }
                }
              };
            }
            continue;
          } else if self.char == '*' {
            // 多行注释
            loop {
              self.read();
              if self.char == '*' {
                self.read();
                if self.char == '/' {
                  self.read();
                  break;
                }
              }
            }
            continue;
          } else if self.char == '=' {
            // oper: /=
            cur_char_string.push(self.char);
            self.read();
            (Token::SlashAssign, cur_char_string)
          } else {
            // oper: /
            (Token::Slash, cur_char_string)
          }
        },
        '%' => {
          if self.char == '=' {
            // oper: %=
            cur_char_string.push(self.char);
            self.read();
            (Token::RemainderAssign, cur_char_string)
          } else {
            // oper: %
            (Token::Remainder, cur_char_string)
          }
        },
        '>' => {
          if self.char == '>' {
            cur_char_string.push(self.char);
            self.read();
            if self.char == '>' {
              cur_char_string.push(self.char);
              self.read();
              if self.char == '=' {
                  // oper: >>>=
                  cur_char_string.push(self.char);
                self.read();
                  (Token::UnsignedShiftRightAssign, cur_char_string)
              } else {
                  //oper:  >>>
                (Token::UnsignedShiftRight, cur_char_string)
              }
            } else if self.char == '=' {
              // oper: >>=
              cur_char_string.push(self.char);
              self.read();
              (Token::ShiftRightAssign, cur_char_string)
            } else {
              // oper: >>
              (Token::ShiftRight, cur_char_string)
            }
          } else if self.char == '=' {
            // oper: >=
            cur_char_string.push(self.char);
            self.read();
            (Token::GreaterOrEqual, cur_char_string)
          } else {
            // oper: >
            (Token::Greater, cur_char_string)
          }
        },
        '<' => {
          if self.char == '<' {
            cur_char_string.push(self.char);
            self.read();
            if self.char == '=' {
              // oper: <<=
              cur_char_string.push(self.char);
              self.read();
              (Token::ShiftLeftAssign, cur_char_string)
            } else {
              // oper: <<
              (Token::ShiftLeft, cur_char_string)
            }
          } else if self.char == '=' {
            // oper: <=
            cur_char_string.push(self.char);
            self.read();
            (Token::LessOrEqual, cur_char_string)
          } else {
            // oper: <
            (Token::Less, cur_char_string)
          }
        },
        '=' => {
          if self.char == '=' {
            cur_char_string.push(self.char);
            self.read();
            if self.char == '=' {
              cur_char_string.push(self.char);
              self.read();
                // oper: ===
                (Token::StrictEqual, cur_char_string)
            } else {
              // oper: ==
              (Token::Equal, cur_char_string)
            }
          } else {
            // oper: =
            (Token::Assign, cur_char_string)
          }
        },
        ':' => (Token::Colon, cur_char_string),
        '.' => {
          // TODO: float
          (Token::Period, cur_char_string)
        },
        ',' => (Token::Comma, cur_char_string),
        ';' => (Token::Semicolon, cur_char_string),
        '(' => (Token::LeftParenthesis, cur_char_string),
        ')' => {
          self.pre_token_need_semicolon = true;
          (Token::RightParenthesis, cur_char_string)
        },
        '[' => (Token::LeftBracket, cur_char_string),
        ']' => {
          self.pre_token_need_semicolon = true;
          (Token::RightBracket, cur_char_string)
        },
        '{' => (Token::LeftBrace, cur_char_string),
        '}' => {
          self.pre_token_need_semicolon = true;
          (Token::RightBrace, cur_char_string)
        },
        
        '~' => (Token::BitwiseNot, cur_char_string),
        '&' => { // 与
          if self.char == '&' {
            cur_char_string.push(self.char);
            self.read();
            if self.char == '=' {
              cur_char_string.push(self.char);
              self.read();
              // oper: &&=
              (Token::LogicalAndAssign, cur_char_string)
            } else {
              // oper: &&
              (Token::LogicalAnd, cur_char_string)
            }
          } else if self.char == '=' {
            cur_char_string.push(self.char);
            self.read();
              // oper: &=
              (Token::AndAssign, cur_char_string)
          } else {
            // oper: &
            (Token::And, cur_char_string)
          }
        },
        '|' => { // 或
          if self.char == '|' {
            cur_char_string.push(self.char);
            self.read();
            if self.char == '=' {
              cur_char_string.push(self.char);
              self.read();
              // oper: ||=
              (Token::LogicalOrAssign, cur_char_string)
            } else {
              // oper: ||
              (Token::LogicalOr, cur_char_string)
            }
          } else if self.char == '=' {
            cur_char_string.push(self.char);
            self.read();
            // oper: !=
            (Token::OrAssign, cur_char_string)
          } else {
            // oper: !
            (Token::Or, cur_char_string)
          }
        },
        '!' => { // 非
          if self.char == '=' {
            cur_char_string.push(self.char);
            self.read();
            if self.char == '=' {
              cur_char_string.push(self.char);
              self.read();
              // oper: !==
              (Token::StrictNotEqual, cur_char_string)
            } else {
              // oper: !=
              (Token::NotEqual, cur_char_string)
            }
          } else {
            // oper: !
            (Token::Not, cur_char_string)
          }
        },
        '^' => { // 异或
          if self.char == '=' {
            cur_char_string.push(self.char);
            self.read();
            // oper: ^=
            (Token::ExclusiveOrAssign, cur_char_string)
          } else {
            // oper: ^
            (Token::ExclusiveOr, cur_char_string)
          }
        },
        '?' => {
          if self.char == '?' {
            cur_char_string.push(self.char);
            self.read();
            if self.char == '=' {
              cur_char_string.push(self.char);
              self.read();
              // oper: ??=
              (Token::NullishCoalescingAssign, cur_char_string)
            } else {
              // oper: ?? 空值合并运算符 Nullish Coalescing (ES2020)
              (Token::NullishCoalescing, cur_char_string)
            }
          } else if self.char == '.' {
            cur_char_string.push(self.char);
            self.read();
              // oper: ?. 可选链 Optional Chaining (ES2020)
              (Token::OptionalChaining, cur_char_string)
          } else {
            // oper: ?
            (Token::QuestionMark, cur_char_string)
          }
        },
        _ => (Token::ILLEGAL, cur_char_string),
      };
      return (token, literal);
    };
  }

  // 读取下一个字符
  pub fn read(&mut self) -> bool {
    if self.next_char_index < self.length {
      self.cur_char_index = self.next_char_index;
      self.char = self.code.get(self.cur_char_index).unwrap().clone();
      self.next_char_index = self.next_char_index + 1;
      // println!("read:{}, {},  {}", self.char, self.cur_char_index,self.next_char_index);
      true
    } else {
      self.next_char_index = self.length;
      self.cur_char_index = self.length;
      false
    }
    
  }

  // 获取标识符
  fn get_identifier(&mut self) -> Result<String, io::Error> {
    let start_index = self.cur_char_index;
    loop {
      if self.char_is_identifier_part() {
        if !self.read() {
          break;
        }
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
    return (Token::Number, chars_to_string(&self.code, start_index, self.cur_char_index))
  }

  fn read_number(&mut self, binary: i32) {
    loop {
      if get_hex_number_value(self.char) < binary {
        if !self.read() {
          break;
        }
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
    while self.char != str_start {
      // TODO: '\'aa\''
      if !self.read() {
        break;
      }
    }
    let literal = chars_to_string(&self.code, start_index, self.next_char_index);
    self.read();
    return (Token::String, literal);
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
    let res = self.parse_assignment_expression();
    res
  }

  // 解析赋值运算符，优先级 2，从右到左
  // https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-assignment-operators
  fn parse_assignment_expression(&mut self) -> Expression {
    let left = self.parse_conditional_expression();
    match self.token {
      Token::Assign | Token::AddAssign | Token::SubtractAssign | Token::MultiplyAssign | Token::SlashAssign | Token::RemainderAssign | Token::ShiftLeftAssign | Token::ShiftRightAssign | Token::UnsignedShiftRightAssign | Token::OrAssign | Token::AndAssign | Token::ExclusiveOrAssign | Token::LogicalAndAssign | Token::LogicalOrAssign | Token::ExponentiationAssign | Token::NullishCoalescingAssign =>  {
        // 跳过各种赋值运算符
        let oper = self.token.clone();
        self.next();
        // from right to left
        let right = self.parse_expression();
        return Expression::Assign(AssignExpression {
          left: Box::new(left),
          operator: oper,
          right: Box::new(right),
        });
      },
      _ => left
      
    }
  }

  // 解析三目运算符，优先级 3，从右到左
  // https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-conditional-operator
  fn parse_conditional_expression(&mut self) -> Expression {
    let left = self.parse_binary_logical_expression();
    if self.token == Token::QuestionMark {
      // 跳过 ?
      self.next();

      let when_true = self.parse_expression();
      // 期待是 :
      self.check_token_and_next(Token::Colon);
      let when_false = self.parse_expression();
      
      return Expression::Conditional(ConditionalExpression{
        condition: Box::new(left),
        when_true: Box::new(when_true),
        when_false: Box::new(when_false),
      });
    }
  
    return left
  }

  // 逻辑或 || 运算符表达式 和 空值合并表达式 ?? 优先级 4，从左到右
  fn parse_binary_logical_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_logical_and_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::LogicalOr,
      Token::NullishCoalescing,
    ], next)
  }

  // 逻辑与 && 运算符表达式 优先级 5，从左到右
  fn parse_logical_and_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_binary_or_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::LogicalAnd,
    ], next)
  }

  // 按位或 | 运算符表达式 优先级 6，从左到右
  fn parse_binary_or_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_binary_exclusive_or_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::Or,
    ], next)
  }

  // 按位异或 (^) 运算符表达式 优先级 7，从左到右
  fn parse_binary_exclusive_or_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_binary_and_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::ExclusiveOr,
    ], next)
  }

  // 按位与 (&) 运算符表达式 优先级 8，从左到右
  fn parse_binary_and_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_equality_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::And,
    ], next)
  }

  // 相等表达式 优先级 9，从左到右
  fn parse_equality_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_relationship_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::Equal ,
      Token::StrictEqual ,
      Token::NotEqual ,
      Token::StrictNotEqual ,
    ], next)
  }

  // 解析关系运算符 > 、< 、>=、<= 优先级 10，从左到右
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-relational-operators
  fn parse_relationship_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_shift_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::Less ,
      Token::Greater ,
      Token::GreaterOrEqual ,
      Token::LessOrEqual ,
      Token::In ,
      Token::Instanceof
    ], next)
  }

  // 解析位运算符 优先级 11，从左到右
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-bitwise-shift-operators
  fn parse_shift_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_additive_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::ShiftLeft ,
      Token::ShiftRight ,
      Token::UnsignedShiftRight ,
    ], next)
  }


  // 解析 + - 语法 优先级 12，从左到右
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-additive-operators
  fn parse_additive_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_multiplicative_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::Plus ,
      Token::Subtract,
    ], next)
  }

  // 解析 * / % 语法 优先级 13，从左到右
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-additive-operators
  fn parse_multiplicative_expression(&mut self) -> Expression {
    let next = |tst: &mut AST| {
      tst.parse_exponentiation_expression()
    };
    self.parse_left_associate_expression(vec![
      Token::Multiply ,
      Token::Slash ,
      Token::Remainder ,
    ], next)
  }

  // 幂运算 1**2 -- 优先级 14，从右到左
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-unary-operators
  fn parse_exponentiation_expression(&mut self) -> Expression {
    let left = self.parse_prefix_unary_expression();
    if self.token == Token::Exponentiation {
      let operator = self.token.clone();
      self.next();
      let right = self.parse_exponentiation_expression();
      Expression::Binary(BinaryExpression {
        left: Box::new(left),
        operator,
        right: Box::new(right),
      })
    } else {
      left
    }
  }
  // 前置一元运算符  -- 优先级 15，从右到左
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-unary-operators
  fn parse_prefix_unary_expression(&mut self) -> Expression {
    match self.token {
      Token::Not | Token::BitwiseNot | Token::Plus | Token::Subtract => {
        let operator = self.token.clone();
        self.next();
        Expression::PrefixUnary(PrefixUnaryExpression {
          operator,
          operand: Box::new(self.parse_postfix_unary_expression()),
        })
      },
      Token::Typeof | Token::Void | Token::Delete | Token::Await => {
        let operator = self.token.clone();
        self.next();
        Expression::PrefixUnary(PrefixUnaryExpression {
          operator,
          operand: Box::new(self.parse_postfix_unary_expression()),
        })
      },
      Token::Increment | Token::Decrement => {
        let operator = self.token.clone();
        self.next();
        let operand = self.parse_postfix_unary_expression();
        // TODO: check operand is Identifier/Property access
        Expression::PrefixUnary(PrefixUnaryExpression {
          operator,
          operand: Box::new(operand),
        })
      },
      _ => self.parse_postfix_unary_expression()
    }
  }

  // 后置一元运算符 ++ -- 优先级 16
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-update-expressions
  fn parse_postfix_unary_expression(&mut self) -> Expression {
    let left = self.parse_left_hand_side_expression();
    if self.token == Token::Increment || self.token == Token::Decrement {
      if self.auto_semicolon_when_new_line {
        return left
      }
      let expr = Expression::PostfixUnary(PostfixUnaryExpression {
        operator: self.token.clone(),
        operand: Box::new(left),
      });
      self.next();
      expr
    } else {
      left
    }
  }

  // 解析访问(.、[])语法 优先级 18，从左到右
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-left-hand-side-expressions
  fn parse_left_hand_side_expression(&mut self) -> Expression {
    let mut left = self.parse_group_expression();
    loop {
      self.cur_expr = left.clone();
      let new_left = match self.token {
        Token::Period => self.parse_property_access_expression(),
        Token::LeftBracket => self.parse_element_access_expression(),
        Token::LeftParenthesis => self.parse_call_expression(),
        // TODO: new
        // TODO: optional chaining
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

  // 解析属性访问([)语法 优先级 18
  fn parse_element_access_expression(&mut self) -> Expression {
    let expression = Box::new(self.cur_expr.clone());
    self.check_token_and_next(Token::LeftBracket);
    let expr = self.parse_expression();
    self.check_token_and_next(Token::RightBracket);
    return Expression::ElementAccess(ElementAccessExpression{
      expression,
      argument: Box::new(expr),
    });
  }

  // 解析属方法调用语法 优先级 18
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-function-calls
  fn parse_call_expression(&mut self) -> Expression {
    // 1. 解析参数
    let expression = Box::new(self.cur_expr.clone());
    let arguments = self.parse_arguments();
    // CallExpression {}
    self.check_token_and_next(Token::RightParenthesis);
    return Expression::Call(CallExpression {
      expression,
      arguments
    });
  }

  // 解析分组表达式 优先级 19
  // ref: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-function-calls
  fn parse_group_expression(&mut self) -> Expression {
     if self.token == Token::LeftParenthesis {
      self.next();
      let expr = self.parse_expression();
      self.check_token_and_next(Token::RightParenthesis);
      return Expression::Group(GroupExpression {
        expression: Box::new(expr),
      })
     }
     self.parse_literal_expression()
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
      Token::False => {
        self.next();
        Expression::Keyword(Keywords::False)
      },
      Token::True => {
        self.next();
        Expression::Keyword(Keywords::True)
      },
      Token::Null => {
        self.next();
        Expression::Keyword(Keywords::Null)
      },
      Token::Undefined => {
        self.next();
        Expression::Keyword(Keywords::Undefined)
      },
      Token::LeftBrace => {
        self.parse_object_literal()
      },
      Token::LeftBracket => {
        self.parse_array_literal()
      },
      Token::Function => {
        Expression::Function(self.parse_function(true))
      },
      _ => {
        self.next();
        Expression::Unknown
      },
    }
  }

  // 解析数组字面量
  fn parse_array_literal(&mut self) -> Expression {
    self.check_token_and_next(Token::LeftBracket);
    let mut elements: Vec<Expression>= vec![];
    while self.token != Token::RightBracket && self.token != Token::EOF {
      // [,,1]
      if self.token == Token::Comma {
        elements.push(Expression::Keyword(Keywords::Undefined));
        self.next();
        continue;
      }
      let item = self.parse_expression();
      elements.push(item);
      if self.token != Token::RightBracket {
        self.check_token_and_next(Token::Comma);
      }
    };
    self.check_token_and_next(Token::RightBracket);

    Expression::Array(ArrayLiteral {
      elements
    })
  }
  // 解析对象字面量
  // https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#prod-ObjectLiteral
  fn parse_object_literal(&mut self) -> Expression {
    self.check_token_and_next(Token::LeftBrace);
    let mut properties: Vec<PropertyAssignment>= vec![];
    while self.token != Token::RightBrace && self.token != Token::EOF {
      // 属性名
      let mut property_name = self.parse_object_property_name();
      if let Expression::Unknown = property_name {
        break;
      }
      
      // 解析值
      let initializer = match self.token {
         // 如果是 :
        Token::Colon => {
          self.next();
          self.parse_expression()
        },
        // TODO: Shorthand method names (ES2015)
        _ => {
          // Shorthand property names (ES2015)
          if let Expression::Identifier(property) = property_name.clone() {
            Expression::Identifier(IdentifierLiteral { literal: property.literal } )
          } else {
            // TODO: throw error
            Expression::Unknown
          }
        },
      };

      if let Expression::Identifier(property) = property_name {
        property_name = Expression::String(StringLiteral {
          literal: property.literal.clone(),
          value: property.literal,
        });
      }
      
      properties.push(PropertyAssignment {
        name: Box::new(property_name),
        initializer: Box::new(initializer),
      });
      // 跳过逗号
      if self.token == Token::Comma {
        self.next();
      }
    }
    self.check_token_and_next(Token::RightBrace);
    Expression::Object(ObjectLiteral {
      properties,
    })
  }

  fn parse_object_property_name(&mut self) -> Expression {
    let property_name_literal = self.literal.clone();
    match self.token {
      Token::Identifier => {
        self.next();
        Expression::Identifier(IdentifierLiteral {
          literal: property_name_literal,
        })
      },
      Token::String => {
        let str_len = property_name_literal.len();
        let slice = String::from(&self.literal[1..str_len-1]);
        self.next();
        Expression::String(StringLiteral {
          literal: property_name_literal,
          value: slice,
        })
      },
      Token::Number => {
        let number_value = self.parse_number_literal_expression();
        self.next();
        Expression::Number(NumberLiteral { literal: property_name_literal, value: number_value })
      },
      // Computed property names (ES2015)
      Token::LeftBracket => {
        self.next();
        let key = self.parse_expression();
        self.check_token_and_next(Token::RightBracket);
        Expression::ComputedPropertyName(ComputedPropertyName { expression: Box::new(key) })
      },
      _ => Expression::Unknown
    }
  }

  // 解析参数
  fn parse_arguments(&mut self) -> Vec<Expression> {
    self.check_token_and_next(Token::LeftParenthesis);
    let mut arguments:Vec<Expression> = vec![];
    while self.token != Token::RightParenthesis && self.token != Token::EOF {
      arguments.push(self.parse_expression());
      if self.token != Token::Comma {
				break
			}
      self.next()
    }
    arguments
  }

  fn parse_number_literal_expression(&mut self) -> f64 {
    // 检测是否是 float
    self.literal.parse::<f64>().unwrap()
  }

  // 解析左结合表达式
  fn parse_left_associate_expression<F: Fn(&mut AST)-> Expression>(&mut self, tokens: Vec<Token>, next: F) -> Expression {
    let mut left = next(self);
    loop {
      // 向左结合
      if tokens.contains(&self.token) {
        let operator = self.token.clone();
        // 跳过当前的运算符
        self.next();
        let right = next(self);
        left = Expression::Binary(BinaryExpression{
          left: Box::new(left),
          operator,
          right: Box::new(right)
        });
      } else {
        break;
      }
    }
    return left;
  }

  // 跳过空白字符
  fn skip_white_space(&mut self) {
    loop {
      match self.char {
        //  跳过空格
          ' '| '\t' => {
            self.read();
          },
          // 这里不处理换行，换行在 scan 的时候处理，因为要处理是否自动添加分号
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
  pub body: Vec<Statement>,
  pub declarations: Vec<Declaration>
}


impl Program {}

#[derive(Debug, Clone)]
pub struct ASTScope {
  pub parent: Option<Box<ASTScope>>,
  pub declarations: Vec<Declaration>
}

impl  ASTScope {
    fn new() -> ASTScope {
      ASTScope {
        parent: None,
        declarations: vec![],
      }
    }

    fn declare(&mut self, declaration: Declaration) {
      self.declarations.push(declaration);
    }
}