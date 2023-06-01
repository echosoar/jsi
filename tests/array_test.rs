use jsi::{JSI, value::Value};


#[test]
fn run_array_to_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let arr = [1,2,3]\n
  arr.push(4);
  arr.toString()")).unwrap();
  assert_eq!(result , Value::String(String::from("1,2,3,4")));
}

#[test]
fn run_array_join() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let arr = [1,2,3]\n
  arr.join(':')")).unwrap();
  assert_eq!(result , Value::String(String::from("1:2:3")));
}

#[test]
// https://github.com/tc39/test262/blob/main/test/built-ins/Array/15.4.5-1.js
fn run_array_instances_has_class() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let arr = []\n
  Object.prototype.toString.call(arr)")).unwrap();
  assert_eq!(result , Value::String(String::from("[object Array]")));
}

#[test]
fn run_array_typeof() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let num = [1,2];
  num.concat([2,3], 4).join(',') + typeof num")).unwrap();
  assert_eq!(result , Value::String(String::from("1,2,2,3,4object")));
}