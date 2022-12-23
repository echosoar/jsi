use jsi::JSI;

#[test]
fn jsi_new() {
  let jsi_vm = JSI::new();
  jsi_vm.run(String::from("let test = 123;let test2 = 'abc'; console.log(1 > 2 ? test : test2"));
}