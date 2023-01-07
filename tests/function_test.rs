use jsi::{JSI, value::Value};

#[test]
fn run_function() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run(String::from("function abc(a, b, c) { return a +b - c;} return abc(123, 456, 789);"));
  assert_eq!(value, Value::Number(-210f64)) // 123 + 456 - 789 = -210
}