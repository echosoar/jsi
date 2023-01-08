use jsi::{JSI,ast_node::{Statement, ExpressionStatement, Expression, BinaryExpression, NumberLiteral}, ast_token::Token};

struct TokenCheck {
  pub oper: String,
  pub token: Token
}

#[test]
fn ast_lexer_token() {
  let token_list = vec![
    TokenCheck { oper: String::from("+"), token: Token::Plus },
    TokenCheck { oper: String::from("-"), token: Token::Subtract },
    TokenCheck { oper: String::from("*"), token: Token::Multiply },
    TokenCheck { oper: String::from("/"), token: Token::Slash },
    TokenCheck { oper: String::from("%"), token: Token::Remainder },
  ];
  let mut jsi_vm = JSI::new();
  for token in token_list.iter() {
    let mut code = String::from("1");
    code.push_str(token.oper.as_str());
    code.push_str("1;");
    let program = jsi_vm.parse(code);
    assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
      expression: Expression::Binary(BinaryExpression {
        left: Box::new(Expression::Number(NumberLiteral{ literal: String::from("1"), value: 1f64 })),
        operator: token.token.clone(),
        right: Box::new(Expression::Number(NumberLiteral{ literal: String::from("1"), value: 1f64 })),
      })
    })]);
  }
}