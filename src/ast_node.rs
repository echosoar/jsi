use crate::ast_token::Token;



#[derive(Debug)]
pub enum Expression {
  // Let(LetVariableStatement),
  // Assign(AssignExpression),
  Number(NumberLiteral),
}

#[derive(Debug)]
pub struct LetVariableStatement {
  pub name: String,
}
#[derive(Debug)]
pub struct AssignExpression {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
}

#[derive(Debug)]
pub struct  NumberLiteral {

}