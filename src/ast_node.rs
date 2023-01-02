use crate::ast_token::Token;

#[derive(Debug)]
pub enum Statement {
  Let(LetVariableStatement),
  Function(FunctionDeclarationStatement),
  Block(BlockStatement),
  Return(ReturnStatement),
  Expression(ExpressionStatement),
  Unknown,
}

#[derive(Debug,Clone)]
pub enum Expression {
  Let(LetVariableDeclaration),
  // Assign(AssignExpression),
  Binary(BinaryExpression),
  Conditional(ConditionalExpression),
  PropertyAccess(PropertyAccessExpression),
  Call(CallExpression),
  Identifier(IdentifierLiteral),
  Number(NumberLiteral),
  String(StringLiteral),
  Keyword(Keywords),
  Unknown,
}

#[derive(Debug,Clone)]
pub enum Keywords {
  False,
  True,
  Null,
  Undefined,
}

#[derive(Debug)]
pub struct LetVariableStatement {
  pub list: Vec<Expression>
}

#[derive(Debug)]
pub struct FunctionDeclarationStatement {
  pub name: IdentifierLiteral,
  pub parameters: Vec<Parameter>,
  pub body: BlockStatement,
}


#[derive(Debug)]
pub struct BlockStatement {
  pub statements: Vec<Statement>
}

#[derive(Debug)]
pub struct ReturnStatement {
  pub expression: Expression
}

#[derive(Debug, Clone)]
pub struct Parameter {
  pub name: IdentifierLiteral,
  pub initializer: Box<Expression>
}

#[derive(Debug)]
pub struct ExpressionStatement {
  pub expression: Expression
}
#[derive(Debug, Clone)]
pub struct AssignExpression {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
}

// 条件表达式
#[derive(Debug, Clone)]
pub struct ConditionalExpression {
  pub condition: Box<Expression>,
  pub when_true: Box<Expression>,
  pub when_false: Box<Expression>,
}

// . 表达式
#[derive(Debug, Clone)]
pub struct BinaryExpression {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
}
// 方法调用表达式
#[derive(Debug, Clone)]
pub struct CallExpression {
  pub expression: Box<Expression>,
  pub arguments: Vec<Expression>,
}

// . 属性访问表达式
#[derive(Debug, Clone)]
pub struct PropertyAccessExpression {
  pub expression: Box<Expression>,
  pub name: IdentifierLiteral,
}

#[derive(Debug, Clone)]
pub struct  IdentifierLiteral {
  pub literal: String,
}

#[derive(Debug, Clone)]
pub struct  NumberLiteral {
  pub literal: String,
  pub value: f64,
}


#[derive(Debug, Clone)]
pub struct  StringLiteral {
  pub literal: String,
  pub value: String
}

#[derive(Debug, Clone)]
pub struct LetVariableDeclaration {
  pub name: String,
  pub initializer: Box<Expression>
}