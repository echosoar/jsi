
use jsi::{JSI, value::Value};

#[test]
fn run_if_else() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let a = false;
  let b = 0;
  if (a) {
    b = 1;
  } else {
    b = 2;
  }\n
  b"));
  assert_eq!(result , Value::Number(2f64));
}

#[test]
fn run_for() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let a = [];
  for(let i = 0; i < 3; i++) {
    a.push(i);
  }\n
  a.join(':')"));
  assert_eq!(result , Value::String(String::from("0:1:2")));
}

#[test]
fn run_for_break_continue() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  let a = [];
  outer:
  for(let i = 0; i < 3; i++) {
    for(let j = 0; j < 5; j++) {
      if (j == 1 && i == 1) {
        continue outer
      }
      if (j == 4) {
        break
      }
      if (j == 3 && i == 2) {
        break outer
      }
      a.push(i * j);
    }\n
  }\n
  a.join(':')"));
  assert_eq!(result , Value::String(String::from("0:0:0:0:0:0:2:4")));
}