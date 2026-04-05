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

// ========== Array fill 测试 ==========
#[test]
fn run_array_fill_basic() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].fill(0).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("0,0,0")));
}

#[test]
fn run_array_fill_with_start() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 4, 5].fill(0, 2).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,0,0,0")));
}

#[test]
fn run_array_fill_with_start_and_end() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 4, 5].fill(0, 1, 3).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,0,0,4,5")));
}

#[test]
fn run_array_fill_negative_indices() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 4, 5].fill(0, -3, -1).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,0,0,5")));
}

#[test]
fn run_array_fill_returns_array() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3];
    arr.fill(0) === arr
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

// ========== Array find 测试 ==========
#[test]
fn run_array_find_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 4, 5].find(x => x > 3)
  ")).unwrap();
  assert_eq!(result, Value::Number(4f64));
}

#[test]
fn run_array_find_not_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].find(x => x > 10)
  ")).unwrap();
  assert_eq!(result, Value::Undefined);
}

#[test]
fn run_array_find_with_index() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [10, 20, 30].find((x, i) => i === 1)
  ")).unwrap();
  assert_eq!(result, Value::Number(20f64));
}

#[test]
fn run_array_find_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    ['apple', 'banana', 'cherry'].find(x => x.startsWith('b'))
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("banana")));
}

// ========== Array findIndex 测试 ==========
#[test]
fn run_array_find_index_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 4, 5].findIndex(x => x > 3)
  ")).unwrap();
  assert_eq!(result, Value::Number(3f64));
}

#[test]
fn run_array_find_index_not_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].findIndex(x => x > 10)
  ")).unwrap();
  assert_eq!(result, Value::Number(-1f64));
}

#[test]
fn run_array_find_index_with_index() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [10, 20, 30].findIndex((x, i) => i === 2)
  ")).unwrap();
  assert_eq!(result, Value::Number(2f64));
}

// ========== Array pop 测试 ==========
#[test]
fn run_array_pop_basic() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3];
    arr.pop()
  ")).unwrap();
  assert_eq!(result, Value::Number(3f64));
}

#[test]
fn run_array_pop_updates_length() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3];
    arr.pop();
    arr.length
  ")).unwrap();
  assert_eq!(result, Value::Number(2f64));
}

#[test]
fn run_array_pop_empty() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [].pop()
  ")).unwrap();
  assert_eq!(result, Value::Undefined);
}

#[test]
fn run_array_pop_multiple() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3, 4];
    arr.pop();
    arr.pop();
    arr.join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2")));
}

// ========== Array reverse 测试 ==========
#[test]
fn run_array_reverse_basic() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].reverse().join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("3,2,1")));
}

#[test]
fn run_array_reverse_returns_same_array() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3];
    arr.reverse() === arr
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn run_array_reverse_empty() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [].reverse().length
  ")).unwrap();
  assert_eq!(result, Value::Number(0f64));
}

#[test]
fn run_array_reverse_single_element() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1].reverse().join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1")));
}

// ========== Array shift 测试 ==========
#[test]
fn run_array_shift_basic() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3];
    arr.shift()
  ")).unwrap();
  assert_eq!(result, Value::Number(1f64));
}

#[test]
fn run_array_shift_updates_length() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3];
    arr.shift();
    arr.length
  ")).unwrap();
  assert_eq!(result, Value::Number(2f64));
}

#[test]
fn run_array_shift_empty() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [].shift()
  ")).unwrap();
  assert_eq!(result, Value::Undefined);
}

#[test]
fn run_array_shift_multiple() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3, 4];
    arr.shift();
    arr.shift();
    arr.join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("3,4")));
}

// ========== Array unshift 测试 ==========
#[test]
fn run_array_unshift_basic() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3];
    arr.unshift(0)
  ")).unwrap();
  assert_eq!(result, Value::Number(4f64));
}

#[test]
fn run_array_unshift_multiple_elements() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [3, 4];
    arr.unshift(1, 2);
    arr.join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,3,4")));
}

#[test]
fn run_array_unshift_empty_array() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [];
    arr.unshift(1);
    arr.join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1")));
}

// ========== Array sort 测试 ==========
#[test]
fn run_array_sort_numbers() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [3, 1, 4, 1, 5, 9, 2, 6].sort().join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,1,2,3,4,5,6,9")));
}

#[test]
fn run_array_sort_strings() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    ['banana', 'apple', 'cherry'].sort().join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("apple,banana,cherry")));
}

#[test]
fn run_array_sort_returns_same_array() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [3, 1, 2];
    arr.sort() === arr
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn run_array_sort_empty() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [].sort().length
  ")).unwrap();
  assert_eq!(result, Value::Number(0f64));
}

// ========== Array slice 测试 ==========
#[test]
fn run_array_slice_basic() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 4, 5].slice(1, 4).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("2,3,4")));
}

#[test]
fn run_array_slice_no_args() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].slice().join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,3")));
}

#[test]
fn run_array_slice_negative_start() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 4, 5].slice(-2).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("4,5")));
}

#[test]
fn run_array_slice_negative_end() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3, 4, 5].slice(1, -1).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("2,3,4")));
}

#[test]
fn run_array_slice_empty() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    [1, 2, 3].slice(5, 10).length
  ")).unwrap();
  assert_eq!(result, Value::Number(0f64));
}

// ========== Array splice 测试 ==========
#[test]
fn run_array_splice_remove() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3, 4, 5];
    arr.splice(2, 2).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("3,4")));
}

#[test]
fn run_array_splice_remove_updates_original() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3, 4, 5];
    arr.splice(2, 2);
    arr.join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,5")));
}

#[test]
fn run_array_splice_insert() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 5];
    arr.splice(2, 0, 3, 4);
    arr.join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,2,3,4,5")));
}

#[test]
fn run_array_splice_replace() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3, 4, 5];
    arr.splice(1, 2, 'a', 'b', 'c');
    arr.join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("1,a,b,c,4,5")));
}

#[test]
fn run_array_splice_negative_start() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3, 4, 5];
    arr.splice(-2, 1).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("4")));
}

#[test]
fn run_array_splice_no_delete_count() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3];
    arr.splice(1).join(',')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("2,3")));
}

#[test]
fn run_array_splice_empty_result() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let arr = [1, 2, 3];
    arr.splice(1, 0).length
  ")).unwrap();
  assert_eq!(result, Value::Number(0f64));
}

#[test]
fn run_array_big() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    var obj = {};
    obj.splice = Array.prototype.splice;
    obj[0] = 'x';
    obj[4294967295] = 'y';
    obj.length = 4294967296;
    var arr = obj.splice(4294967295, 1);
    let res = `objLength:${obj.length},arr:${arr}`;
    res
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("objLength:4294967295,arr:y")));

}
