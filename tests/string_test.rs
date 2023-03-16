use jsi::{JSI, value::Value};

#[test]
fn run_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let a = '123';
  let b = 'abc';
  a + b
  ")).unwrap();
  assert_eq!(result , Value::String(String::from("123abc")));
}

#[test]
fn run_string_object() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let a = String(123), b = new String('abc');
  a + b")).unwrap();
  assert_eq!(result , Value::String(String::from("123abc")));
}



#[test]
fn run_xxx() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  do function g() {} while (false)
  "));
  println!("result: {:?}", result)
}

