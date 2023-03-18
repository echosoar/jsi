use jsi::{JSI, value::Value};
use yaml_rust::YamlLoader;

#[test]
fn run_string() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
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
fn run_xxx() {
  // let mut jsi = JSI::new();
  // let result = jsi.run(String::from("\
  // do function g() {} while (false)
  // "));
  // println!("result: {:?}", result)
  let s ="
  description: redeclaration with AsyncGeneratorDeclaration (AsyncFunctionDeclaration in BlockStatement)
  esid: sec-block-static-semantics-early-errors
  features: [async-iteration, async-functions]
  flags: [generated]
  negative:
    phase: parse
    type: SyntaxError
  info: |
      Block : { StatementList }
  
      It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains
      any duplicate entries.
  
  ";
      let docs = YamlLoader::load_from_str(s).unwrap();
      println!("{:?}", docs[0]["negativex"]);
}

