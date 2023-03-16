use jsi::{JSI, value::Value};

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
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
  for (var i = 0; i <= 1000; i++)
  {
    var x = i / 10.0;
  
    assert.sameValue(
      Math.round(x),
      Math.floor(x + 0.5),
      'Math.round(i / 10.0) must return the same value returned by Math.floor(x + 0.5)'
    );
  }
  
  for (i = -5; i >= -1000; i--)
  {
    if (i === -5)
    {
      x = -0.500000000000001;
    } else
    {
      x = i / 10.0;
    }
  
    assert.sameValue(
      Math.round(x),
      Math.floor(x + 0.5),
      'Math.round(i / 10.0) must return the same value returned by Math.floor(x + 0.5)'
    );
  }
  

  "));
  println!("result: {:?}", result)
}

