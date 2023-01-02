use jsi::JSI;

#[test]
fn jsi_new() {
  let mut jsi_vm = JSI::new();
  // jsi_vm.run(String::from("let test = 123.456;let test2 = 'abc'; console.log(1 + 3 > 2 ? test : test2, false)"));
  // jsi_vm.run(String::from("let test = 123.456;let test2 = 'abc'; 1 + true - 4 * 2 / 1.3 % 2 + 32;"));
  jsi_vm.run(String::from("function abc(a, b, c) { return b + c;} abc(123, 456, 789);"));
}