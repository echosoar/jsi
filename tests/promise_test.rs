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

#[test]
fn run_promise_pending_then() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolveCache;
  let promise = new Promise(resolve => {
    resolveCache = resolve;
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
fn run_promise_all() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let promise1 = Promise.resolve('value1');
  let promise2 = Promise.resolve('value2');
  let promise3 = Promise.resolve('value3');
  Promise.all([promise1, promise2, promise3])
  ")).unwrap();
  
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow_mut();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state , Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    
    if let Value::Array(array_rc) = value {
      let array_borrowed = array_rc.borrow();
      let val0 = array_borrowed.get_value("0".to_string());
      let val1 = array_borrowed.get_value("1".to_string());  
      let val2 = array_borrowed.get_value("2".to_string());
      assert_eq!(val0, Value::String("value1".to_string()));
      assert_eq!(val1, Value::String("value2".to_string()));
      assert_eq!(val2, Value::String("value3".to_string()));
    } else {
      panic!("Expected an Array");
    }
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_promise_all_reject() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let promise1 = Promise.resolve('value1');
  let promise2 = Promise.reject('error2');
  let promise3 = Promise.resolve('value3');
  Promise.all([promise1, promise2, promise3])
  ")).unwrap();
  
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow_mut();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state , Value::String(String::from("rejected")));
    let reason = promise_mut.get_inner_property_value(String::from("[[PromiseRejectedReason]]")).unwrap();
    assert_eq!(reason , Value::String(String::from("error2")));
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_promise_race() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let promise1 = Promise.resolve('first');
  let promise2 = Promise.resolve('second');
  let promise3 = Promise.resolve('third');
  Promise.race([promise1, promise2, promise3])
  ")).unwrap();
  
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow_mut();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state , Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value , Value::String(String::from("first")));
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_promise_race_reject() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let promise1 = Promise.reject('error1');
  let promise2 = Promise.resolve('value2');
  Promise.race([promise1, promise2])
  ")).unwrap();
  
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow_mut();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state , Value::String(String::from("rejected")));
    let reason = promise_mut.get_inner_property_value(String::from("[[PromiseRejectedReason]]")).unwrap();
    assert_eq!(reason , Value::String(String::from("error1")));
  } else {
    panic!("Expected a Promise");
  }
}
