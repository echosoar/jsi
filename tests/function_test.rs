use jsi::{JSI, value::Value};

#[test]
fn run_function() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run(String::from("function abc(a, b, c) { return a +b - c;} abc(123, 456, 789);"));
  assert_eq!(value, Value::Number(-210f64)) // 123 + 456 - 789 = -210
}

#[test]
fn run_function_name_and_length() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run(String::from("\
  function abc(a, b, c) { } ;\
  let foo = function(x, y) {};\
  let obj = { x: function() {}};\
  {\
    abc: { name: abc.name, length: abc.length},\
    foo: { name: foo.name, length: foo.length},\
    obj: { name: obj.x.name, length: obj.x.length},\
  };"));
  match value {
    Value::Object(obj) => {
      let abc = (*obj).borrow().get_property(String::from("abc")).to_object();
      let abc_name = (*abc).borrow().get_property(String::from("name"));
      let abc_length = (*abc).borrow().get_property(String::from("length"));
      assert_eq!(abc_name, Value::String(String::from("abc")));
      assert_eq!(abc_length, Value::Number(3f64));

      let foo = (*obj).borrow().get_property(String::from("foo")).to_object();
      let foo_name = (*foo).borrow().get_property(String::from("name"));
      let foo_length = (*foo).borrow().get_property(String::from("length"));
      // bind let name
      assert_eq!(foo_name, Value::String(String::from("foo")));
      assert_eq!(foo_length, Value::Number(2f64));

      let x = (*obj).borrow().get_property(String::from("obj")).to_object();
      let x_name = (*x).borrow().get_property(String::from("name"));
      let x_length = (*x).borrow().get_property(String::from("length"));
      // bind property name
      assert_eq!(x_name, Value::String(String::from("x")));
      assert_eq!(x_length, Value::Number(0f64));
    },
    _ => assert!(false, ""),
  };
}