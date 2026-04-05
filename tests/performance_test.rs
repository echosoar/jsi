use jsi::{JSI, value::Value};

#[test]
fn run_fibonacci_performance() {
  let mut jsi = JSI::new();
  let start_time = std::time::Instant::now();
  let result = jsi.run(String::from("\
    function fibonacci(n) {
      if (n <= 1) return n;
      return fibonacci(n - 1) + fibonacci(n - 2);
    }

    let result = {};
    for (let i = 0; i < 15; i++) {
      let value = fibonacci(i);
      result[`run_${i}`] = value;
    }

    let keys = Object.keys(result).map(key => key + result[key]).join(',');
    keys
  ")).unwrap();
  let duration = start_time.elapsed();
  println!("Fibonacci performance test completed in: {:?}", duration);

  assert_eq!(result, Value::String(String::from("run_00,run_11,run_21,run_32,run_43,run_55,run_68,run_713,run_821,run_934,run_1055,run_1189,run_12144,run_13233,run_14377")));
}