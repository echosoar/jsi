use jsi::JSI;

fn main() {
    // Test without console.log
    let code = r#"
var obj = { toString: function() { return "test"; } };
[obj, obj].sort();
"#;

    let mut jsi = JSI::new();
    let result = jsi.run(String::from(code));
    println!("Result: {:?}", result);
}
