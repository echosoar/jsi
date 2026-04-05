use jsi::{JSI, value::Value, error::JSIErrorType};


#[test]
fn run_function_base() {
  let mut jsi_vm = JSI::new();
  let value = jsi_vm.run_with_bytecode(String::from("\n
  function add(x, y) {
    return x * 2 + y;
  };
  add(1, 'a')")).unwrap();
  assert_eq!(value , Value::String(String::from("2a")));
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
// ========== this 相关测试 ==========

#[test]
fn run_function_this_in_object_method() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let obj = {
      name: 'test',
      getName: function() {
        return this.name;
      }
    };
    obj.getName()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("test")));
}

#[test]
fn run_function_this_in_nested_object() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let outer = {
      value: 100,
      inner: {
        value: 200,
        getValue: function() {
          return this.value;
        }
      }
    };
    outer.inner.getValue()
  ")).unwrap();
  assert_eq!(result, Value::Number(200f64));
}

#[test]
fn run_function_this_in_constructor() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    function Person(name, age) {
      this.name = name;
      this.age = age;
    }
    let p = new Person('Alice', 25);
    p.name + ':' + p.age
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("Alice:25")));
}

#[test]
fn run_function_this_in_arrow_function() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let obj = {
      name: 'arrow',
      getName: function() {
        return this.name;
      }
    };
    obj.getName()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("arrow")));
}

#[test]
fn run_function_this_prototype_chain() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    function Counter() {
      this.count = 0;
    }
    Counter.prototype.increment = function() {
      this.count++;
      return this;
    };
    Counter.prototype.getCount = function() {
      return this.count;
    };
    let c = new Counter();
    c.increment().increment().getCount()
  ")).unwrap();
  assert_eq!(result, Value::Number(2f64));
}

#[test]
fn run_function_this_method_chain() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let calc = {
      value: 0,
      add: function(n) {
        this.value += n;
        return this;
      },
      subtract: function(n) {
        this.value -= n;
        return this;
      },
      getValue: function() {
        return this.value;
      }
    };
    calc.add(10).subtract(3).getValue()
  ")).unwrap();
  assert_eq!(result, Value::Number(7f64));
}

#[test]
fn run_function_this_in_callback() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let obj = {
      multiplier: 2,
      numbers: [1, 2, 3],
      sum: function() {
        let self = this;
        let total = 0;
        let arr = self.numbers;
        for (let i = 0; i < arr.length; i++) {
          total += arr[i] * self.multiplier;
        }
        return total;
      }
    };
    obj.sum()
  ")).unwrap();
  assert_eq!(result, Value::Number(12f64));
}

#[test]
fn run_function_this_with_computed_property() {
  let mut jsi = JSI::new();
  let result = jsi.run(String::from("\
    let key = 'dynamic';
    let obj = {
      [key]: 'computed value',
      getKey: function() {
        return this[key];
      }
    };
    obj.getKey()
  ")).unwrap();
  assert_eq!(result, Value::String(String::from("computed value")));
}
