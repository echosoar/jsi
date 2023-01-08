
// 关键字
// ref: https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Lexical_grammar#%E5%85%B3%E9%94%AE%E5%AD%97
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
  // es6 keywords
  Break,
  Case,
  Catch,
  Class,
  Const,
  Continue,
  Debugger,
  Default,
  Delete,
  Do,
  Else,
  Export,
  Extends,
  Finally,
  For,
  Function,
  If,
  Import,
  In,
  Instanceof,
  New,
  Return,
  Super,
  Switch,
  This,
  Throw,
  Try,
  Typeof,
  Var,
  Void,
  While,
  With,
  Yield,
  // 仅在严格模式下作为关键字
  Implements,
  Interface,
  Let,
  Package,
  Private,
  Protected,
  Public,
  Static,

  // 字面量 literal
  Undefined,
  Null,
  True,
  False,
  // 类型标识符
  Number,
  String,
  // 普通标识符
  Identifier,
  // 运算符 operators
  // 标点符号
  Plus, // "+"
	Subtract, // "-"
	Multiply, // "*"
	Slash, // "/"
	Remainder, // "%"
	And, // "&"
	Or, // "|"
	ExclusiveOr, // "^"
	ShiftLeft, // "<<"
	ShiftRight, // ">>"
	UnsignedShiftRight, // ">>>"
	AndNot, // "&^"
	AddAssign, // "+="
	SubtractAssign, // "-="
	MultiplyAssign, // "*="
	QuotientAssign, // "/="
	RemainderAssign, // "%="
	AndAssign, // "&="
	OrAssign, // "|="
	ExclusiveOrAssign, // "^="
	ShiftLeftAssign, // "<<="
	ShiftRightAssign, // ">>="
	UnsignedShiftRightAssign, // ">>>="
	AndNotAssign, // "&^="
	LogicalAnd, // "&&"
	LogicalOr, // "||"
	Increment, // "++"
	Decrement, // "--"
	Equal, // "=="
	StrictEqual, // "==="
	Less, // "<"
	Greater, // ">"
	Assign, // "="
	Not, // "!"
	BitwiseNot, // "~"
	NotEqual, // "!="
	StrictNotEqual, // "!=="
	LessOrEqual, // "<="
	GreaterOrEqual, // ">="
	LeftParenthesis, // "("
	LeftBracket, // "["
	LeftBrace, // "{"
	Comma, // ",",
	Period, // "."
	RightParenthesis, // ")"
	RightBracket, // "]"
	RightBrace, // "}"
	Semicolon, // ";"
	Colon, // ":"
	QuestionMark, // "?"
  Exponentiation, // "**" ES2017
	NullishCoalescing, // "??" ES2020
	OptionalChaining, // "?." ES2020
  // not keyword
  ILLEGAL,
  // 结尾
  EOF,
}


pub fn get_token_keyword(literal: &String, is_strict: bool) -> Token {
  let str = literal.as_str();
  match str {
    "break" => Token::Break,
    "case" => Token::Case,
    "catch" => Token::Catch,
    "class" => Token::Class,
    "const" => Token::Const,
    "continue" => Token::Continue,
    "debugger" => Token::Debugger,
    "default" => Token::Default,
    "delete" => Token::Delete,
    "do" => Token::Do,
    "else" => Token::Else,
    "export" => Token::Export,
    "extends" => Token::Extends,
    "finally" => Token::Finally,
    "for" => Token::For,
    "function" => Token::Function,
    "if" => Token::If,
    "import" => Token::Import,
    "in" => Token::In,
    "instanceof" => Token::Instanceof,
    "new" => Token::New,
    "return" => Token::Return,
    "super" => Token::Super,
    "switch" => Token::Switch,
    "this" => Token::This,
    "throw" => Token::Throw,
    "try" => Token::Try,
    "typeof" => Token::Typeof,
    "var" => Token::Var,
    "void" => Token::Void,
    "while" => Token::While,
    "with" => Token::With,
    "yield" => Token::Yield,
    _ => {
      if is_strict {
        match str {
          "implements" => Token::Implements,
          "interface" => Token::Interface,
          "let" => Token::Let,
          "package" => Token::Package,
          "private" => Token::Private,
          "protected" => Token::Protected,
          "public" => Token::Public,
          "static" => Token::Static,
          _ => Token::ILLEGAL,
        }
      } else {
        Token::ILLEGAL
      }
    }
  }
}

pub fn get_token_literal(literal: &String) -> Token {
  match literal.as_str() {
    "null" => Token::Null,
    "true" => Token::True,
    "false" => Token::False,
    _ => Token::ILLEGAL
  }
}