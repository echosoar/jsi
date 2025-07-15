use jsi::{JSI, value::Value, error::JSIErrorType};

#[test]
fn test_promise_constructor_exists() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("typeof Promise")).unwrap();
    assert_eq!(value, Value::String(String::from("function")));
}

#[test]
fn test_promise_resolve() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("Promise.resolve(42)")).unwrap();
    // Should return a Promise object
    match value {
        Value::Promise(_) => {
            // Success - it's a promise
        },
        _ => panic!("Promise.resolve should return a Promise object"),
    }
}

#[test]
fn test_promise_reject() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("Promise.reject('error')")).unwrap();
    // Should return a Promise object
    match value {
        Value::Promise(_) => {
            // Success - it's a promise
        },
        _ => panic!("Promise.reject should return a Promise object"),
    }
}

#[test]
fn test_promise_all() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("Promise.all([])")).unwrap();
    // Should return a Promise object
    match value {
        Value::Promise(_) => {
            // Success - it's a promise
        },
        _ => panic!("Promise.all should return a Promise object"),
    }
}

#[test]
fn test_promise_race() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("Promise.race([])")).unwrap();
    // Should return a Promise object
    match value {
        Value::Promise(_) => {
            // Success - it's a promise
        },
        _ => panic!("Promise.race should return a Promise object"),
    }
}

#[test]
fn test_promise_constructor_requires_function() {
    let mut jsi_vm = JSI::new();
    let result = jsi_vm.run(String::from("new Promise()"));
    if let Err(jsi_error) = result {
        assert_eq!(jsi_error.error_type, JSIErrorType::TypeError);
        assert_eq!(jsi_error.message, String::from("Promise resolver is not a function"));
    } else {
        panic!("Promise constructor should require a function argument");
    }
}

#[test]
fn test_promise_constructor_with_function() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("new Promise(function() {})")).unwrap();
    // Should return a Promise object
    match value {
        Value::Promise(_) => {
            // Success - it's a promise
        },
        _ => panic!("Promise constructor with function should return a Promise object"),
    }
}

#[test]
fn test_promise_then_method_exists() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("typeof Promise.resolve(42).then")).unwrap();
    assert_eq!(value, Value::String(String::from("function")));
}

#[test]
fn test_promise_catch_method_exists() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("typeof Promise.resolve(42).catch")).unwrap();
    assert_eq!(value, Value::String(String::from("function")));
}

#[test]
fn test_promise_finally_method_exists() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("typeof Promise.resolve(42).finally")).unwrap();
    assert_eq!(value, Value::String(String::from("function")));
}

#[test]
fn test_promise_then_returns_promise() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("Promise.resolve(42).then(function() {})")).unwrap();
    // Should return a Promise object
    match value {
        Value::Promise(_) => {
            // Success - it's a promise
        },
        _ => panic!("Promise.then should return a Promise object"),
    }
}

#[test]
fn test_promise_catch_returns_promise() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("Promise.reject('error').catch(function() {})")).unwrap();
    // Should return a Promise object
    match value {
        Value::Promise(_) => {
            // Success - it's a promise
        },
        _ => panic!("Promise.catch should return a Promise object"),
    }
}

#[test]
fn test_promise_finally_returns_promise() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run(String::from("Promise.resolve(42).finally(function() {})")).unwrap();
    // Should return a Promise object
    match value {
        Value::Promise(_) => {
            // Success - it's a promise
        },
        _ => panic!("Promise.finally should return a Promise object"),
    }
}

// Test both AST and bytecode execution modes
#[test]
fn test_promise_with_bytecode() {
    let mut jsi_vm = JSI::new();
    let value = jsi_vm.run_with_bytecode(String::from("Promise.resolve(123)")).unwrap();
    match value {
        Value::Promise(_) => {
            // Success - it's a promise
        },
        _ => panic!("Promise.resolve should return a Promise object in bytecode mode"),
    }
}