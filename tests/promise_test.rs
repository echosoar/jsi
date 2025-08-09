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
  let res = promise.then(value => value + 'xyz');
  res
  ")).unwrap();
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow_mut();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state , Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value , Value::String(String::from("123abcxyz")));
  } else {
    panic!("Expected a Promise");
  }
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

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow_mut();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state , Value::String(String::from("rejected")));
    let reason = promise_mut.get_inner_property_value(String::from("[[PromiseRejectedReason]]")).unwrap();
    assert_eq!(reason , Value::String(String::from("123abc:reject1:reject2:resolve3")));
  } else {
    panic!("Expected a Promise");
  }
}
