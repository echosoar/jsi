use jsi::{JSI, value::Value};

#[test]
fn run_promise() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolveCache;
  let promise = new Promise(resolve => {
    resolveCache = resolve;
  });
  resolveCache('123abc');
  let res = promise.then(value => value + 'xyz');
  res
  ")).unwrap();
  assert_eq!(result , Value::String(String::from("123abc")));
}
