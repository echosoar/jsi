use jsi::{JSI, value::Value};

#[test]
fn run_string() {
  let mut jsi = JSI::new();
  let result = jsi.run_with_bytecode(String::from("\
  let a = '123';
  let b = 'abc';
  a + b
  ")).unwrap();
  assert_eq!(result , Value::String(String::from("123abc")));
}

#[test]
fn run_string_object() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let a = String(123), b = new String('abc');
  a + b")).unwrap();
  assert_eq!(result , Value::String(String::from("123abc")));
}


#[test]
fn run_string_typeof() {
  let mut jsi = JSI::new();
  let result = jsi.run_with_bytecode(String::from("\
  typeof 'abc'")).unwrap();
  assert_eq!(result , Value::String(String::from("string")));
}

#[test]
fn run_string_xxx() {
  let mut jsi = JSI::new();
  let result = jsi.run_with_bytecode(String::from("\
  !('')")).unwrap();
  assert_eq!(result , Value::Boolean(true));
} 
// ========== String includes 测试 ==========
#[test]
fn run_string_includes_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.includes('world')
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn run_string_includes_not_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.includes('xyz')
  ")).unwrap();
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn run_string_includes_with_position() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.includes('world', 6)
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn run_string_includes_with_position_not_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.includes('hello', 6)
  ")).unwrap();
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn run_string_includes_empty_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello'.includes('')
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

// ========== String indexOf 测试 ==========
#[test]
fn run_string_index_of_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.indexOf('world')
  ")).unwrap();
  assert_eq!(result, Value::Number(6f64));
}

#[test]
fn run_string_index_of_not_found() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.indexOf('xyz')
  ")).unwrap();
  assert_eq!(result, Value::Number(-1f64));
}

#[test]
fn run_string_index_of_with_position() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello hello'.indexOf('lo', 4)
  ")).unwrap();
  assert_eq!(result, Value::Number(9f64));
}

#[test]
fn run_string_index_of_from_start() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'abcabc'.indexOf('a')
  ")).unwrap();
  assert_eq!(result, Value::Number(0f64));
}

#[test]
fn run_string_index_of_empty_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello'.indexOf('')
  ")).unwrap();
  assert_eq!(result, Value::Number(0f64));
}

// ========== String trim 测试 ==========
#[test]
fn run_string_trim_both_sides() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    '  hello  '.trim()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("hello")));
}

#[test]
fn run_string_trim_start() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    '   hello'.trim()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("hello")));
}

#[test]
fn run_string_trim_end() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello   '.trim()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("hello")));
}

#[test]
fn run_string_trim_no_whitespace() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello'.trim()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("hello")));
}

// ========== String startsWith 测试 ==========
#[test]
fn run_string_starts_with_true() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.startsWith('hello')
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn run_string_starts_with_false() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.startsWith('world')
  ")).unwrap();
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn run_string_starts_with_position() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.startsWith('world', 6)
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

// ========== String endsWith 测试 ==========
#[test]
fn run_string_ends_with_true() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.endsWith('world')
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn run_string_ends_with_false() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.endsWith('hello')
  ")).unwrap();
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn run_string_ends_with_position() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.endsWith('hello', 5)
  ")).unwrap();
  assert_eq!(result, Value::Boolean(true));
}

// ========== String slice 测试 ==========
#[test]
fn run_string_slice_basic() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.slice(0, 5)
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("hello")));
}

#[test]
fn run_string_slice_no_end() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.slice(6)
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("world")));
}

#[test]
fn run_string_slice_negative() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.slice(-5)
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("world")));
}

#[test]
fn run_string_slice_negative_both() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.slice(-11, -6)
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("hello")));
}

// ========== String toLowerCase 测试 ==========
#[test]
fn run_string_to_lower_case() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'HELLO World'.toLowerCase()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("hello world")));
}

#[test]
fn run_string_to_lower_case_already_lower() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello'.toLowerCase()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("hello")));
}

// ========== String toUpperCase 测试 ==========
#[test]
fn run_string_to_upper_case() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello World'.toUpperCase()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("HELLO WORLD")));
}

#[test]
fn run_string_to_upper_case_already_upper() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'HELLO'.toUpperCase()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("HELLO")));
}

// ========== String split 测试 ==========
#[test]
fn run_string_split_by_comma() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'a,b,c'.split(',').join('-')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("a-b-c")));
}

#[test]
fn run_string_split_by_space() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.split(' ').length
  ")).unwrap();
  assert_eq!(result, Value::Number(2f64));
}

#[test]
fn run_string_split_empty_separator() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello'.split('').join('-')
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("h-e-l-l-o")));
}

#[test]
fn run_string_split_no_separator() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello world'.split().length
  ")).unwrap();
  assert_eq!(result, Value::Number(1f64));
}

#[test]
fn run_string_split_no_match() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    'hello'.split('x').length
  ")).unwrap();
  assert_eq!(result, Value::Number(1f64));
}
