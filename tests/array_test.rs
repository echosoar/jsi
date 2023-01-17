use jsi::{JSI, value::Value};


#[test]
fn run_array() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let arr = [1,2,3]\n
  arr.toString()"));
  assert_eq!(result , Value::String(String::from("1,2,3")));
}