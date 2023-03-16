use std::{cell::RefCell, rc::Rc};

use jsi::{JSI, value::Value, builtins::object::Object, ast_node::ClassType, error::JSIErrorType};

#[test]
fn run_function_name_and_length() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run(String::from("\n
  function abc(a, b, c) { } ;\n
  let foo = function(x, y) {};\n
  let obj = { x: function() {}};\n
  let res = {\n
    abc: { name: abc.name, length: abc.length},\n
    foo: { name: foo.name, length: foo.length},\n
    obj: { name: obj.x.name, length: obj.x.length},\n
  };\n
  res")).unwrap();
  let global_tmp = Rc::new(RefCell::new(Object::new(ClassType::Object,None)));
  match value {
    Value::Object(obj) => {
      let abc = (*obj).borrow().get_property_value(String::from("abc")).to_object(&global_tmp);
      let abc_name = (*abc).borrow().get_property_value(String::from("name"));
      let abc_length = (*abc).borrow().get_property_value(String::from("length"));
      assert_eq!(abc_name, Value::String(String::from("abc")));
      assert_eq!(abc_length, Value::Number(3f64));

      let foo = (*obj).borrow().get_property_value(String::from("foo")).to_object(&global_tmp);
      let foo_name = (*foo).borrow().get_property_value(String::from("name"));
      let foo_length = (*foo).borrow().get_property_value(String::from("length"));
      // bind let name
      assert_eq!(foo_name, Value::String(String::from("foo")));
      assert_eq!(foo_length, Value::Number(2f64));

      let x = (*obj).borrow().get_property_value(String::from("obj")).to_object(&global_tmp);
      let x_name = (*x).borrow().get_property_value(String::from("name"));
      let x_length = (*x).borrow().get_property_value(String::from("length"));
      // bind property name
      assert_eq!(x_name, Value::String(String::from("x")));
      assert_eq!(x_length, Value::Number(0f64));
    },
    _ => assert!(false, ""),
  };
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