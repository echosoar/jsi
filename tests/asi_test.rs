use jsi::{JSI,ast_node::{Statement, ExpressionStatement, Expression, IdentifierLiteral, PrefixUnaryExpression, AssignExpression, BinaryExpression, FunctionDeclaration, BlockStatement, ReturnStatement, Keywords}, ast_token::Token, value::Value};

#[test]
// test262: https://github.com/tc39/test262/blob/main/test/language/asi/S7.9.2_A1_T5.js
fn asi_increment_after_identifier() {
  let mut jsi_vm = JSI::new();
  let program = jsi_vm.parse(String::from("a = b\n
  ++c"));
  assert_eq!(program.body, vec![
    Statement::Expression(ExpressionStatement { // a + b
      expression: Expression::Assign(AssignExpression {
        left: Box::new(Expression::Identifier(IdentifierLiteral { literal: String::from("a") })),
        operator: Token::Assign,
        right: Box::new(Expression::Identifier(IdentifierLiteral { literal: String::from("b") })),
      }),
    }),
    Statement::Expression(ExpressionStatement { // ++c
      expression: Expression::PrefixUnary(PrefixUnaryExpression {
        operand: Box::new(Expression::Identifier(IdentifierLiteral { literal: String::from("c") })),
        operator: Token::Increment,
      })
    })
  ]);
}

#[test]
// test262: https://github.com/tc39/test262/blob/main/test/language/asi/S7.9.2_A1_T4.js
fn asi_after_keyword_return() {
  let mut jsi_vm = JSI::new();
  let program = jsi_vm.parse(String::from("function test(){\n
    return\n
    a+b\n
  }"));
  assert_eq!(program.body, vec![
    Statement::Function(FunctionDeclaration { // a + b
      is_anonymous: false,
      name: IdentifierLiteral { literal: String::from("test") },
      parameters: vec![],
      body: BlockStatement {
        statements: vec![
          Statement::Return(ReturnStatement {
            expression: Expression::Keyword(Keywords::Undefined)
          }),
          Statement::Expression(ExpressionStatement { // ++c
            expression: Expression::Binary(BinaryExpression {
              left: Box::new(Expression::Identifier(IdentifierLiteral { literal: String::from("a") })),
              operator: Token::Plus,
              right: Box::new(Expression::Identifier(IdentifierLiteral { literal: String::from("b") })),
            })
          })
        ]
      },
      declarations: vec![],
    }),
  ]);
}



#[test]
// test262: https://github.com/tc39/test262/blob/main/test/language/asi/S7.9.2_A1_T7.js
fn asi_not_insert_after_identifier() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run(String::from("function c (a){
    return 2*a;
  }
  
  var a=1,b=2,d=4,e=5;
  
  a=b+c
  (d+e)"));
  assert_eq!(value,Value::Number(20f64));
}