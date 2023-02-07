
use jsi::{JSI, value::Value};

#[test]
fn run_function_instances_has_class() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let a = false;
  let b = 0;
  if (a) {
    b = 1;
  } else {
    b = 2;
  }\n
  b"));
  assert_eq!(result , Value::Number(2f64));
}