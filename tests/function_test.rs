use jsi::{JSI, value::Value, error::JSIErrorType};

#[test]
fn run_function_scope1() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run(String::from("\n
  let fun1 = function(x, y) {
    let a = 123;
    return fun2();
  };\n
  let fun2 = function() {
    return a;
  };\n
  fun1()"));
  if let Err(jsi_error) = value {
    assert_eq!(jsi_error.error_type, JSIErrorType::ReferenceError);
    assert_eq!(jsi_error.message , String::from("a is not defined"));
  } else {
    assert!(false , "need TypeError");
  }
}

#[test]
fn run_function_scope2() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run(String::from("\n
  let a = 123;
  let fun = function() {
    return a;
  };\n
  fun()")).unwrap();
  assert_eq!(value , Value::Number(123f64));
}

#[test]
fn run_function_instances_has_class() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  function func() {}\n
  Object.prototype.toString.call(func)")).unwrap();
  assert_eq!(result , Value::String(String::from("[object Function]")));
}

#[test]
fn run_function_typeof() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  function func() {}\n
  typeof func")).unwrap();
  assert_eq!(result , Value::String(String::from("function")));
}