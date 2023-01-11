use jsi::{JSI, value::Value};


#[test]
fn run_binary_assign() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("1 == true;"));
  assert_eq!(result , Value::Boolean(true));
}