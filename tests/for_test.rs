use jsi::{JSI, value::Value};


// ==================== for-in tests ====================

#[test]
fn run_for_in_object_keys() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let keys = [];
  let obj = {a: 1, b: 2, c: 3};
  for (let key in obj) {
    keys.push(key);
  }
  keys.length")).unwrap();
  // Check that we got 3 keys (order may vary due to HashMap)
  assert_eq!(result, Value::Number(3.0));
}

#[test]
fn run_for_in_var_declaration() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let keys = [];
  let obj = {x: 10, y: 20};
  for (var key in obj) {
    keys.push(key);
  }
  keys.length")).unwrap();
  // Check that we got 2 keys
  assert_eq!(result, Value::Number(2.0));
}

#[test]
fn run_for_in_const_declaration() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let keys = [];
  let obj = {foo: 'bar'};
  for (const k in obj) {
    keys.push(k);
  }
  keys.join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("foo")));
}

#[test]
fn run_for_in_array_indices() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let count = 0;
  let arr = [10, 20, 30];
  for (let i in arr) {
    count = count + 1;
  }
  count")).unwrap();
  // Should iterate over 3 indices
  assert_eq!(result, Value::Number(3.0));
}

#[test]
fn run_for_in_with_break() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let count = 0;
  let obj = {a: 1, b: 2, c: 3, d: 4};
  for (let key in obj) {
    count = count + 1;
    if (count == 2) {
      break;
    }
  }
  count")).unwrap();
  // Should break after 2 iterations
  assert_eq!(result, Value::Number(2.0));
}

#[test]
fn run_for_in_with_continue() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let sum = 0;
  let obj = {a: 1, b: 2, c: 3};
  for (let key in obj) {
    if (key == 'b') {
      continue;
    }
    sum = sum + obj[key];
  }
  sum")).unwrap();
  // 1 + 3 = 4 (skipping b's value 2)
  assert_eq!(result, Value::Number(4.0));
}

#[test]
fn run_for_in_access_value() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let sum = 0;
  let obj = {a: 10, b: 20, c: 30};
  for (let key in obj) {
    sum = sum + obj[key];
  }
  sum")).unwrap();
  assert_eq!(result, Value::Number(60.0));
}

// ==================== for-of tests ====================

#[test]
fn run_for_of_array_values() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let values = [];
  let arr = [1, 2, 3];
  for (let val of arr) {
    values.push(val);
  }
  values.join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,3")));
}

#[test]
fn run_for_of_var_declaration() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let sum = 0;
  let arr = [10, 20, 30];
  for (var num of arr) {
    sum = sum + num;
  }
  sum")).unwrap();
  assert_eq!(result, Value::Number(60.0));
}

#[test]
fn run_for_of_const_declaration() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let values = [];
  let arr = [5, 6, 7];
  for (const v of arr) {
    values.push(v * 2);
  }
  values.join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("10,12,14")));
}

#[test]
fn run_for_of_string_chars() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let chars = [];
  let str = 'abc';
  for (let char of str) {
    chars.push(char);
  }
  chars.join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("a,b,c")));
}

#[test]
fn run_for_of_with_break() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let values = [];
  let arr = [1, 2, 3, 4, 5];
  for (let val of arr) {
    values.push(val);
    if (val == 3) {
      break;
    }
  }
  values.join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,3")));
}

#[test]
fn run_for_of_with_continue() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let values = [];
  let arr = [1, 2, 3, 4, 5];
  for (let val of arr) {
    if (val == 3) {
      continue;
    }
    values.push(val);
  }
  values.join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,4,5")));
}

#[test]
fn run_for_of_sum() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let sum = 0;
  let arr = [1, 2, 3, 4, 5];
  for (let num of arr) {
    sum = sum + num;
  }
  sum")).unwrap();
  assert_eq!(result, Value::Number(15.0));
}

#[test]
fn run_for_of_empty_array() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let count = 0;
  let arr = [];
  for (let val of arr) {
    count = count + 1;
  }
  count")).unwrap();
  assert_eq!(result, Value::Number(0.0));
}

#[test]
fn run_for_in_empty_object() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let count = 0;
  let obj = {};
  for (let key in obj) {
    count = count + 1;
  }
  count")).unwrap();
  assert_eq!(result, Value::Number(0.0));
}

#[test]
fn run_for_of_nested_loop() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let result = [];
  let arr = [[1, 2], [3, 4]];
  for (let inner of arr) {
    for (let val of inner) {
      result.push(val);
    }
  }
  result.join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,3,4")));
}