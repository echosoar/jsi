use jsi::{JSI,ast_node::{Statement, ExpressionStatement, Expression, IdentifierLiteral, PrefixUnaryExpression, AssignExpression}, ast_token::Token};

#[test]
fn ast_lexer_insert_semicolon() {
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