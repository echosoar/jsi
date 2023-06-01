use std::{rc::Rc, cell::RefCell};

use crate::{context::{Context}, ast_node::{ClassType, CallContext}, error::JSIResult};
use super::{object::{create_object, Property, Object},function::builtin_function};
use crate::{value::{Value}};
pub fn create_console(ctx: &mut Context) -> Rc<RefCell<Object>> {
  let console_obj = create_object(ctx, ClassType::Object, None);
  let console_rc = Rc::clone(&console_obj);
  let mut console = console_rc.borrow_mut();
  // console.log
  let name = String::from("log");
  console.property.insert(name.clone(), Property { enumerable: true, value: builtin_function(ctx, name, 0f64, console_log) });
  console_obj
}


// console.log
fn console_log(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  println!("args {:?}", args);
  let mut strs: Vec<String> = vec![];
  for arg in args.iter() {
    strs.push(arg.to_string(call_ctx.ctx));
  }
  println!("{}", strs.join(" "));
  Ok(Value::Undefined)
}