use std::{rc::Rc};
use crate::constants::{GLOBAL_PROMISE_NAME, PROTO_PROPERTY_NAME};
use crate::context::{Context};
use crate::error::{JSIError, JSIErrorType};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, error::JSIResult};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Property}, function::builtin_function};

pub fn create_promise(ctx: &mut Context, init: Value) -> Value {
    let global_promise = get_global_object_by_name(ctx, GLOBAL_PROMISE_NAME);
    let promise = create_object(ctx, ClassType::Promise, None);
    {
        let promise_rc = Rc::clone(&promise);
        let mut promise_mut = promise_rc.borrow_mut();
        promise_mut.constructor = Some(Rc::downgrade(&global_promise));
    }

    Value::Promise(promise)
}


// 全局构造方法
pub fn bind_global_promise(ctx: &mut Context) {
  let global_promise = get_global_object_by_name(ctx, GLOBAL_PROMISE_NAME);
  let mut global_promise_borrowed = (*global_promise).borrow_mut();
  let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
  global_promise_borrowed.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
}

// 创建 Promise 实例
fn create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let mut param = Value::Undefined;
  if args.len() == 0 {
    return Err(JSIError::new(JSIErrorType::TypeError, "Promise resolver is not a function".to_string(), 0, 0));
  }
    let executor = &args[0];
    if !matches!(executor, Value::Function(_)) {
        return Err(JSIError::new(JSIErrorType::TypeError, "Promise resolver is not a function".to_string(), 0, 0));
    }

  Ok(create_promise(call_ctx.ctx, executor.to_owned()))
}