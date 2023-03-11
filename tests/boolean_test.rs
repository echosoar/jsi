use jsi::{JSI, value::Value};

#[test]
fn run_boolean_to_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let bool1 = false, bool2 = true;
  bool1.toString() + bool2.toString()")).unwrap();
  assert_eq!(result , Value::String(String::from("falsetrue")));
}

#[test]
fn run_boolean_object() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let bool1 = Boolean(), bool2 = new Boolean(1);
  bool1.toString() + bool2.toString()")).unwrap();
  assert_eq!(result , Value::String(String::from("falsetrue")));
}

