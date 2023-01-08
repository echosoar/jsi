use crate::ast_token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
  Let(LetVariableStatement),
  Function(FunctionDeclaration),
  Block(BlockStatement),
  Return(ReturnStatement),
  Expression(ExpressionStatement),
  Unknown,
}

#[derive(Debug,Clone, PartialEq)]
pub enum Expression {
  Let(LetVariableDeclaration),
  // Assign(AssignExpression),
  Binary(BinaryExpression),
  Conditional(ConditionalExpression),
  PropertyAccess(PropertyAccessExpression),
  ElementAccess(ElementAccessExpression),
  Call(CallExpression),
  Identifier(IdentifierLiteral),
  Number(NumberLiteral),
  String(StringLiteral),
  Keyword(Keywords),
  Object(ObjectLiteral),
  Function(FunctionDeclaration),
  Unknown,
}

#[derive(Debug,Clone, PartialEq)]
pub enum Keywords {
  False,
  True,
  Null,
  Undefined,
  X,
}

#[derive(Debug,Clone, PartialEq)]
pub enum Declaration {
  Function(FunctionDeclaration)
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetVariableStatement {
  pub list: Vec<Expression>
}



#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
  pub is_anonymous: bool,
  pub name: IdentifierLiteral,
  pub parameters: Vec<Parameter>,
  pub body: BlockStatement,
  pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatement {
  pub statements: Vec<Statement>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
  pub expression: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
  pub name: IdentifierLiteral,
  pub initializer: Box<Expression>
}

#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpression {
  pub condition: Box<Expression>,
  pub when_true: Box<Expression>,
  pub when_false: Box<Expression>,
}

// . 表达式
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpression {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
}
// 方法调用表达式
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression {
  pub expression: Box<Expression>,
  pub arguments: Vec<Expression>,
}

// . 属性访问表达式
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyAccessExpression {
  pub expression: Box<Expression>,
  pub name: IdentifierLiteral,
}


// [] 属性访问表达式
#[derive(Debug, Clone, PartialEq)]
pub struct ElementAccessExpression {
  pub expression: Box<Expression>,
  pub argument: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierLiteral {
  pub literal: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct  NumberLiteral {
  pub literal: String,
  pub value: f64,
}


#[derive(Debug, Clone, PartialEq)]
pub struct  StringLiteral {
  pub literal: String,
  pub value: String
}

#[derive(Debug, Clone, PartialEq)]
pub struct  ObjectLiteral {
  pub properties: Vec<PropertyAssignment>
}


#[derive(Debug, Clone, PartialEq)]
pub struct  PropertyAssignment {
  pub name: Box<Expression>,
  pub initializer: Box<Expression>
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetVariableDeclaration {
  pub name: String,
  pub initializer: Box<Expression>
}