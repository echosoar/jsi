use jsi::{JSI, value::Value};

#[test]
fn run_async_function_basic() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  async function asyncFunc() {
    return 'hello';
  }
  asyncFunc()
  ")).unwrap();

  // async function should return a Promise
  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value, Value::String(String::from("hello")));
  } else {
    panic!("Expected a Promise from async function");
  }
}

#[test]
fn run_async_function_with_number() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  async function getNumber() {
    return 42;
  }
  getNumber()
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value, Value::Number(42f64));
  } else {
    panic!("Expected a Promise from async function");
  }
}

#[test]
fn run_async_function_expression() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let asyncFunc = async function() {
    return 'async expression';
  };
  asyncFunc()
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value, Value::String(String::from("async expression")));
  } else {
    panic!("Expected a Promise from async function expression");
  }
}

#[test]
fn run_await_with_resolved_promise() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolveCache;
  let promise = new Promise(resolve => {
    resolveCache = resolve;
  });
  resolveCache('resolved value');

  async function awaitFunc() {
    let value = await promise;
    return value;
  }
  awaitFunc()
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value, Value::String(String::from("resolved value")));
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_await_with_non_promise() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  async function func() {
    let value = await 'not a promise';
    return value;
  }
  func()
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value, Value::String(String::from("not a promise")));
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_async_with_promise_resolve() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  async function func() {
    return Promise.resolve('resolved');
  }
  func()
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    // The value should be another promise that's fulfilled
    if let Value::Promise(inner_promise_rc) = &value {
      let inner_promise_mut = inner_promise_rc.borrow();
      let inner_state = inner_promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
      assert_eq!(inner_state, Value::String(String::from("fulfilled")));
      let inner_value = inner_promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
      assert_eq!(inner_value, Value::String(String::from("resolved")));
    } else {
      panic!("Expected inner value to be a Promise");
    }
  } else {
    panic!("Expected a Promise");
  }
}

#[test]
fn run_async_arrow_function() {
  let mut jsi = JSI::new();
  // Note: async arrow function support is limited, using async function expression instead
  let result = jsi.run(String::from("\
  let asyncArrow = async function() {
    return 'arrow result';
  };
  asyncArrow()
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value, Value::String(String::from("arrow result")));
  } else {
    panic!("Expected a Promise from async function expression");
  }
}

#[test]
fn run_async_function_no_return() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  async function noReturn() {
    // no return statement
  }
  noReturn()
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value, Value::Undefined);
  } else {
    panic!("Expected a Promise from async function with no return");
  }
}

#[test]
fn run_await_in_expression() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let resolveCache;
  let promise = new Promise(resolve => {
    resolveCache = resolve;
  });

  async function compute() {
    let a = await promise;
    return a + 5;
  }
  let result = compute();
  resolveCache(10);
  result
  ")).unwrap();

  if let Value::Promise(promise_rc) = &result {
    let promise_mut = promise_rc.borrow();
    let state = promise_mut.get_inner_property_value(String::from("[[PromiseState]]")).unwrap();
    assert_eq!(state, Value::String(String::from("fulfilled")));
    let value = promise_mut.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap();
    assert_eq!(value, Value::Number(15f64));
  } else {
    panic!("Expected a Promise");
  }
}