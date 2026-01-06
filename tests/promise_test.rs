use jsi::{JSI, value::Value};

#[test]
fn run_promise_base() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolveCache;
  let promise = new Promise(resolve => {
    resolveCache = resolve;
  });
  promise.then(value => {
    return value + 'xyz';
  });
  promise.then(value => {
    throw new Error('should not call');
  });
  let res = promise.then(value => value + 'xyz');
  resolveCache('123abc');
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
  let resolveCache2;
  let promise = new Promise(resolve => {
    resolveCache = resolve;
  });
  
  let res = promise.then(value1 => {
    return new Promise(resolve => {
      resolveCache2 = resolve;
    }).then(value2 => {
      return value1 + 'x:' + value2;
    });
  }).then(value3 => {
    return value3 + '5:';
  });
  resolveCache('a:');
  resolveCache2('b:');
  res
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow_mut();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state , Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value , Value::String(String::from("a:x:b:5:")));
  } else {
    panic!("Expected a Promise");
  }
}
