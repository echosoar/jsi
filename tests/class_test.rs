use jsi::JSI;


#[test]
fn ast_class() {
  let mut jsi_vm = JSI::new();
  let program= jsi_vm.parse(String::from("class A {\n
    private xxx = 213;\n
    constructor(arg) {\n
        this.a = arg;
    }
    
    async func() {
      return new Promise(resolve => {
        resolve(this.a);
      })
    }
  }"));
  println!("program {:?}", program);
  // assert_eq!(value,Value::Number(20f64));
}