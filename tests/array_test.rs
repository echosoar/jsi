use jsi::{JSI, value::Value};


#[test]
fn run_array_to_string() {
  let mut jsi = JSI::new();
  let result = jsi.run_with_bytecode(String::from("\
  let arr = [1,2,3]\n
  arr.push(4);
  arr.toString()")).unwrap();
  assert_eq!(result , Value::String(String::from("1,2,3,4")));
}

#[test]
fn run_array_join() {
  let mut jsi = JSI::new();
  let result = jsi.run_with_bytecode(String::from("\
  let arr = [1,2,3]\n
  arr.join(':')")).unwrap();
  assert_eq!(result , Value::String(String::from("1:2:3")));
}

#[test]
// https://github.com/tc39/test262/blob/main/test/built-ins/Array/15.4.5-1.js
fn run_array_instances_has_class() {
  let mut jsi = JSI::new();
  let result = jsi.run_with_bytecode(String::from("\
  let arr = []\n
  Object.prototype.toString.call(arr)")).unwrap();
  assert_eq!(result , Value::String(String::from("[object Array]")));
}

#[test]
fn run_array_typeof() {
  let mut jsi = JSI::new();
  let result = jsi.run_with_bytecode(String::from("\
  let num = [1,2];
  num.concat([2,3], 4).join(',') + typeof num")).unwrap();
  assert_eq!(result , Value::String(String::from("1,2,2,3,4object")));
}

#[test]
fn run_array_map() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  let result = jsi.run_with_bytecode(String::from("\n
  function double(x) {
    return x * 2;
  };
  let arr = [1, 2, 3];
  arr.map(double).join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("2,4,6")));
}

#[test]
fn run_array_map_with_index() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  let result = jsi.run_with_bytecode(String::from("\n
  let arr = [10, 20, 30];
  arr.map((x, i) => (x + i)).join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("10,21,32")));
}

#[test]
fn run_array_map_with_object() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  let result = jsi.run_with_bytecode(String::from("\n
  let arr = [{ name: 'Alice' }, { name: 'Bob' }, { name: 'Charlie' }];
  arr.map((x, i) => x.name + ':' + i).join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("Alice:0,Bob:1,Charlie:2")));
}

#[test]
fn run_array_for_each() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  let result = jsi.run_with_bytecode(String::from("\n
  let sum = 0;
  function addSum(x) { sum = sum + x };
  let arr = [1, 2, 3];
  arr.forEach(addSum);
  sum")).unwrap();
  assert_eq!(result, Value::Number(6.0));
}

#[test]
fn run_array_for_each_simple() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  // Test that forEach iterates correctly with a simple side effect
  let result = jsi.run_with_bytecode(String::from("\n
  let count = 0;
  function increment(x) { count = count + 1 };
  let arr = [10, 20, 30];
  arr.forEach(increment);
  count")).unwrap();
  assert_eq!(result, Value::Number(3.0));
}

#[test]
fn run_array_for_each_returns_undefined() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  let result = jsi.run_with_bytecode(String::from("\n
  function returnX(x) {
    return x;
  };
  let arr = [1, 2, 3];
  arr.forEach(returnX)")).unwrap();
  assert_eq!(result, Value::Undefined);
}

#[test]
fn run_array_filter() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  let result = jsi.run_with_bytecode(String::from("\n
  function greaterThan2(x) {
    return x > 2;
  };
  let arr = [1, 2, 3, 4, 5];
  arr.filter(greaterThan2).join(',')")).unwrap();
  assert_eq!(result, Value::String(String::from("3,4,5")));
}

#[test]
fn run_array_filter_empty() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  let result = jsi.run_with_bytecode(String::from("\n
  function greaterThan10(x) {
    return x > 10;
  };
  let arr = [1, 2, 3];
  arr.filter(greaterThan10).length")).unwrap();
  assert_eq!(result, Value::Number(0.0));
}
// ========== Array includes 测试 ==========
#[test]
fn run_array_includes_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].includes(2)
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn run_array_includes_not_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].includes(4)
  ")).unwrap();
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn run_array_includes_with_from_index() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 2].includes(2, 2)
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn run_array_includes_with_negative_from_index() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].includes(3, -1)
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn run_array_includes_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    ['a', 'b', 'c'].includes('b')
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

// ========== Array indexOf 测试 ==========
#[test]
fn run_array_index_of_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].indexOf(2)
  ")).unwrap();
  assert_eq!(result, Value::Number(1f64));
}

#[test]
fn run_array_index_of_not_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].indexOf(4)
  ")).unwrap();
  assert_eq!(result, Value::Number(-1f64));
}

#[test]
fn run_array_index_of_with_from_index() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 2].indexOf(2, 2)
  ")).unwrap();
  assert_eq!(result, Value::Number(3f64));
}

#[test]
fn run_array_index_of_first_occurrence() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 2].indexOf(2)
  ")).unwrap();
  assert_eq!(result, Value::Number(1f64));
}

#[test]
fn run_array_index_of_negative_from_index() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].indexOf(3, -1)
  ")).unwrap();
  assert_eq!(result, Value::Number(2f64));
}

#[test]
fn run_array_index_of_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    ['a', 'b', 'c'].indexOf('b')
  ")).unwrap();
  assert_eq!(result, Value::Number(1f64));
}
