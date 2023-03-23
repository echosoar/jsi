use jsi::{JSI, value::Value, error::JSIErrorType};

#[test]
fn run_throw_new_error() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let errA = new Error;
  let errB = new Error(123);
  let errC = new Error('abc');
  let errD = Error('def');
  let result = {
    errA: errA.message,
    errB: errB.message,
    errC: errC.message,
    errD: errD.message
  }
  "));
  println!("result: {:?}", result);
  // assert_eq!(result , Value::String(String::from("1,2")));
}

#[test]
fn run_reference_error() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let a = b + 1"));
  if let Err(jsi_error) = result {
    assert_eq!(jsi_error.message , String::from("b is not defined"));
  } else {
    assert!(false , "need reference error");
  }
}

#[test]
fn run_new_number_error() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let num = new 123;\n
  "));
  if let Err(jsi_error) = result {
    assert_eq!(jsi_error.error_type, JSIErrorType::TypeError);
    assert_eq!(jsi_error.message , String::from("123 is not a constructor"));
  } else {
    assert!(false , "need TypeError");
  }
}

#[test]
fn run_new_false_error() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let num = new false;\n
  "));
  if let Err(jsi_error) = result {
    assert_eq!(jsi_error.error_type, JSIErrorType::TypeError);
    assert_eq!(jsi_error.message , String::from("false is not a constructor"));
  } else {
    assert!(false , "need TypeError");
  }
}

#[test]
fn run_read_properties_of_null_error() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let num = null.a;\n
  "));
  if let Err(jsi_error) = result {
    assert_eq!(jsi_error.error_type, JSIErrorType::TypeError);
    assert_eq!(jsi_error.message , String::from("Cannot read properties of null (reading 'a')"));
  } else {
    assert!(false , "need TypeError");
  }
}

#[test]
fn run_read_properties_of_number_property() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let num = null.1;\n
  "));
  if let Err(jsi_error) = result {
    assert_eq!(jsi_error.error_type, JSIErrorType::SyntaxError);
    assert_eq!(jsi_error.message , String::from("Unexpected number"));
  } else {
    assert!(false , "need SyntaxError");
  }
}

#[test]
fn run_read_properties_of_null_error_catch() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  try {
    let num = null.a;
  } catch (e) {
    e.message
  }
  ")).unwrap();
  assert_eq!(result , Value::String(String::from("Cannot read properties of null (reading 'a')")));
}

#[test]
fn run_throw_error_catch() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  try {
    throw {a: 'abc'}
  } catch (obj) {
    obj.a
  }
  ")).unwrap();
  assert_eq!(result , Value::String(String::from("abc")));
}


#[test]
fn run_const_error() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    const a = 123;
    a = 456;
  "));
  println!("result: {:?}", result);
  if let Err(jsi_error) = result {
    assert_eq!(jsi_error.error_type, JSIErrorType::TypeError);
    assert_eq!(jsi_error.message , String::from("Assignment to constant variable"));
  } else {
    assert!(false , "need SyntaxError");
  }
}