use crate::ast_token::Token;

#[derive(Debug)]
pub enum Statement {
  Let(LetVariableStatement),
  Unknown,
}

#[derive(Debug)]
pub enum Expression {
  Let(LetVariableDeclaration),
  // Assign(AssignExpression),
  Number(NumberLiteral),
  String(StringLiteral),
  Undefined,
  Unknown,
}

#[derive(Debug)]
pub struct LetVariableStatement {
  pub list: Vec<Expression>
}
#[derive(Debug)]
pub struct AssignExpression {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
}

#[derive(Debug)]
pub struct  NumberLiteral {
  pub literal: String,
  pub value: f64,
}

#[derive(Debug)]
pub struct  StringLiteral {
  pub literal: String,
  pub value: String
}

#[derive(Debug)]
pub struct LetVariableDeclaration {
  pub name: String,
  pub initializer: Box<Expression>
}