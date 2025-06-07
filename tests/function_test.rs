use jsi::{JSI, value::Value, error::JSIErrorType};


#[test]
fn run_function_base() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run_with_bytecode(String::from("\n
  function add(x, y) {
    function inner(x, y) {
      return x + y;
    };
    return inner(x, y);
  };
  add(1, 'a')")).unwrap();
  assert_eq!(value , Value::String(String::from("1a")));
}


#[test]
fn run_function_scope1() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run(String::from("\n
  let fun1 = function(x, y) {
    let a = 123;
    return fun2();
  };\n
  let fun2 = function() {
    return a;
  };\n
  fun1()"));
  if let Err(jsi_error) = value {
    assert_eq!(jsi_error.error_type, JSIErrorType::ReferenceError);
    assert_eq!(jsi_error.message , String::from("a is not defined"));
  } else {
    assert!(false , "need TypeError");
  }
}

#[test]
fn run_function_scope2() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run(String::from("\n
  let a = 123;
  let fun = function() {
    return a;
  };\n
  fun()")).unwrap();
  assert_eq!(value , Value::Number(123f64));
}

#[test]
fn run_function_instances_has_class() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  function func() {}\n
  Object.prototype.toString.call(func)")).unwrap();
  assert_eq!(result , Value::String(String::from("[object Function]")));
}

#[test]
fn run_function_typeof() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
var check=0;
while(function f(){}){ 
  if(typeof(f) === 'function') {
    check = -1;
    break; 
  } else {
    check = 1;
    break; 
  }
}check.toString() + typeof function() {}")).unwrap();
  assert_eq!(result , Value::String(String::from("1function")));
}

#[test]
fn run_arrow_function() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  let result = jsi.run(String::from("\
  let a = (a, b ,c) => {
    return '1' + a + b + c;
  }
  let b = b => {
    return '2' + b;
  };
  let c = c => c + '3';
  let d = (d,d) => [arguments.length, arguments[0], arguments[1], d, '4'].join();
  a(1, 'a', false) + a.name + b(2) + b.name + c(3) + c.name + d(4,5);")).unwrap();
  assert_eq!(result , Value::String(String::from("11afalsea22b33c2,4,5,5,4")));
}

#[test]
fn run_new_function() {
  let mut jsi = JSI::new();
  jsi.set_strict(false);
  let result = jsi.run(String::from("\
  let a = function(a, b ,c) {
    this.name = a + b + c;
  }
  a.prototype.age = 456;
  let b = new a(1,'2', false);
  let c = a;
  let d = a.bind(123);
  b.age + b.name + (a === c) + (a === d);
  ")).unwrap();
  assert_eq!(result , Value::String(String::from("45612falsetruefalse")));
}