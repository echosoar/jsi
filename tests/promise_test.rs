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

#[test]
fn run_promise_all_empty() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  Promise.all([])
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    if let Value::Array(arr) = &value {
      let arr_mut = arr.borrow();
      let length = arr_mut.get_property_value(String::from("length"));
      assert_eq!(length, Value::Number(0f64));
    } else {
      panic!("Expected an Array");
    }
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_promise_all_all_fulfilled() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolve1, resolve2, resolve3;
  let p1 = new Promise(r => { resolve1 = r; });
  let p2 = new Promise(r => { resolve2 = r; });
  let p3 = new Promise(r => { resolve3 = r; });
  let allPromise = Promise.all([p1, p2, p3]);
  resolve1('a');
  resolve2('b');
  resolve3('c');
  allPromise
  ")).unwrap();
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    if let Value::Array(arr) = &value {
      let arr_mut = arr.borrow();
      let length = arr_mut.get_property_value(String::from("length"));
      assert_eq!(length, Value::Number(3f64));
      let val0 = arr_mut.get_property_value(String::from("0"));
      assert_eq!(val0, Value::String(String::from("a")));
      let val1 = arr_mut.get_property_value(String::from("1"));
      assert_eq!(val1, Value::String(String::from("b")));
      let val2 = arr_mut.get_property_value(String::from("2"));
      assert_eq!(val2, Value::String(String::from("c")));
    } else {
      panic!("Expected an Array");
    }
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_promise_all_with_non_promise_values() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  Promise.all([1, 'hello', true])
  ")).unwrap();
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    if let Value::Array(arr) = &value {
      let arr_mut = arr.borrow();
      let length = arr_mut.get_property_value(String::from("length"));
      assert_eq!(length, Value::Number(3f64));
      let val0 = arr_mut.get_property_value(String::from("0"));
      assert_eq!(val0, Value::Number(1f64));
      let val1 = arr_mut.get_property_value(String::from("1"));
      assert_eq!(val1, Value::String(String::from("hello")));
      let val2 = arr_mut.get_property_value(String::from("2"));
      assert_eq!(val2, Value::Boolean(true));
    } else {
      panic!("Expected an Array");
    }
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_promise_all_one_rejected() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolve1, reject2, resolve3;
  let p1 = new Promise(r => { resolve1 = r; });
  let p2 = new Promise((_, r) => { reject2 = r; });
  let p3 = new Promise(r => { resolve3 = r; });
  let allPromise = Promise.all([p1, p2, p3]);
  resolve1('a');
  reject2('error');
  resolve3('c');
  allPromise
  ")).unwrap();
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("rejected")));
    let reason = promise_mut.get_inner_property_value(String::from("[[PromiseRejectedReason]]")).unwrap();
    assert_eq!(reason, Value::String(String::from("error")));
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_promise_all_preserves_order() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolve1, resolve2, resolve3;
  let p1 = new Promise(r => { resolve1 = r; });
  let p2 = new Promise(r => { resolve2 = r; });
  let p3 = new Promise(r => { resolve3 = r; });
  let allPromise = Promise.all([p1, p2, p3]);
  // Resolve in different order: 3, 1, 2
  resolve3('third');
  resolve1('first');
  resolve2('second');
  allPromise
  ")).unwrap();
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    if let Value::Array(arr) = &value {
      let arr_mut = arr.borrow();
      // Results should be in original order: ['first', 'second', 'third']
      let val0 = arr_mut.get_property_value(String::from("0"));
      assert_eq!(val0, Value::String(String::from("first")));
      let val1 = arr_mut.get_property_value(String::from("1"));
      assert_eq!(val1, Value::String(String::from("second")));
      let val2 = arr_mut.get_property_value(String::from("2"));
      assert_eq!(val2, Value::String(String::from("third")));
    } else {
      panic!("Expected an Array");
    }
  } else {
    panic!("Expected a Promise");
  }
}
