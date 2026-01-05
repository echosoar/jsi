use jsi::{
    ast_node::{
        Expression, ExpressionStatement, IdentifierLiteral, NewExpression, NumberLiteral,
        PropertyAccessExpression, Statement,
    },
    JSI,
};

#[test]
fn run_new_ast() {
    let mut jsi_vm = JSI::new();
    let program = jsi_vm
        .parse(String::from(
            "
    new a(123).b
  ",
        ))
        .unwrap();
    println!("program {:?}", program);

    assert_eq!(
        program.body,
        vec![Statement::Expression(ExpressionStatement {
            expression: Expression::PropertyAccess(PropertyAccessExpression {
                expression: Box::new(Expression::New(NewExpression {
                    expression: Box::new(Expression::Identifier(IdentifierLiteral {
                        literal: String::from("a"),
                    })),
                    arguments: vec![Expression::Number(NumberLiteral {
                        literal: String::from("123"),
                        value: 123.0
                    })],
                })),
                name: IdentifierLiteral {
                    literal: String::from("b"),
                },
            })
        })]
    );
}
