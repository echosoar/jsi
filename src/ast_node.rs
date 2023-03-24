use std::rc::Rc;
use std::{fmt, cell::RefCell, rc::Weak};
use crate::context::{Context};
use crate::{ast_token::Token, value::Value, builtins::{object::Object}, error::JSIResult};

#[derive(Clone)]
pub enum Statement {
  // A -> Z
  Block(BlockStatement),
  Break(BreakStatement),
  BuiltinFunction(BuiltinFunction),
  Class(ClassDeclaration),
  Continue(ContinueStatement),
  Expression(ExpressionStatement),
  For(ForStatement),
  Function(FunctionDeclaration),
  If(IfStatement),
  Label(LabeledStatement),
  Return(ReturnStatement),
  Switch(SwitchStatement),
  Throw(ThrowStatement),
  Try(TryCatchStatement),
  Unknown, // 未知
  Var(VariableDeclarationStatement),
  While(ForStatement),
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
    let stype = match self {
      Statement::Block(_) => { "block"},
      Statement::Break(_) => { "break"},
      Statement::BuiltinFunction(_) => { "builtin function"},
      Statement::Class(_) => { "class"},
      Statement::Continue(_) => { "continue"},
      Statement::Expression(_) => { "expression"},
      Statement::For(_) => { "for"},
      Statement::Function(_) => { "function"},
      Statement::If(_) => { "if"},
      Statement::Label(_) => { "label"},
      Statement::Return(_) => { "return"},
      Statement::Switch(_) => { "switch"},
      Statement::Try(_) => { "try"},
      Statement::Var(_) => { "var"},
      Statement::While(_) => { "while"},
      _ => {
        "other"
      },
    };
    write!(f, "{}", stype)
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
  Array(ArrayLiteral),
  Function(FunctionDeclaration),
  New(NewExpression),
  Sequence(SequenceExpression),
  TemplateLiteral(TemplateLiteralExpression),
  // {[a]: 12}
  ComputedPropertyName(ComputedPropertyName),
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
  This,
}

impl Keywords {
    pub fn to_string(&self) -> String {
      match &self {
        Keywords::False => String::from("false"),
        Keywords::True => String::from("true"),
        Keywords::Null => String::from("null"),
        Keywords::Undefined => String::from("undefined"),
        Keywords::This => String::from("this"),
      }
    }
}

#[derive(Debug,Clone, PartialEq)]
pub enum Declaration {
  Function(FunctionDeclaration)
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableFlag {
  Var,
  Let,
  Const,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarationStatement {
  pub list: Vec<Expression>,
  pub flag: VariableFlag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
  pub condition: Expression,
  pub then_statement: Box<Statement>,
  pub else_statement: Box<Statement>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStatement {
  pub condition: Expression,
   pub clauses: Vec<CaseClause>,
   pub default_index: i32
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseClause {
  pub condition: Option<Expression>,
  pub statements: Vec<Statement>
}


#[derive(Debug, Clone, PartialEq)]
pub struct LabeledStatement {
  pub label: IdentifierLiteral,
  pub statement: Box<Statement>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
  pub initializer: Box<Statement>,
  pub condition: Expression,
  pub incrementor: Expression,
  pub statement: Box<Statement>,
  pub post_judgment: bool,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ThrowStatement {
  pub expression: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryCatchStatement {
  pub body: BlockStatement,
  pub catch: Option<CatchClause>,
  pub finally: Option<BlockStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
  pub declaration: Option<IdentifierLiteral>,
  pub body: BlockStatement
}



#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
  pub is_anonymous: bool,
  pub name: IdentifierLiteral,
  pub parameters: Vec<Parameter>,
  pub body: BlockStatement,
  pub declarations: Vec<Declaration>,
}

// ES2015 Computed Property Name
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedPropertyName {
  pub expression: Box<Expression>
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpression {
  pub expression: Box<Expression>,
  pub arguments: Vec<Expression>,
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
pub struct BreakStatement {
  pub label: Option<IdentifierLiteral>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContinueStatement {
  pub label: Option<IdentifierLiteral>
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

// 字符串模板表达式
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceExpression {
  pub expressions: Vec<Expression>
}

// 字符串模板表达式
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateLiteralExpression {
  pub spans: Vec<Expression>
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
pub struct ArrayLiteral {
  pub elements: Vec<Expression>
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



pub type BuiltinFunction = fn(&mut CallContext, Vec<Value>) -> JSIResult<Value>;
pub struct CallContext<'a> {
  // 全局对象，globalThis
  pub ctx: &'a mut Context,
  // 调用时的 this
  pub this: Weak<RefCell<Object>>,
  // 引用，调用的发起方，比如  a.call()，reference 就是 a
  // 当直接调用 call() 的时候，refererce 是 None
  pub reference: Option<Weak<RefCell<Object>>>,
}

impl <'a>CallContext<'a> {
  pub fn call_function(&mut self, function_define: Rc<RefCell<Object>>, call_this: Option<Value>, reference: Option<Weak<RefCell<Object>>>, arguments: Vec<Value>) -> JSIResult<Value> {
    self.ctx.call_function_object(function_define, call_this,reference,  arguments)
  }
}

#[derive(Debug, Clone)]
pub enum ClassType {
  Object,
  Array,
  Function,
  String,
  Boolean,
  Number,
  Null,
  //
  Error,
}

impl  ClassType {
  pub fn to_string(&self) -> String {
    match self {
      Self::Object => String::from("Object"),
      Self::Array => String::from("Array"),
      Self::Function => String::from("Function"),
      Self::String => String::from("String"),
      Self::Boolean => String::from("Boolean"),
      Self::Number => String::from("Number"),
      Self::Null => String::from("Null"),
      Self::Error => String::from("Error"),
    }
  }
}