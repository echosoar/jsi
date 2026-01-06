use std::f32::consts::E;
use std::vec;
use std::{rc::Rc};
use std::cell::{RefCell};
use super::object::Object;
use crate::builtins::array::create_array;
use crate::constants::{GLOBAL_PROMISE_NAME, PROTO_PROPERTY_NAME};
use crate::context::{Context};
use crate::error::{JSIError, JSIErrorType};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, error::JSIResult};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Property}, function::builtin_function};

pub const PROMISE_STATE: &str = "[[PromiseState]]";
pub const PROMISE_FULFILLED_VALUE: &str = "[[PromiseFulfilledValue]]";
pub const PROMISE_REJECTED_REASON: &str = "[[PromiseRejectedReason]]";
pub const PROMISE_FULFILLED_REACTIONS: &str = "[[PromiseFulfilledReactions]]";
pub const PROMISE_REJECTED_REACTIONS: &str = "[[PromiseRejectedReactions]]";

pub fn create_promise(ctx: &mut Context, init: Value) -> Value {
    let (promise, resolve_fn, reject_fn) = create_promise_helper(ctx);

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


pub fn create_promise_helper(ctx: &mut Context) -> (Rc<RefCell<Object>>, Value, Value) {
    let global_promise = get_global_object_by_name(ctx, GLOBAL_PROMISE_NAME);
    let promise = create_object(ctx, ClassType::Promise, None);
    {
        let promise_rc = Rc::clone(&promise);
        let mut promise_mut = promise_rc.borrow_mut();
        promise_mut.constructor = Some(Rc::downgrade(&global_promise));
        promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("pending".to_string()));

        // 创建数组，用来存放 pending 状态的 then 回调
        let fulfilled_callbacks = create_array(ctx, 0);
        promise_mut.set_inner_property_value(PROMISE_FULFILLED_REACTIONS.to_string(), fulfilled_callbacks);

        let rejected_callbacks = create_array(ctx, 0);
        promise_mut.set_inner_property_value(PROMISE_REJECTED_REACTIONS.to_string(), rejected_callbacks);


        // 绑定原型链
        let promise_proto = get_global_object_prototype_by_name(ctx, GLOBAL_PROMISE_NAME);
        promise_mut.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&promise_proto)));
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

    return (promise, resolve_fn, reject_fn);
}

// 全局构造方法
pub fn bind_global_promise(ctx: &mut Context) {
  let global_promise = get_global_object_by_name(ctx, GLOBAL_PROMISE_NAME);
  let mut global_promise_borrowed = (*global_promise).borrow_mut();
  let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
  global_promise_borrowed.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);

  let resolve_name = String::from("resolve");
  global_promise_borrowed.property.insert(resolve_name.clone(), Property { enumerable: true, value: builtin_function(ctx, resolve_name, 1f64, resolve_static) });

    let reject_name = String::from("reject");
    global_promise_borrowed.property.insert(reject_name.clone(), Property { enumerable: true, value: builtin_function(ctx, reject_name, 1f64, reject_static) });

    // 原型方法 then
    if let Some(props) = &global_promise_borrowed.prototype {
        let prototype_rc = Rc::clone(props);
        let mut prototype_mut = prototype_rc.borrow_mut();
        prototype_mut.define_builtin_function_property(ctx, String::from("then"), 2, then);
    }
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
            promise_mut.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), value.clone());

            // 执行所有 resolve 回调
            let all_reactions = promise_mut.get_inner_property_value(PROMISE_FULFILLED_REACTIONS.to_string()).unwrap();
            exec_all_reactions(call_ctx.ctx, all_reactions, value, true);

             // 清空 fulfilled 和 rejected 回调数组
            promise_mut.set_inner_property_value(PROMISE_FULFILLED_REACTIONS.to_string(), Value::Undefined);
            promise_mut.set_inner_property_value(PROMISE_REJECTED_REACTIONS.to_string(), Value::Undefined);
            // 清空 parent 引用，防止循环引用
            promise_mut.set_inner_property_value(String::from("parent"), Value::Undefined);
        }
    }

    Ok(Value::Undefined)
}

// reject 方法
fn reject(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let reject_fn = call_ctx.reference.as_ref().and_then(|r| r.upgrade()).expect("reject rc error");
    let promise = reject_fn.borrow().get_inner_property_value(String::from("promise")).unwrap();
    if let Value::RefObject(promise_rc_weak) = promise {
        let upgrade = promise_rc_weak.upgrade();
        if let Some(promise_rc) = upgrade {
            let mut promise_mut = promise_rc.borrow_mut();
            // 设置 Promise 的状态为 rejected
            promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("rejected".to_string()));
            // 处理 reject 的原因
            let reason = args.get(0).cloned().unwrap_or(Value::Undefined);
            promise_mut.set_inner_property_value(PROMISE_REJECTED_REASON.to_string(), reason.clone());

            // 执行所有 reject 回调
            let all_reactions = promise_mut.get_inner_property_value(PROMISE_REJECTED_REACTIONS.to_string()).unwrap();
            
            exec_all_reactions(call_ctx.ctx, all_reactions, reason, false);

            // 清空 fulfilled 和 rejected 回调数组
            promise_mut.set_inner_property_value(PROMISE_FULFILLED_REACTIONS.to_string(), Value::Undefined);
            promise_mut.set_inner_property_value(PROMISE_REJECTED_REACTIONS.to_string(), Value::Undefined);
            // 清空 parent 引用，防止循环引用
            promise_mut.set_inner_property_value(String::from("parent"), Value::Undefined);
            
        }
    }

   Ok(Value::Undefined)
}

fn then(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let on_fulfilled = args.get(0).cloned().unwrap_or(Value::Undefined);
    let on_rejected = args.get(1).cloned().unwrap_or(Value::Undefined);
    // then 的逻辑是返回信的 Promise
    // let result_promise = create_promise(call_ctx.ctx, Value::Undefined);
    // 获取 result 的 fulfilled 和 rejected 回调数组
    let (result_promise, new_resolve_fn, new_reject_fn) = create_promise_helper(call_ctx.ctx);

    let this_promise_obj = get_promise_object_from_this(&call_ctx.this).unwrap();
    let state = {
        let this_promise = this_promise_obj.borrow();
        this_promise.get_inner_property_value(PROMISE_STATE.to_string()).unwrap()
    };

    {
        // 把 this 和 result promise 关联起来
        let mut result_promise_mut = result_promise.borrow_mut();
        result_promise_mut.set_inner_property_value(String::from("parent"), Value::Promise(Rc::clone(&this_promise_obj)));
    }

    if let Value::String(state_str) = state {
        if state_str == String::from("fulfilled") {
            let this_promise = this_promise_obj.borrow();
            let fulfilled_value = this_promise.get_inner_property_value(PROMISE_FULFILLED_VALUE.to_string()).unwrap_or(Value::Undefined);
            execute_promise_reaction(call_ctx.ctx, on_fulfilled, fulfilled_value, vec![new_resolve_fn, new_reject_fn], true);
        } else if state_str == String::from("rejected") {
            let this_promise = this_promise_obj.borrow();
            let rejected_reason = this_promise.get_inner_property_value(PROMISE_REJECTED_REASON.to_string()).unwrap_or(Value::Undefined);
            execute_promise_reaction(call_ctx.ctx, on_rejected, rejected_reason, vec![new_resolve_fn, new_reject_fn], false);
        } else  if state_str == String::from("pending") {
            // 把  on_fulfilled 和 on_rejected 存储起来，并且执行 new_resolve_fn 和 new_reject_fn
            add_to_promise_reactions(this_promise_obj, on_fulfilled, on_rejected, new_resolve_fn, new_reject_fn);
        }
    }

    return Ok(Value::Promise(result_promise));
}

fn add_to_promise_reactions(this_promise_obj: &Rc<RefCell<Object>>, on_fulfilled:Value, on_rejected:Value, new_resolve_fn:Value, new_reject_fn:Value) {
    let this_promise = this_promise_obj.borrow();
    let fulfilled_reactions = this_promise.get_inner_property_value(PROMISE_FULFILLED_REACTIONS.to_string()).unwrap();
    if let Value::Array(fulfill_array) = fulfilled_reactions {
        let mut fulfill_array_mut = fulfill_array.borrow_mut();
        let length = fulfill_array_mut.get_inner_property_value("length".to_string()).unwrap_or(Value::Number(0f64));
        if let Value::Number(len) = length {
            fulfill_array_mut.set_inner_property_value(len.to_string(), on_fulfilled.clone());
            // ${len}_rsolve_fn
            fulfill_array_mut.set_inner_property_value(format!("{}_resolve_fn", len), new_resolve_fn.clone());
            fulfill_array_mut.set_inner_property_value(format!("{}_reject_fn", len), new_reject_fn.clone());
            fulfill_array_mut.set_inner_property_value("length".to_string(), Value::Number(len + 1f64));
        }
    }
    let rejected_reactions = this_promise.get_inner_property_value(PROMISE_REJECTED_REACTIONS.to_string()).unwrap();
    if let Value::Array(reject_array) = rejected_reactions {
        let mut reject_array_mut = reject_array.borrow_mut();
        let length = reject_array_mut.get_inner_property_value("length".to_string()).unwrap_or(Value::Number(0f64));
        if let Value::Number(len) = length {
            reject_array_mut.set_inner_property_value(len.to_string(), on_rejected.clone());
            // ${len}_rsolve_fn
            reject_array_mut.set_inner_property_value(format!("{}_resolve_fn", len), new_resolve_fn.clone());
            reject_array_mut.set_inner_property_value(format!("{}_reject_fn", len), new_reject_fn.clone());
            reject_array_mut.set_inner_property_value("length".to_string(), Value::Number(len + 1f64));
        }
    }
}

fn exec_all_reactions(ctx: &mut Context, reactions_array_obj: Value, value: Value, is_fulfilled: bool) {
    if let Value::Array(reactions_array) = reactions_array_obj {
        let reactions_borrowed = reactions_array.borrow();
        if let Some(length_prop) = reactions_borrowed.get_inner_property_value("length".to_string()) {
            if let Value::Number(length) = length_prop {
                for i in 0..(length as usize) {
                    let handler = reactions_borrowed.get_inner_property_value(i.to_string()).unwrap_or(Value::Undefined);
                    let next_resolve = reactions_borrowed.get_inner_property_value(format!("{}_resolve_fn", i)).unwrap_or(Value::Undefined);
                    let next_reject = reactions_borrowed.get_inner_property_value(format!("{}_reject_fn", i)).unwrap_or(Value::Undefined);
                    execute_promise_reaction(ctx, handler, value.clone(), vec![next_resolve, next_reject], is_fulfilled);
                }
            }
        }
    }
}

// 执行 Promise 的回调
// then_handler 代表传入到 then 的回调方法，is_fulfilled 为 true 代表执行 onFulfilled，否则执行 onRejected
fn execute_promise_reaction(ctx: &mut Context, then_handler: Value, value: Value, next_resolve_reject: Vec<Value>, is_fulfilled: bool) {
    // 执行完 then 回调后的返回值
    let returned_data = if let Value::Undefined = then_handler {
        if is_fulfilled {
            Ok(value.clone())
        } else {
            Err(JSIError::new(JSIErrorType::TypeError, "".to_string(), 0, 0))
        }
    } else if let Value::Function(then_handler_fun) = then_handler {
        let mut call_ctx = CallContext {
            ctx,
            this: Value::Undefined,
            reference: None,
            func_name: String::from("then_handler_fun"),
        };
        call_ctx.call_function(then_handler_fun, None, None, vec![value.clone()])
    } else {
        Ok(value.clone())
    };

    match returned_data {
        Ok(returned_value) => {
            if let Value::Promise(current_promise) = returned_value {
                // 根据 promise 的状态，调用下一个 promise 的 resolve 或 reject 方法
                let state = {
                    let curren_promise_ref = current_promise.borrow();
                    curren_promise_ref.get_inner_property_value(String::from("[[PromiseState]]")).unwrap()
                };
                if let Value::String(state_str) = state {
                    if state_str == String::from("fulfilled") {
                        let curren_promise_ref = current_promise.borrow();
                        let fulfilled_value = curren_promise_ref.get_inner_property_value(String::from("[[PromiseFulfilledValue]]")).unwrap_or(Value::Undefined);
                        // 调用下一个 promise 的 resolve 方法
                        if let Some(next_resolve) = next_resolve_reject.get(0) {
                            if let Value::Function(next_resolve_fun) = next_resolve {
                                let mut call_ctx = CallContext {
                                    ctx,
                                    this: Value::Undefined,
                                    reference: Some(Rc::downgrade(next_resolve_fun)),
                                    func_name: String::from("next_resolve_fun"),
                                };
                                call_ctx.call_function(Rc::clone(next_resolve_fun), None, None, vec![fulfilled_value.clone()]).unwrap();
                            }
                        }
                    } else if state_str == String::from("rejected") {
                        let curren_promise_ref = current_promise.borrow();
                        let rejected_reason = curren_promise_ref.get_inner_property_value(String::from("[[PromiseRejectedReason]]")).unwrap_or(Value::Undefined);
                        // 调用下一个 promise 的 reject 方法
                        if let Some(next_reject) = next_resolve_reject.get(1) {
                            if let Value::Function(next_reject_fun) = next_reject {
                                let mut call_ctx = CallContext {
                                    ctx,
                                    this: Value::Undefined,
                                    reference: Some(Rc::downgrade(next_reject_fun)),
                                    func_name: String::from("next_reject_fun"),
                                };

                                call_ctx.call_function(Rc::clone(next_reject_fun), None, None, vec![rejected_reason.clone()]).unwrap();
                            }
                        }
                    } else {
                        let on_fulfilled = next_resolve_reject.get(0).cloned().unwrap_or(Value::Undefined);
                        let on_rejected = next_resolve_reject.get(1).cloned().unwrap_or(Value::Undefined);

                        let call_ctx = &mut CallContext {
                            ctx,
                            this: Value::Promise(current_promise.clone()),
                            reference: None,
                            func_name: String::from("then"),
                        };

                        let new_promise = then(call_ctx, vec![on_fulfilled, on_rejected]).unwrap();
                        let mut current_promise_ref = current_promise.borrow_mut();
                        current_promise_ref.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), new_promise);

                    }
                }
            } else {
                // 调用下一个 promise 的 resolve 方法
                if let Some(next_resolve) = next_resolve_reject.get(0) {
                    if let Value::Function(next_resolve_fun) = next_resolve {
                        let mut call_ctx = CallContext {
                            ctx,
                            this: Value::Undefined,
                            reference: Some(Rc::downgrade(next_resolve_fun)),
                            func_name: String::from("next_resolve_fun"),
                        };
                        call_ctx.call_function(Rc::clone(next_resolve_fun), None, None, vec![returned_value.clone()]).unwrap();
                    }
                }
            }
        },
        Err(err) => {
            // 调用下一个 promise 的 reject 方法
            if let Some(next_reject) = next_resolve_reject.get(1) {
                if let Value::Function(next_reject_fun) = next_reject {
                    let mut call_ctx = CallContext {
                        ctx,
                        this: Value::Undefined,
                        reference: Some(Rc::downgrade(next_reject_fun)),
                        func_name: String::from("next_reject_fun"),
                    };
                    call_ctx.call_function(Rc::clone(next_reject_fun), None, None, vec![Value::String(err.message)]).unwrap();
                }
            }
        }
    }

}

// Promise.resolve 静态方法
fn resolve_static(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let value = args.get(0).or(Some(&Value::Undefined)).unwrap();

    if let Value::Promise(_) = value {
        return Ok(value.to_owned());
    }

    let promise_value = create_promise(call_ctx.ctx, Value::Undefined);

    if let Value::Promise(promise_rc) = &promise_value {
        let mut promise_mut = promise_rc.borrow_mut();
        promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("fulfilled".to_string()));
        promise_mut.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), value.to_owned());
    }
    Ok(promise_value)
}

// Promise.reject 静态方法
fn reject_static(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let reason = args.get(0).or(Some(&Value::Undefined)).unwrap();

    let promise_value = create_promise(call_ctx.ctx, Value::Undefined);

    if let Value::Promise(promise_rc) = &promise_value {
        let mut promise_mut = promise_rc.borrow_mut();
        promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("rejected".to_string()));
        promise_mut.set_inner_property_value(PROMISE_REJECTED_REASON.to_string(), reason.to_owned());
    }
    Ok(promise_value)
}

fn get_promise_object_from_this<'a>(this_value: &'a Value) -> Option<&'a Rc<RefCell<Object>>> {
    match &this_value {
        Value::Promise(promise_rc) => Some(promise_rc),
        _ => None,
    }
}