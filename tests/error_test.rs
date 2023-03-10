use jsi::{JSI, value::Value};

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