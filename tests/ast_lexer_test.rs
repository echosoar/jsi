use jsi::{JSI,ast_node::{Statement, ExpressionStatement, Expression, BinaryExpression, NumberLiteral, IdentifierLiteral, PostfixUnaryExpression, PrefixUnaryExpression}, ast_token::Token};

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
    TokenCheck { oper: String::from("**"), token: Token::Exponentiation },
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


#[test]
fn ast_lexer_prefix_unary_token() {
  let token_list = vec![
    TokenCheck { oper: String::from("!"), token: Token::Not },
    TokenCheck { oper: String::from("~"), token: Token::BitwiseNot },
    TokenCheck { oper: String::from("+"), token: Token::Plus },
    TokenCheck { oper: String::from("-"), token: Token::Subtract },
    TokenCheck { oper: String::from("++"), token: Token::Increment },
    TokenCheck { oper: String::from("--"), token: Token::Decrement },
    TokenCheck { oper: String::from("typeof"), token: Token::Typeof },
    TokenCheck { oper: String::from("void"), token: Token::Void },
    TokenCheck { oper: String::from("delete"), token: Token::Delete },
    TokenCheck { oper: String::from("await"), token: Token::Await },
  ];
  let mut jsi_vm = JSI::new();
  for token in token_list.iter() {
    let mut code = String::from("");
    code.push_str(token.oper.as_str());
    code.push_str(" i;");
    let program = jsi_vm.parse(code);
    assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
      expression: Expression::PrefixUnary(PrefixUnaryExpression {
        operand: Box::new(Expression::Identifier(IdentifierLiteral{ literal: String::from("i") })),
        operator: token.token.clone(),
      })
    })]);
  }
}


#[test]
fn ast_lexer_postfix_unary_token() {
  let token_list = vec![
    TokenCheck { oper: String::from("++"), token: Token::Increment },
    TokenCheck { oper: String::from("--"), token: Token::Decrement },
  ];
  let mut jsi_vm = JSI::new();
  for token in token_list.iter() {
    let mut code = String::from("i");
    code.push_str(token.oper.as_str());
    code.push_str(";");
    let program = jsi_vm.parse(code);
    assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
      expression: Expression::PostfixUnary(PostfixUnaryExpression {
        operand: Box::new(Expression::Identifier(IdentifierLiteral{ literal: String::from("i") })),
        operator: token.token.clone(),
      })
    })]);
  }
}