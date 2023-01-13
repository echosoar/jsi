use jsi::JSI;


#[test]
fn ast_class() {
  let mut jsi_vm = JSI::new();
  let program= jsi_vm.parse(String::from("class A {}"));
  println!("program {:?}", program);
  // assert_eq!(value,Value::Number(20f64));
}