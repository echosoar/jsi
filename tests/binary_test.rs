use jsi::{JSI, value::Value};


#[test]
fn run_binary_equal() {
  let check_list = vec![
    // same type
    ("1 == 1", true),
    ("0 == 1", false),
    ("true == true", true),
    ("false == true", false),
    ("'123' == '123'", true),
    ("'124' == '123'", false),
    ("null == null", true),
    ("undefined == undefined", true),
    // diff type
    ("1 == true", true),
    ("1 === true", false), // strict equal
    ("0 == false", true),
    ("0 === false", false), // strict equal
    ("2 == true", false),
    ("0 == 1", false),
    ("'0' == 0", true),
    ("'0' === 0", false), // strict equal
    ("'1' == 0", false),
    ("123 == '123'", true),
    ("123 === '123'", false), // strict equal
    ("123 == '124'", false),
    ("null == undefined", true),
    ("null === undefined", false), // strict equal
    ("'1' == true", true),
    ("'1' === true", false),  // strict equal
    ("'true' == true", false),
    ("'0' == false", true),
    ("'0' === false", false),  // strict equal
    ("'false' == false", false),
    // TODO: Object/Array/Function
    
  ];
  let mut jsi = JSI::new();
  for check_item in check_list {
    assert_eq!(jsi.run(String::from(check_item.0)), Value::Boolean(check_item.1), "expr: {:?}", check_item.0);
  }
  
}