use jsi::{JSI, value::Value};


#[test]
fn run_array_to_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let arr = [1,2,3]\n
  arr.push(4);
  arr.toString()"));
  assert_eq!(result , Value::String(String::from("1,2,3,4")));
}

#[test]
fn run_array_join() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let arr = [1,2,3]\n
  arr.join(':')"));
  assert_eq!(result , Value::String(String::from("1:2:3")));
}