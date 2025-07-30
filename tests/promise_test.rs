use jsi::{JSI, value::Value};

#[test]
fn run_promise_base() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolveCache;
  let promise = new Promise(resolve => {
    resolveCache = resolve;
  });
  resolveCache('123abc');
  // let res = promise.then(value => value + 'xyz');
  promise
  ")).unwrap();
  assert_eq!(result , Value::String(String::from("123abc")));
}


#[test]
fn run_promise_then() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolveCache;
  let promise = new Promise(resolve => {
    resolveCache = resolve;
  });
  resolveCache('123abc');
  let res = promise.then(value => {
    return Promise.reject(value + ':reject1');
  }).then(value => {
    return Promise.reject(value + ':resolve2');
  }, rejValue => {
    return Promise.resolve(rejValue + ':reject2');
  }).then(value => {
    return Promise.reject(value + ':resolve3');
  }, rejValue => {
    return Promise.resolve(rejValue + ':reject3');
  });
  res
  ")).unwrap();
  assert_eq!(result , Value::String(String::from("123abc:reject1:reject2:resolve3")));
}
