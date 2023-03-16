use jsi::{JSI,ast_node::{Statement, ExpressionStatement, Expression, BinaryExpression, NumberLiteral, IdentifierLiteral, PostfixUnaryExpression, PrefixUnaryExpression, AssignExpression, GroupExpression, ConditionalExpression}, ast_token::Token};

struct TokenCheck {
  pub oper: String,
  pub token: Token
}

#[test]
fn ast_lexer_binary_token() {
  let token_list = vec![
    TokenCheck { oper: String::from("+"), token: Token::Plus },
    TokenCheck { oper: String::from("-"), token: Token::Subtract },
    TokenCheck { oper: String::from("*"), token: Token::Multiply },
    TokenCheck { oper: String::from("/"), token: Token::Slash },
    TokenCheck { oper: String::from("%"), token: Token::Remainder },
    TokenCheck { oper: String::from("**"), token: Token::Exponentiation },
    TokenCheck { oper: String::from("??"), token: Token::NullishCoalescing },
    TokenCheck { oper: String::from("||"), token: Token::LogicalOr },
    TokenCheck { oper: String::from("&&"), token: Token::LogicalAnd },
    TokenCheck { oper: String::from("&"), token: Token::And },
    TokenCheck { oper: String::from("|"), token: Token::Or },
    TokenCheck { oper: String::from("^"), token: Token::ExclusiveOr },
    TokenCheck { oper: String::from("=="), token: Token::Equal },
    TokenCheck { oper: String::from("==="), token: Token::StrictEqual },
    TokenCheck { oper: String::from("!="), token: Token::NotEqual },
    TokenCheck { oper: String::from("!=="), token: Token::StrictNotEqual },
    TokenCheck { oper: String::from("<"), token: Token::Less },
    TokenCheck { oper: String::from("<="), token: Token::LessOrEqual },
    TokenCheck { oper: String::from(">"), token: Token::Greater },
    TokenCheck { oper: String::from(">="), token: Token::GreaterOrEqual },
    TokenCheck { oper: String::from("in"), token: Token::In },
    TokenCheck { oper: String::from("instanceof"), token: Token::Instanceof },
    TokenCheck { oper: String::from("<<"), token: Token::ShiftLeft },
    TokenCheck { oper: String::from(">>"), token: Token::ShiftRight },
    TokenCheck { oper: String::from(">>>"), token: Token::UnsignedShiftRight },
  ];
  let mut jsi_vm = JSI::new();
  for token in token_list.iter() {
    let mut code = String::from("1 ");
    code.push_str(token.oper.as_str());
    code.push_str(" 1;");
    let program = jsi_vm.parse(code).unwrap();
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
fn ast_lexer_assign_token() {
  let token_list = vec![
    TokenCheck { oper: String::from("="), token: Token::Assign },
    TokenCheck { oper: String::from("+="), token: Token::AddAssign },
    TokenCheck { oper: String::from("-="), token: Token::SubtractAssign },
    TokenCheck { oper: String::from("**="), token: Token::ExponentiationAssign },
    TokenCheck { oper: String::from("*="), token: Token::MultiplyAssign },
    TokenCheck { oper: String::from("/="), token: Token::SlashAssign },
    TokenCheck { oper: String::from("%="), token: Token::RemainderAssign },
    TokenCheck { oper: String::from("<<="), token: Token::ShiftLeftAssign },
    TokenCheck { oper: String::from(">>="), token: Token::ShiftRightAssign },
    TokenCheck { oper: String::from(">>>="), token: Token::UnsignedShiftRightAssign },
    TokenCheck { oper: String::from("&="), token: Token::AndAssign },
    TokenCheck { oper: String::from("^="), token: Token::ExclusiveOrAssign },
    TokenCheck { oper: String::from("|="), token: Token::OrAssign },
    TokenCheck { oper: String::from("&&="), token: Token::LogicalAndAssign },
    TokenCheck { oper: String::from("||="), token: Token::LogicalOrAssign },
    TokenCheck { oper: String::from("??="), token: Token::NullishCoalescingAssign },
  ];
  let mut jsi_vm = JSI::new();
  for token in token_list.iter() {
    let mut code = String::from("1");
    code.push_str(token.oper.as_str());
    code.push_str("1;");
    let program = jsi_vm.parse(code).unwrap();
    assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
      expression: Expression::Assign(AssignExpression {
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
    let program = jsi_vm.parse(code).unwrap();
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
    let program = jsi_vm.parse(code).unwrap();
    assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
      expression: Expression::PostfixUnary(PostfixUnaryExpression {
        operand: Box::new(Expression::Identifier(IdentifierLiteral{ literal: String::from("i") })),
        operator: token.token.clone(),
      })
    })]);
  }
}

#[test]
// 右结合性 a [op] b [op] c == a [op] (b [op] c)
fn ast_lexer_associativity_right_exponentiation() {
  let token_list = vec![
    TokenCheck { oper: String::from("**"), token: Token::Exponentiation },
  ];
  let mut jsi_vm = JSI::new();
  for token in token_list.iter() {
    let mut code = String::from("2 ");
    code.push_str(token.oper.as_str());
    code.push_str(" 3 ");
    code.push_str(token.oper.as_str());
    code.push_str(" 2;");
    let program = jsi_vm.parse(code).unwrap();
    assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
      expression: Expression::Binary(BinaryExpression { // 向右结合 2 op (3 op 2)
        left: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2,
        operator: token.token.clone(),
        right: Box::new(Expression::Binary(BinaryExpression { // 3 op 2
          left: Box::new(Expression::Number(NumberLiteral {  literal: String::from("3"), value: 3f64 })), // 3
          operator: token.token.clone(),
          right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2
        })),
      })
    })]);
  }

  let token_list_assign = vec![
    TokenCheck { oper: String::from("="), token: Token::Assign },
    TokenCheck { oper: String::from("+="), token: Token::AddAssign },
    TokenCheck { oper: String::from("-="), token: Token::SubtractAssign },
    TokenCheck { oper: String::from("**="), token: Token::ExponentiationAssign },
    TokenCheck { oper: String::from("*="), token: Token::MultiplyAssign },
    TokenCheck { oper: String::from("/="), token: Token::SlashAssign },
    TokenCheck { oper: String::from("%="), token: Token::RemainderAssign },
    TokenCheck { oper: String::from("<<="), token: Token::ShiftLeftAssign },
    TokenCheck { oper: String::from(">>="), token: Token::ShiftRightAssign },
    TokenCheck { oper: String::from(">>>="), token: Token::UnsignedShiftRightAssign },
    TokenCheck { oper: String::from("&="), token: Token::AndAssign },
    TokenCheck { oper: String::from("^="), token: Token::ExclusiveOrAssign },
    TokenCheck { oper: String::from("|="), token: Token::OrAssign },
    TokenCheck { oper: String::from("&&="), token: Token::LogicalAndAssign },
    TokenCheck { oper: String::from("||="), token: Token::LogicalOrAssign },
    TokenCheck { oper: String::from("??="), token: Token::NullishCoalescingAssign },
  ];
  for token in token_list_assign.iter() {
    let mut code = String::from("2 ");
    code.push_str(token.oper.as_str());
    code.push_str(" 3 ");
    code.push_str(token.oper.as_str());
    code.push_str(" 2;");
    let program = jsi_vm.parse(code).unwrap();
    assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
      expression: Expression::Assign(AssignExpression { // 向右结合 2 op (3 op 2)
        left: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2,
        operator: token.token.clone(),
        right: Box::new(Expression::Assign(AssignExpression { // 3 op 2
          left: Box::new(Expression::Number(NumberLiteral {  literal: String::from("3"), value: 3f64 })), // 3
          operator: token.token.clone(),
          right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2
        })),
      })
    })]);
  }
  // TODO: single oper
  // 三目运算符
  let program = jsi_vm.parse(String::from("1 ? 2 ? 3: 4: 5;")).unwrap();
  assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
    expression: Expression::Conditional(ConditionalExpression {
      condition: Box::new(Expression::Number(NumberLiteral {  literal: String::from("1"), value: 1f64 })),
      when_true: Box::new(Expression::Conditional(ConditionalExpression {
        condition: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })),
        when_true: Box::new(Expression::Number(NumberLiteral {  literal: String::from("3"), value: 3f64 })),
        when_false: Box::new(Expression::Number(NumberLiteral {  literal: String::from("4"), value: 4f64 }))
      })),
      when_false: Box::new(Expression::Number(NumberLiteral {  literal: String::from("5"), value: 5f64 }))
    })
  })]);
}



#[test]
// 左结合性 a [op] b [op] c == (a [op] b) [op] c
fn ast_lexer_associativity_left_exponentiation() {
  let token_list = vec![
    TokenCheck { oper: String::from("+"), token: Token::Plus },
    TokenCheck { oper: String::from("-"), token: Token::Subtract },
    TokenCheck { oper: String::from("*"), token: Token::Multiply },
    TokenCheck { oper: String::from("/"), token: Token::Slash },
    TokenCheck { oper: String::from("%"), token: Token::Remainder },
    TokenCheck { oper: String::from("<<"), token: Token::ShiftLeft },
    TokenCheck { oper: String::from(">>"), token: Token::ShiftRight },
    TokenCheck { oper: String::from(">>>"), token: Token::UnsignedShiftRight },
    TokenCheck { oper: String::from("<"), token: Token::Less },
    TokenCheck { oper: String::from(">"), token: Token::Greater },
    TokenCheck { oper: String::from("in"), token: Token::In },
    TokenCheck { oper: String::from("instanceof"), token: Token::Instanceof },
    TokenCheck { oper: String::from("&"), token: Token::And },
    TokenCheck { oper: String::from("^"), token: Token::ExclusiveOr },
    TokenCheck { oper: String::from("|"), token: Token::Or },
    TokenCheck { oper: String::from("&&"), token: Token::LogicalAnd },
    TokenCheck { oper: String::from("||"), token: Token::LogicalOr },
    TokenCheck { oper: String::from("??"), token: Token::NullishCoalescing },
  ];
  let mut jsi_vm = JSI::new();
  for token in token_list.iter() {
    let mut code = String::from("2 ");
    code.push_str(token.oper.as_str());
    code.push_str(" 3 ");
    code.push_str(token.oper.as_str());
    code.push_str(" 2;");
    let program = jsi_vm.parse(code).unwrap();
    assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
      expression: Expression::Binary(BinaryExpression { // 向左结合 (2 op 3) op 2
        left: Box::new(Expression::Binary(BinaryExpression { // 2 op 3
          left: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2
          operator: token.token.clone(),
          right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("3"), value: 3f64 })), // 3
        })),
        operator: token.token.clone(),
        right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2,
      })
    })]);
  }
  // TODO: queal
}

#[test]
fn ast_lexer_priority_between_exponentiation_shift() {
  let mut jsi_vm = JSI::new();
  let program = jsi_vm.parse(String::from("2 ** 3 >> 1;")).unwrap();
  assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
    expression: Expression::Binary(BinaryExpression {
      left: Box::new(Expression::Binary(BinaryExpression { // 2 ** 3
        left: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2
        operator: Token::Exponentiation, // **
        right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("3"), value: 3f64 })), // 3
      })),
      operator: Token::ShiftRight, // >>
      right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("1"), value: 1f64 })), // 1,
    })
  })]);
}


#[test]
fn ast_lexer_complex() {
  let mut jsi_vm = JSI::new();
  let program = jsi_vm.parse(String::from("(1 + 2) * 3 - 4 ** 2 >> (1 * 4 -3 + 1 == 2 ? 1 : 2);")).unwrap(); // return value is 4
  assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
    expression: Expression::Binary(BinaryExpression {
      left: Box::new(Expression::Binary(BinaryExpression { // (1 + 2) * 3 - 4 ** 2
        left: Box::new(Expression::Binary(BinaryExpression { // (1 + 2) * 3
          left: Box::new(Expression::Group(GroupExpression { // (1 + 2)
            expression:  Box::new(Expression::Binary(BinaryExpression { // 1 + 2
              left: Box::new(Expression::Number(NumberLiteral {  literal: String::from("1"), value: 1f64 })), // 1
              operator: Token::Plus, // +
              right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2
            }))
          })),
          operator: Token::Multiply, // *
          right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("3"), value: 3f64 })), // 3
        })),
        operator: Token::Subtract, // -
        right: Box::new(Expression::Binary(BinaryExpression { // 4 ** 2
          left: Box::new(Expression::Number(NumberLiteral {  literal: String::from("4"), value: 4f64 })), // 4
          operator: Token::Exponentiation, // -
          right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2
        })),
      })),
      operator: Token::ShiftRight, // >>
      right: Box::new(Expression::Group(GroupExpression { // (1 * 4 -3 + 1 == 2 ? 1 : 2)
        expression: Box::new(Expression::Conditional(ConditionalExpression { // 1 * 4 -3 + 1 == 2 ? 1 : 2
          condition: Box::new(Expression::Binary(BinaryExpression { // 1 * 4 -3 + 1 == 2
            left: Box::new(Expression::Binary(BinaryExpression { // 1 * 4 -3 + 1
              left: Box::new(Expression::Binary(BinaryExpression { // 1 * 4 -3
                left: Box::new(Expression::Binary(BinaryExpression { // 1 * 4
                  left: Box::new(Expression::Number(NumberLiteral {  literal: String::from("1"), value: 1f64 })), // 1
                  operator: Token::Multiply, // *
                  right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("4"), value: 4f64 })), // 4
                })),
                operator: Token::Subtract, // -
                right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("3"), value: 3f64 })), // 3
              })),
              operator: Token::Plus, // +
              right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("1"), value: 1f64 })), // 1
            })),
            operator: Token::Equal, // =
            right: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2
          })),
          when_true: Box::new(Expression::Number(NumberLiteral {  literal: String::from("1"), value: 1f64 })), // 1
          when_false: Box::new(Expression::Number(NumberLiteral {  literal: String::from("2"), value: 2f64 })), // 2
        }))
      })),
    })
  })]);
}
