use jsi::{JSI, ast_node::{Expression, Statement, ObjectLiteral, ExpressionStatement, PropertyAssignment, IdentifierLiteral, NumberLiteral, StringLiteral, Keywords, BinaryExpression}, ast_token::Token};

#[test]
fn ast_base() {
  let mut jsi = JSI::new();
  let program = jsi.parse(String::from("{a: 123, b: '123', [1 + 'a']: false}"));
  assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
    expression: Expression::Object(ObjectLiteral {
      properties: vec![
        PropertyAssignment{
          name: Box::new(Expression::Identifier(IdentifierLiteral { literal: String::from("a")})),
          initializer: Box::new(Expression::Number(NumberLiteral { literal: String::from("123"), value: 123f64 })),
        },
        PropertyAssignment{
          name: Box::new(Expression::Identifier(IdentifierLiteral { literal: String::from("b")})),
          initializer: Box::new(Expression::String(StringLiteral { literal: String::from("'123'"), value: String::from("123") })),
        },
        PropertyAssignment{
          name: Box::new(Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Number(NumberLiteral { literal: String::from("1"), value: 1f64 })),
            operator: Token::Plus,
            right: Box::new(Expression::String(StringLiteral { literal: String::from("'a'"), value: String::from("a") })),
          })),
          initializer: Box::new(Expression::Keyword(Keywords::False)),
        }
      ]
    })
  })]);
}

#[test]
fn ast_with_child_object() {
  let mut jsi = JSI::new();
  let program = jsi.parse(String::from("{obj: { x: false}}"));
  assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
    expression: Expression::Object(ObjectLiteral {
      properties: vec![
        PropertyAssignment{
          name: Box::new(Expression::Identifier(IdentifierLiteral { literal: String::from("obj")})),
          initializer: Box::new(Expression::Object(ObjectLiteral {
            properties: vec![
              PropertyAssignment{
                name: Box::new(Expression::Identifier(IdentifierLiteral { literal: String::from("x")})),
                initializer: Box::new(Expression::Keyword(Keywords::False)),
              },
            ]
          })),
        },
      ]
    })
  })]);
}

#[test]
fn run_object() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let a = 123;
    let obj = { a, [a]: a};
    obj;
  "));
  println!("result: {:?}", result);
  
}