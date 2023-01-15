use std::{fmt, cell::{RefCell, RefMut}, borrow::BorrowMut, rc::{Rc, Weak}};

use crate::{ast_token::Token, value::Value, builtins::{object::Object, global::Global}};

#[derive(Clone)]
pub enum Statement {
  Var(VariableDeclarationStatement),
  Function(FunctionDeclaration),
  Class(ClassDeclaration),
  Block(BlockStatement),
  Return(ReturnStatement),
  Expression(ExpressionStatement),
  BuiltinFunction(BuiltinFunction),
  Unknown,
}

impl PartialEq for Statement {
  fn eq(&self, other: &Statement) -> bool {
    match (self, other) {
      (Statement::Var(a), Statement::Var(b)) => *a == *b,
      (Statement::Function(a), Statement::Function(b)) => *a == *b,
      (Statement::Class(a), Statement::Class(b)) => *a == *b,
      (Statement::Block(a), Statement::Block(b)) => *a == *b,
      (Statement::Return(a), Statement::Return(b)) => *a == *b,
      (Statement::Expression(a), Statement::Expression(b)) => *a == *b,
      _ => false,
    }
  }
}

impl fmt::Debug for Statement {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "statement")
  }
}

#[derive(Debug,Clone, PartialEq)]
pub enum Expression {
  Var(VariableDeclaration),
  Assign(AssignExpression),
  Binary(BinaryExpression),
  Conditional(ConditionalExpression),
  PropertyAccess(PropertyAccessExpression),
  ElementAccess(ElementAccessExpression),
  Call(CallExpression),
  PrefixUnary(PrefixUnaryExpression),
  PostfixUnary(PostfixUnaryExpression),
  Group(GroupExpression),
  Identifier(IdentifierLiteral),
  Number(NumberLiteral),
  String(StringLiteral),
  Keyword(Keywords),
  Object(ObjectLiteral),
  Function(FunctionDeclaration),
  // for class
  Class(ClassDeclaration),
  Constructor(FunctionDeclaration),
  ClassMethod(ClassMethodDeclaration),
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
pub enum VariableFlag {
  Var,
  Let,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarationStatement {
  pub list: Vec<Expression>,
  pub flag: VariableFlag,
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
pub struct ClassDeclaration {
  pub name: IdentifierLiteral,
  pub members: Vec<Expression>,
  // 继承
  pub heritage: Option<Box<ClassDeclaration>>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMethodDeclaration {
  pub name: IdentifierLiteral,
  pub modifiers: Vec<Token>,
  pub method: Box<FunctionDeclaration>,
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
#[derive(Debug, Clone, PartialEq)]
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


// 前置一元运算符表达式
#[derive(Debug, Clone, PartialEq)]
pub struct PrefixUnaryExpression {
  pub operand: Box<Expression>,
  pub operator: Token,
}

// 后置一元运算符表达式 ++ --
#[derive(Debug, Clone, PartialEq)]
pub struct PostfixUnaryExpression {
  pub operand: Box<Expression>,
  pub operator: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupExpression {
  pub expression: Box<Expression>
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
pub struct VariableDeclaration {
  pub name: String,
  pub initializer: Box<Expression>
}



pub type BuiltinFunction = fn(&mut CallContext, Vec<Value>) -> Value;
pub struct CallContext<'a> {
  pub global: &'a Global,
  pub this: Weak<RefCell<Object>>
}