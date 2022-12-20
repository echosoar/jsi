
// 关键字
// ref: https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Lexical_grammar#%E5%85%B3%E9%94%AE%E5%AD%97
#[derive(Debug, PartialEq)]
pub enum Token {
  // es6 keywords
  BREAK,
  CASE,
  CATCH,
  CLASS,
  CONST,
  CONTINUE,
  DEBUGGER,
  DEFAULT,
  DELETE,
  DO,
  ELSE,
  EXPORT,
  EXTENDS,
  FINALLY,
  FOR,
  FUNCTION,
  IF,
  IMPORT,
  IN,
  INSTANCEOF,
  NEW,
  RETURN,
  SUPER,
  SWITCH,
  THIS,
  THROW,
  TRY,
  TYPEOF,
  VAR,
  VOID,
  WHILE,
  WITH,
  YIELD,
  // 仅在严格模式下作为关键字
  IMPLEMENTS,
  INTERFACE,
  LET,
  PACKAGE,
  PRIVATE,
  PROTECTED,
  PUBLIC,
  STATIC,

  // 字面量 literal
  NULL,
  TRUE,
  FALSE,
  // 类型标识符
  NUMBER,
  // 普通标识符
  IDENTIFIER,
  // 运算符 operators
  ASSIGN,
  // not keyword
  ILLEGAL,
  // 结尾
  EOF,
}

pub fn get_token_keyword(literal: &String, is_strict: bool) -> Token {
  let str = literal.as_str();
  match str {
    "break" => Token::BREAK,
    "case" => Token::CASE,
    "catch" => Token::CATCH,
    "class" => Token::CLASS,
    "const" => Token::CONST,
    "continue" => Token::CONTINUE,
    "debugger" => Token::DEBUGGER,
    "default" => Token::DEFAULT,
    "delete" => Token::DELETE,
    "do" => Token::DO,
    "else" => Token::ELSE,
    "export" => Token::EXPORT,
    "extends" => Token::EXTENDS,
    "finally" => Token::FINALLY,
    "for" => Token::FOR,
    "function" => Token::FUNCTION,
    "if" => Token::IF,
    "import" => Token::IMPORT,
    "in" => Token::IN,
    "instanceof" => Token::INSTANCEOF,
    "new" => Token::NEW,
    "return" => Token::RETURN,
    "super" => Token::SUPER,
    "switch" => Token::SWITCH,
    "this" => Token::THIS,
    "throw" => Token::THROW,
    "try" => Token::TRY,
    "typeof" => Token::TYPEOF,
    "var" => Token::VAR,
    "void" => Token::VOID,
    "while" => Token::WHILE,
    "with" => Token::WITH,
    "yield" => Token::YIELD,
    _ => {
      if is_strict {
        return match str {
          "implements" => Token::IMPLEMENTS,
          "interface" => Token::INTERFACE,
          "let" => Token::LET,
          "package" => Token::PACKAGE,
          "private" => Token::PRIVATE,
          "protected" => Token::PROTECTED,
          "public" => Token::PUBLIC,
          "static" => Token::STATIC,
          _ => Token::ILLEGAL,
        }
      }
      return Token::ILLEGAL
    }
  }
}

pub fn get_token_literal(literal: &String) -> Token {
  match literal.as_str() {
    "null" => Token::NULL,
    "true" => Token::TRUE,
    "false" => Token::FALSE,
    _ => Token::ILLEGAL
  }
}