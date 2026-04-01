use jsi::{JSI, value::Value};


#[test]
fn run_binary_equal() {
  let check_list = vec![
    // same type
    ("1 == 1", true),
    ("1 != 1", false),
    ("0 == 1", false),
    ("0 != 1", true),
    ("true == true", true),
    ("true != true", false),
    ("false == true", false),
    ("false != true", true),
    ("'123' == '123'", true),
    ("'123' != '123'", false),
    ("'124' == '123'", false),
    ("'124' != '123'", true),
    ("null == null", true),
    ("null != null", false),
    ("undefined == undefined", true),
    ("undefined != undefined", false),
    // diff type
    ("1 == true", true),
    ("1 != true", false),
    ("1 === true", false), // strict equal
    ("1 !== true", true), // strict not equal
    ("0 == false", true),
    ("0 != false", false),
    ("0 === false", false), // strict equal
    ("0 !== false", true), // strict not equal
    ("2 == true", false),
    ("2 != true", true),
    ("0 == 1", false),
    ("'0' == 0", true),
    ("'0' != 0", false),
    ("'0' === 0", false), // strict equal
    ("'0' !== 0", true), // strict equal
    ("'1' == 0", false),
    ("123 == '123'", true),
    ("123 === '123'", false), // strict equal
    ("123 == '124'", false),
    ("null == undefined", true),
    ("null != undefined", false),
    ("null === undefined", false), // strict equal
    ("null !== undefined", true), // strict equal
    ("'1' == true", true),
    ("'1' === true", false),  // strict equal
    ("'true' == true", false),
    ("'0' == false", true),
    ("'0' === false", false),  // strict equal
    ("'false' == false", false),
    ("!0", true),
    ("!1", false),
    ("!''", true),
    ("!'0'", false),
    ("!false", true),
    ("!true", false),
    ("++1 === 2", true),
    ("--2 === 1", true),
    // TODO: Object/Array/Function
    
  ];
  let mut jsi = JSI::new();
  for check_item in check_list {
    assert_eq!(jsi.run_with_bytecode(String::from(check_item.0)).unwrap(), Value::Boolean(check_item.1), "expr: {:?}", check_item.0);
  }
  
}

#[test]
fn run_binary_bitwise() {
  let check_list: Vec<(&str, f64)> = vec![
    // <<
    ("1 << 0", 1.0),
    ("1 << 1", 2.0),
    ("1 << 3", 8.0),
    ("5 << 1", 10.0),
    ("-1 << 0", -1.0),
    // >>
    ("8 >> 1", 4.0),
    ("8 >> 3", 1.0),
    ("-8 >> 1", -4.0),
    // >>>
    ("8 >>> 1", 4.0),
    ("-1 >>> 0", 4294967295.0),
    ("-8 >>> 1", 2147483644.0),
    // |
    ("0 | 0", 0.0),
    ("1 | 0", 1.0),
    ("1 | 2", 3.0),
    ("5 | 3", 7.0),
    // ^
    ("0 ^ 0", 0.0),
    ("1 ^ 1", 0.0),
    ("5 ^ 3", 6.0),
    ("15 ^ 9", 6.0),
    // &
    ("0 & 0", 0.0),
    ("1 & 1", 1.0),
    ("5 & 3", 1.0),
    ("15 & 9", 9.0),
  ];
  let mut jsi = JSI::new();
  for check_item in check_list {
    assert_eq!(jsi.run_with_bytecode(String::from(check_item.0)).unwrap(), Value::Number(check_item.1), "expr: {:?}", check_item.0);
  }
}