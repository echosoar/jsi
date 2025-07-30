use std::vec;
use std::{rc::Rc};
use crate::constants::{GLOBAL_PROMISE_NAME, PROTO_PROPERTY_NAME};
use crate::context::{Context};
use crate::error::{JSIError, JSIErrorType};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, error::JSIResult};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Property}, function::builtin_function};

pub const PROMISE_STATE: &str = "[[PromiseState]]";
pub const PROMISE_FULFILLED_VALUE: &str = "[[PromiseFulfilledValue]]";
pub const PROMISE_REJECTED_REASON: &str = "[[PromiseRejectedReason]]";

pub fn create_promise(ctx: &mut Context, init: Value) -> Value {
    let global_promise = get_global_object_by_name(ctx, GLOBAL_PROMISE_NAME);
    let promise = create_object(ctx, ClassType::Promise, None);
    {
        let promise_rc = Rc::clone(&promise);
        let mut promise_mut = promise_rc.borrow_mut();
        promise_mut.constructor = Some(Rc::downgrade(&global_promise));
        promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("pending".to_string()));
    }

    let resolve_fn = builtin_function(ctx, "resolve".to_string(), 1f64, resolve);
    if let Value::Function(resolve_function) = &resolve_fn {
        let mut resolve_function_mut = resolve_function.borrow_mut();
        resolve_function_mut.set_inner_property_value(String::from("promise"), Value::RefObject(Rc::downgrade(&promise)));
    }

    let reject_fn = builtin_function(ctx, "reject".to_string(), 1f64, reject);
    if let Value::Function(reject_function) = &reject_fn {
        let mut reject_function_mut = reject_function.borrow_mut();
        reject_function_mut.set_inner_property_value(String::from("promise"), Value::RefObject(Rc::downgrade(&promise)));
    }

    if let Value::Function(init_function) = init {
        let mut call_ctx = CallContext {
            ctx,
            // TODO:
            this: Value::Undefined,
            reference: None,
            func_name: String::from(""),
        };
        call_ctx.call_function(init_function, None, None, vec![resolve_fn, reject_fn]).unwrap();
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
  if args.len() == 0 {
    return Err(JSIError::new(JSIErrorType::TypeError, "Promise resolver is not a function".to_string(), 0, 0));
  }
    let executor = &args[0];
    if !matches!(executor, Value::Function(_)) {
        return Err(JSIError::new(JSIErrorType::TypeError, "Promise resolver is not a function".to_string(), 0, 0));
    }

  Ok(create_promise(call_ctx.ctx, executor.to_owned()))
}


// resolve 方法
fn resolve(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {

    let resolve_fn = call_ctx.reference.as_ref().and_then(|r| r.upgrade()).expect("resolve rc error");
    
    let promise = resolve_fn.borrow().get_inner_property_value(String::from("promise")).unwrap();

    if let Value::RefObject(promise_rc_weak) = promise {
        if let Some(promise_rc) = promise_rc_weak.upgrade() {
            let mut promise_mut = promise_rc.borrow_mut();
            // 设置 Promise 的状态为 fulfilled
            promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("fulfilled".to_string()));
            // 处理 resolve 的值
            let value = args.get(0).cloned().unwrap_or(Value::Undefined);
            promise_mut.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), value);
        }
    }

    Ok(Value::Undefined)
}

// reject 方法
fn reject(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let reject_fn = call_ctx.reference.as_ref().and_then(|r| r.upgrade()).expect("reject rc error");
    
    let promise = reject_fn.borrow().get_inner_property_value(String::from("promise")).unwrap();

    if let Value::RefObject(promise_rc_weak) = promise {
        if let Some(promise_rc) = promise_rc_weak.upgrade() {
            let mut promise_mut = promise_rc.borrow_mut();
            // 设置 Promise 的状态为 rejected
            promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("rejected".to_string()));
            // 处理 reject 的原因
            let reason = args.get(0).cloned().unwrap_or(Value::Undefined);
            promise_mut.set_inner_property_value(PROMISE_REJECTED_REASON.to_string(), reason);
        }
    }

   Ok(Value::Undefined)
}