use jsi::{JSI, ast_node::{Expression, Statement, ObjectLiteral, ExpressionStatement, PropertyAssignment, NumberLiteral, StringLiteral, Keywords, BinaryExpression, ComputedPropertyName}, ast_token::Token, value::Value};

#[test]
fn ast_base() {
  let mut jsi = JSI::new();
  let program = jsi.parse(String::from("{a: 123, b: '123', [1 + 'a']: false}"));
  let expr = match &program.body[0] {
    Statement::Expression(expr_statement) => {
      expr_statement.expression.clone()
    },
    _ => Expression::Unknown,
  };
  assert_eq!(expr, Expression::Object(ObjectLiteral {
      properties: vec![
        PropertyAssignment{
          name: Box::new(Expression::String(StringLiteral { literal: String::from("a"), value: String::from("a")})),
          initializer: Box::new(Expression::Number(NumberLiteral { literal: String::from("123"), value: 123f64 })),
        },
        PropertyAssignment{
          name: Box::new(Expression::String(StringLiteral { literal: String::from("b"), value: String::from("b")})),
          initializer: Box::new(Expression::String(StringLiteral { literal: String::from("'123'"), value: String::from("123") })),
        },
        PropertyAssignment{
          name: Box::new(Expression::ComputedPropertyName(ComputedPropertyName {
            expression: Box::new(Expression::Binary(BinaryExpression {
              left: Box::new(Expression::Number(NumberLiteral { literal: String::from("1"), value: 1f64 })),
              operator: Token::Plus,
              right: Box::new(Expression::String(StringLiteral { literal: String::from("'a'"), value: String::from("a") })),
            }))
          })),
          initializer: Box::new(Expression::Keyword(Keywords::False)),
        }
      ]
  }));
}

#[test]
fn ast_with_child_object() {
  let mut jsi = JSI::new();
  let program = jsi.parse(String::from("{obj: { x: false}}"));
  assert_eq!(program.body, vec![Statement::Expression(ExpressionStatement {
    expression: Expression::Object(ObjectLiteral {
      properties: vec![
        PropertyAssignment{
          name: Box::new(Expression::String(StringLiteral { literal: String::from("obj"), value: String::from("obj")})),
          initializer: Box::new(Expression::Object(ObjectLiteral {
            properties: vec![
              PropertyAssignment{
                name: Box::new(Expression::String(StringLiteral { literal: String::from("x"), value: String::from("x")})),
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
    let a = 'foo', b = 42, c = {};\
    let obj = {a, b, c};\
    return obj.b;"));
  assert_eq!(result , Value::Number(42f64));
}

#[test]
fn run_object_duplicate_naming() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("let obj = { x: 1, x: 2}; obj.x;"));
  assert_eq!(result , Value::Number(2f64));
}

#[test]
fn run_object_property_access() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("let obj = { a: 'foo', b: 123}; obj['a'] + obj.b;"));
  assert_eq!(result , Value::String(String::from("foo123")));
}


#[test]
fn run_object_with_function_property() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("let obj = { fun: function(a) {return a + 123;}}; obj.fun('abc') + 456;"));
  assert_eq!(result , Value::String(String::from("abc123456")));
}


#[test]
fn run_object_as_param_ref() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let obj = { a: 123};\
  let fun = function(obj) {\
    let x = 123;\
    x = 456;\
    obj.a = x;};\
  fun(obj);\
  obj.a;\
  "));
  assert_eq!(result , Value::Number(456f64));
}

#[test]
fn run_object_with_array_key() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let a = [1,2]\n
  let b = {[a]: 3}\n
  Object.keys(b).toString()"));
  assert_eq!(result , Value::String(String::from("1,2")));
}