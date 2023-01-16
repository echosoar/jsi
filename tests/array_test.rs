use jsi::{JSI, value::Value};


#[test]
fn run_array() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let obj = [1,2,3]\n
  // Object.keys returns an array\n
  /* array.toString() */
  Object.keys(obj).toString()"));
  assert_eq!(result , Value::String(String::from("a,b,c")));
}