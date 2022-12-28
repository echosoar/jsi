use crate::ast_token::Token;

#[derive(Debug)]
pub enum Statement {
  Let(LetVariableStatement),
  Expression(ExpressionStatement),
  Unknown,
}

#[derive(Debug,Clone)]
pub enum Expression {
  Let(LetVariableDeclaration),
  // Assign(AssignExpression),
  Binary(BinaryExpression),
  PropertyAccess(PropertyAccessExpression),
  Identifier(IdentifierLiteral),
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
pub struct ExpressionStatement {
  pub expression: Expression
}
#[derive(Debug)]
pub struct AssignExpression {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
}

// . 表达式
#[derive(Debug)]
pub struct BinaryExpression {
  pub left: Box<Expression>,
  pub operator: Token,
  pub right: Box<Expression>,
}


impl Clone for BinaryExpression {
  fn clone(&self) -> Self {
    BinaryExpression {
      left: self.left.clone(),
      operator: self.operator.clone(),
      right: self.right.clone(),
    }
  }
}

// . 表达式
#[derive(Debug)]
pub struct PropertyAccessExpression {
  pub expression: Box<Expression>,
  pub name: IdentifierLiteral,
}

impl Clone for PropertyAccessExpression {
  fn clone(&self) -> Self {
    PropertyAccessExpression {
      expression: self.expression.clone(),
      name: self.name.clone(),
    }
  }
}

#[derive(Debug)]
pub struct  IdentifierLiteral {
  pub literal: String,
}

impl Clone for IdentifierLiteral {
  fn clone(&self) -> Self {
    IdentifierLiteral {
      literal: self.literal.clone(),
    }
  }
}

#[derive(Debug)]
pub struct  NumberLiteral {
  pub literal: String,
  pub value: f64,
}


impl Clone for NumberLiteral {
  fn clone(&self) -> Self {
    NumberLiteral {
      literal: self.literal.clone(),
      value: self.value,
    }
  }
}


#[derive(Debug)]
pub struct  StringLiteral {
  pub literal: String,
  pub value: String
}

impl Clone for StringLiteral {
  fn clone(&self) -> Self {
    StringLiteral {
      literal: self.literal.clone(),
      value: self.value.clone(),
    }
  }
}

#[derive(Debug)]
pub struct LetVariableDeclaration {
  pub name: String,
  pub initializer: Box<Expression>
}

impl Clone for LetVariableDeclaration {
  fn clone(&self) -> Self {
    LetVariableDeclaration {
      name: self.name.clone(),
      initializer: self.initializer.clone(),
    }
  }
}