use std::vec;
use std::{rc::Rc};
use std::cell::{RefCell};
use super::object::Object;
use super::array::create_array;
use crate::constants::{GLOBAL_PROMISE_NAME, PROTO_PROPERTY_NAME};
use crate::context::{Context};
use crate::error::{JSIError, JSIErrorType};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, error::JSIResult};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Property}, function::builtin_function};

pub const PROMISE_STATE: &str = "[[PromiseState]]";
pub const PROMISE_FULFILLED_VALUE: &str = "[[PromiseFulfilledValue]]";
pub const PROMISE_REJECTED_REASON: &str = "[[PromiseRejectedReason]]";
pub const PROMISE_FULFILL_REACTIONS: &str = "[[PromiseFulfillReactions]]";
pub const PROMISE_REJECT_REACTIONS: &str = "[[PromiseRejectReactions]]";

pub fn create_promise(ctx: &mut Context, init: Value) -> Value {
    let global_promise = get_global_object_by_name(ctx, GLOBAL_PROMISE_NAME);
    let promise = create_object(ctx, ClassType::Promise, None);
    {
        let promise_rc = Rc::clone(&promise);
        let mut promise_mut = promise_rc.borrow_mut();
        promise_mut.constructor = Some(Rc::downgrade(&global_promise));
        promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("pending".to_string()));
        
        // Initialize reaction arrays for pending promises
        let empty_array = create_array(ctx, 0);
        promise_mut.set_inner_property_value(PROMISE_FULFILL_REACTIONS.to_string(), empty_array.clone());
        promise_mut.set_inner_property_value(PROMISE_REJECT_REACTIONS.to_string(), empty_array);

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

  let resolve_name = String::from("resolve");
  global_promise_borrowed.property.insert(resolve_name.clone(), Property { enumerable: true, value: builtin_function(ctx, resolve_name, 1f64, resolve_static) });

    let reject_name = String::from("reject");
    global_promise_borrowed.property.insert(reject_name.clone(), Property { enumerable: true, value: builtin_function(ctx, reject_name, 1f64, reject_static) });

    let all_name = String::from("all");
    global_promise_borrowed.property.insert(all_name.clone(), Property { enumerable: true, value: builtin_function(ctx, all_name, 1f64, promise_all) });

    let race_name = String::from("race");
    global_promise_borrowed.property.insert(race_name.clone(), Property { enumerable: true, value: builtin_function(ctx, race_name, 1f64, promise_race) });

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


// Helper function to execute promise reactions
fn execute_promise_reactions(ctx: &mut Context, reactions: Value, value: Value, is_fulfilled: bool) {
    if let Value::Array(reactions_array) = reactions {
        let reactions_borrowed = reactions_array.borrow();
        if let Some(length_prop) = reactions_borrowed.get_inner_property_value("length".to_string()) {
            if let Value::Number(length) = length_prop {
                for i in 0..(length as usize) {
                    if let Some(reaction_prop) = reactions_borrowed.get_inner_property_value(i.to_string()) {
                        if let Value::Object(reaction_obj) = reaction_prop {
                            let reaction_borrowed = reaction_obj.borrow();
                            if let Some(handler) = reaction_borrowed.get_inner_property_value("handler".to_string()) {
                                if let Some(capability) = reaction_borrowed.get_inner_property_value("capability".to_string()) {
                                    // Execute the handler and resolve/reject the capability promise
                                    execute_promise_reaction(ctx, handler, value.clone(), capability, is_fulfilled);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Execute a single promise reaction
fn execute_promise_reaction(ctx: &mut Context, handler: Value, value: Value, capability: Value, is_fulfilled: bool) {
    if let Value::Object(capability_obj) = capability {
        let capability_borrowed = capability_obj.borrow();
        if let Some(resolve_fn) = capability_borrowed.get_inner_property_value("resolve".to_string()) {
            if let Some(reject_fn) = capability_borrowed.get_inner_property_value("reject".to_string()) {
                
                let result = if matches!(handler, Value::Undefined) {
                    // If no handler, just pass through the value
                    if is_fulfilled {
                        Ok(value.clone())
                    } else {
                        Err(JSIError::new(JSIErrorType::Unknown, "Promise rejection".to_string(), 0, 0))
                    }
                } else if let Value::Function(handler_fn) = handler {
                    // Execute the handler function
                    let mut call_ctx = CallContext {
                        ctx,
                        this: Value::Undefined,
                        reference: None,
                        func_name: String::from("then_handler"),
                    };
                    call_ctx.call_function(handler_fn, None, None, vec![value.clone()])
                } else {
                    Ok(value.clone())
                };

                // Handle the result based on whether it's a promise or a regular value
                match result {
                    Ok(resolved_value) => {
                        // Check if the resolved value is a promise
                        if let Value::Promise(returned_promise_rc) = resolved_value {
                            // If a promise is returned, we need to "unwrap" it
                            let returned_promise = returned_promise_rc.borrow();
                            let returned_state = returned_promise.get_inner_property_value(PROMISE_STATE.to_string()).unwrap();
                            
                            if let Value::String(state_str) = returned_state {
                                if state_str == "fulfilled" {
                                    let fulfilled_value = returned_promise.get_inner_property_value(PROMISE_FULFILLED_VALUE.to_string()).unwrap_or(Value::Undefined);
                                    drop(returned_promise); // Release borrow
                                    
                                    if let Value::Function(resolve_function) = resolve_fn {
                                        let mut call_ctx = CallContext {
                                            ctx,
                                            this: Value::Undefined,
                                            reference: Some(Rc::downgrade(&resolve_function)),
                                            func_name: String::from("resolve"),
                                        };
                                        let _ = resolve(&mut call_ctx, vec![fulfilled_value]);
                                    }
                                } else if state_str == "rejected" {
                                    let rejected_reason = returned_promise.get_inner_property_value(PROMISE_REJECTED_REASON.to_string()).unwrap_or(Value::Undefined);
                                    drop(returned_promise); // Release borrow
                                    
                                    if let Value::Function(reject_function) = reject_fn {
                                        let mut call_ctx = CallContext {
                                            ctx,
                                            this: Value::Undefined,
                                            reference: Some(Rc::downgrade(&reject_function)),
                                            func_name: String::from("reject"),
                                        };
                                        let _ = reject(&mut call_ctx, vec![rejected_reason]);
                                    }
                                } else {
                                    // Pending promise - would need to add reaction to wait for it
                                    // For now, just resolve with undefined
                                    if let Value::Function(resolve_function) = resolve_fn {
                                        let mut call_ctx = CallContext {
                                            ctx,
                                            this: Value::Undefined,
                                            reference: Some(Rc::downgrade(&resolve_function)),
                                            func_name: String::from("resolve"),
                                        };
                                        let _ = resolve(&mut call_ctx, vec![Value::Undefined]);
                                    }
                                }
                            }
                        } else {
                            // Regular value, just resolve with it
                            if let Value::Function(resolve_function) = resolve_fn {
                                let mut call_ctx = CallContext {
                                    ctx,
                                    this: Value::Undefined,
                                    reference: Some(Rc::downgrade(&resolve_function)),
                                    func_name: String::from("resolve"),
                                };
                                let _ = resolve(&mut call_ctx, vec![resolved_value]);
                            }
                        }
                    },
                    Err(_) => {
                        // Handler threw an error, reject the capability promise
                        if let Value::Function(reject_function) = reject_fn {
                            let mut call_ctx = CallContext {
                                ctx,
                                this: Value::Undefined,
                                reference: Some(Rc::downgrade(&reject_function)),
                                func_name: String::from("reject"),
                            };
                            let _ = reject(&mut call_ctx, vec![value.clone()]);
                        }
                    }
                }
            }
        }
    }
}

// resolve 方法
fn resolve(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {

    let resolve_fn = call_ctx.reference.as_ref().and_then(|r| r.upgrade()).expect("resolve rc error");
    
    let promise = resolve_fn.borrow().get_inner_property_value(String::from("promise")).unwrap();

    if let Value::RefObject(promise_rc_weak) = promise {
        if let Some(promise_rc) = promise_rc_weak.upgrade() {
            let mut promise_mut = promise_rc.borrow_mut();
            
            // Check if promise is already settled
            let state = promise_mut.get_inner_property_value(PROMISE_STATE.to_string()).unwrap();
            if let Value::String(state_str) = state {
                if state_str != "pending" {
                    return Ok(Value::Undefined);
                }
            }
            
            // 设置 Promise 的状态为 fulfilled
            promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("fulfilled".to_string()));
            // 处理 resolve 的值
            let value = args.get(0).cloned().unwrap_or(Value::Undefined);
            promise_mut.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), value.clone());
            
            // Execute fulfill reactions
            let fulfill_reactions = promise_mut.get_inner_property_value(PROMISE_FULFILL_REACTIONS.to_string()).unwrap_or(Value::Undefined);
            drop(promise_mut); // Release the borrow before calling execute_promise_reactions
            
            execute_promise_reactions(call_ctx.ctx, fulfill_reactions, value, true);
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
            
            // Check if promise is already settled
            let state = promise_mut.get_inner_property_value(PROMISE_STATE.to_string()).unwrap();
            if let Value::String(state_str) = state {
                if state_str != "pending" {
                    return Ok(Value::Undefined);
                }
            }
            
            // 设置 Promise 的状态为 rejected
            promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("rejected".to_string()));
            // 处理 reject 的原因
            let reason = args.get(0).cloned().unwrap_or(Value::Undefined);
            promise_mut.set_inner_property_value(PROMISE_REJECTED_REASON.to_string(), reason.clone());
            
            // Execute reject reactions
            let reject_reactions = promise_mut.get_inner_property_value(PROMISE_REJECT_REACTIONS.to_string()).unwrap_or(Value::Undefined);
            drop(promise_mut); // Release the borrow before calling execute_promise_reactions
            
            execute_promise_reactions(call_ctx.ctx, reject_reactions, reason, false);
        }
    }

   Ok(Value::Undefined)
}

fn then(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let on_fulfilled = args.get(0).cloned().unwrap_or(Value::Undefined);
    let on_rejected = args.get(1).cloned().unwrap_or(Value::Undefined);

    // Create a new promise to return
    let result_promise = create_promise(call_ctx.ctx, Value::Undefined);
    
    // Get resolve and reject functions for the new promise
    let (result_resolve_fn, result_reject_fn) = {
        if let Value::Promise(result_promise_rc) = &result_promise {
            let resolve_fn = builtin_function(call_ctx.ctx, "resolve".to_string(), 1f64, resolve);
            let reject_fn = builtin_function(call_ctx.ctx, "reject".to_string(), 1f64, reject);
            
            if let Value::Function(resolve_function) = &resolve_fn {
                let mut resolve_function_mut = resolve_function.borrow_mut();
                resolve_function_mut.set_inner_property_value(String::from("promise"), Value::RefObject(Rc::downgrade(&result_promise_rc)));
            }
            
            if let Value::Function(reject_function) = &reject_fn {
                let mut reject_function_mut = reject_function.borrow_mut();
                reject_function_mut.set_inner_property_value(String::from("promise"), Value::RefObject(Rc::downgrade(&result_promise_rc)));
            }
            
            (resolve_fn, reject_fn)
        } else {
            return Err(JSIError::new(JSIErrorType::TypeError, "Failed to create result promise".to_string(), 0, 0));
        }
    };

    // Create promise capability object
    let capability = create_object(call_ctx.ctx, ClassType::Object, None);
    {
        let mut capability_mut = capability.borrow_mut();
        capability_mut.set_inner_property_value("resolve".to_string(), result_resolve_fn);
        capability_mut.set_inner_property_value("reject".to_string(), result_reject_fn);
    }

    // Get the current promise state
    let this_promise_obj = get_promise_object_from_this(&call_ctx.this).unwrap();
    let this_promise = this_promise_obj.borrow();
    let state = this_promise.get_inner_property_value(PROMISE_STATE.to_string()).unwrap();

    if let Value::String(state_str) = state {
        if state_str == "fulfilled" {
            // Promise is already fulfilled
            let fulfilled_value = this_promise.get_inner_property_value(PROMISE_FULFILLED_VALUE.to_string()).unwrap_or(Value::Undefined);
            drop(this_promise); // Release the borrow
            
            // Execute onFulfilled callback if provided
            if !matches!(on_fulfilled, Value::Undefined) {
                execute_promise_reaction(call_ctx.ctx, on_fulfilled, fulfilled_value, Value::Object(capability), true);
            } else {
                // No handler, just pass through the value
                execute_promise_reaction(call_ctx.ctx, Value::Undefined, fulfilled_value, Value::Object(capability), true);
            }
        } else if state_str == "rejected" {
            // Promise is already rejected
            let rejected_reason = this_promise.get_inner_property_value(PROMISE_REJECTED_REASON.to_string()).unwrap_or(Value::Undefined);
            drop(this_promise); // Release the borrow
            
            // Execute onRejected callback if provided
            if !matches!(on_rejected, Value::Undefined) {
                execute_promise_reaction(call_ctx.ctx, on_rejected, rejected_reason, Value::Object(capability), false);
            } else {
                // No handler, just pass through the rejection
                execute_promise_reaction(call_ctx.ctx, Value::Undefined, rejected_reason, Value::Object(capability), false);
            }
        } else {
            // Promise is pending - store the reactions for later execution
            
            // Need to reborrow as mutable
            drop(this_promise);
            let this_promise = this_promise_obj.borrow_mut();
            
            // Create fulfill reaction
            let fulfill_reaction = create_object(call_ctx.ctx, ClassType::Object, None);
            {
                let mut fulfill_reaction_mut = fulfill_reaction.borrow_mut();
                fulfill_reaction_mut.set_inner_property_value("handler".to_string(), on_fulfilled);
                fulfill_reaction_mut.set_inner_property_value("capability".to_string(), Value::Object(Rc::clone(&capability)));
            }
            
            // Create reject reaction  
            let reject_reaction = create_object(call_ctx.ctx, ClassType::Object, None);
            {
                let mut reject_reaction_mut = reject_reaction.borrow_mut();
                reject_reaction_mut.set_inner_property_value("handler".to_string(), on_rejected);
                reject_reaction_mut.set_inner_property_value("capability".to_string(), Value::Object(capability));
            }
            
            // Add reactions to the promise's reaction arrays
            let fulfill_reactions = this_promise.get_inner_property_value(PROMISE_FULFILL_REACTIONS.to_string()).unwrap();
            let reject_reactions = this_promise.get_inner_property_value(PROMISE_REJECT_REACTIONS.to_string()).unwrap();
            
            if let Value::Array(fulfill_array) = fulfill_reactions {
                let mut fulfill_array_mut = fulfill_array.borrow_mut();
                let length = fulfill_array_mut.get_inner_property_value("length".to_string()).unwrap_or(Value::Number(0f64));
                if let Value::Number(len) = length {
                    fulfill_array_mut.set_inner_property_value(len.to_string(), Value::Object(fulfill_reaction));
                    fulfill_array_mut.set_inner_property_value("length".to_string(), Value::Number(len + 1f64));
                }
            }
            
            if let Value::Array(reject_array) = reject_reactions {
                let mut reject_array_mut = reject_array.borrow_mut();
                let length = reject_array_mut.get_inner_property_value("length".to_string()).unwrap_or(Value::Number(0f64));
                if let Value::Number(len) = length {
                    reject_array_mut.set_inner_property_value(len.to_string(), Value::Object(reject_reaction));
                    reject_array_mut.set_inner_property_value("length".to_string(), Value::Number(len + 1f64));
                }
            }
        }
    }

    Ok(result_promise)
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

// Promise.all 静态方法
fn promise_all(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let promises_array = args.get(0).cloned().unwrap_or(Value::Undefined);
    
    // Check if argument is an array-like object
    if !matches!(promises_array, Value::Array(_)) {
        return Err(JSIError::new(JSIErrorType::TypeError, "Promise.all requires an array".to_string(), 0, 0));
    }
    
    let result_promise = create_promise(call_ctx.ctx, Value::Undefined);
    
    if let Value::Array(promises_rc) = promises_array {
        let promises_borrowed = promises_rc.borrow();
        let length_prop = promises_borrowed.get_value("length".to_string());
        
        if let Value::Number(length) = length_prop {
            let length = length as usize;
            
            if length == 0 {
                // Empty array - resolve immediately with empty array
                if let Value::Promise(result_promise_rc) = &result_promise {
                    let mut result_promise_mut = result_promise_rc.borrow_mut();
                    result_promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("fulfilled".to_string()));
                    let empty_result = create_array(call_ctx.ctx, 0);
                    result_promise_mut.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), empty_result);
                }
                return Ok(result_promise);
            }
            
            // Create result array and track completion
            let result_array = create_array(call_ctx.ctx, length);
            let mut all_fulfilled = true;
            
            // Check each promise and collect results
            for i in 0..length {
                let promise_value = promises_borrowed.get_value(i.to_string());
                
                if !matches!(promise_value, Value::Undefined) {
                    match promise_value {
                        Value::Promise(promise_rc) => {
                            let promise_borrowed = promise_rc.borrow();
                            let state = promise_borrowed.get_inner_property_value(PROMISE_STATE.to_string()).unwrap_or(Value::String("pending".to_string()));
                            
                            if let Value::String(state_str) = state {
                                match state_str.as_str() {
                                    "fulfilled" => {
                                        let fulfilled_value = promise_borrowed.get_inner_property_value(PROMISE_FULFILLED_VALUE.to_string()).unwrap_or(Value::Undefined);
                                        if let Value::Array(result_rc) = &result_array {
                                            let mut result_mut = result_rc.borrow_mut();
                                            result_mut.define_property(i.to_string(), Property { enumerable: true, value: fulfilled_value });
                                        }
                                    },
                                    "rejected" => {
                                        // Any rejected promise causes Promise.all to reject immediately
                                        let rejected_reason = promise_borrowed.get_inner_property_value(PROMISE_REJECTED_REASON.to_string()).unwrap_or(Value::Undefined);
                                        if let Value::Promise(result_promise_rc) = &result_promise {
                                            let mut result_promise_mut = result_promise_rc.borrow_mut();
                                            result_promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("rejected".to_string()));
                                            result_promise_mut.set_inner_property_value(PROMISE_REJECTED_REASON.to_string(), rejected_reason);
                                        }
                                        return Ok(result_promise);
                                    },
                                    "pending" => {
                                        all_fulfilled = false;
                                        // TODO: For full implementation, we'd need to add reactions to wait for pending promises
                                        // For now, just treat as undefined
                                        if let Value::Array(result_rc) = &result_array {
                                            let mut result_mut = result_rc.borrow_mut();
                                            result_mut.define_property(i.to_string(), Property { enumerable: true, value: Value::Undefined });
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        },
                        _ => {
                            // Non-promise values are treated as fulfilled promises
                            if let Value::Array(result_rc) = &result_array {
                                let mut result_mut = result_rc.borrow_mut();
                                result_mut.define_property(i.to_string(), Property { enumerable: true, value: promise_value });
                            }
                        }
                    }
                }
            }
            
            // If all promises are fulfilled, resolve with the result array
            if all_fulfilled {
                if let Value::Promise(result_promise_rc) = &result_promise {
                    let mut result_promise_mut = result_promise_rc.borrow_mut();
                    result_promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("fulfilled".to_string()));
                    result_promise_mut.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), result_array);
                }
            } else {
                // For simplicity, if there are pending promises, resolve with current state
                // In a full implementation, we'd wait for all to complete
                if let Value::Promise(result_promise_rc) = &result_promise {
                    let mut result_promise_mut = result_promise_rc.borrow_mut();
                    result_promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("fulfilled".to_string()));
                    result_promise_mut.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), result_array);
                }
            }
        }
    }
    
    Ok(result_promise)
}

// Promise.race 静态方法
fn promise_race(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let promises_array = args.get(0).cloned().unwrap_or(Value::Undefined);
    
    // Check if argument is an array-like object
    if !matches!(promises_array, Value::Array(_)) {
        return Err(JSIError::new(JSIErrorType::TypeError, "Promise.race requires an array".to_string(), 0, 0));
    }
    
    let result_promise = create_promise(call_ctx.ctx, Value::Undefined);
    
    if let Value::Array(promises_rc) = promises_array {
        let promises_borrowed = promises_rc.borrow();
        let length_prop = promises_borrowed.get_value("length".to_string());
        
        if let Value::Number(length) = length_prop {
            let length = length as usize;
            
            if length == 0 {
                // Empty array - promise remains pending forever
                return Ok(result_promise);
            }
            
            // Find the first settled promise
            for i in 0..length {
                let promise_value = promises_borrowed.get_value(i.to_string());
                
                if !matches!(promise_value, Value::Undefined) {
                    match promise_value {
                        Value::Promise(promise_rc) => {
                            let promise_borrowed = promise_rc.borrow();
                            let state = promise_borrowed.get_inner_property_value(PROMISE_STATE.to_string()).unwrap_or(Value::String("pending".to_string()));
                            
                            if let Value::String(state_str) = state {
                                match state_str.as_str() {
                                    "fulfilled" => {
                                        let fulfilled_value = promise_borrowed.get_inner_property_value(PROMISE_FULFILLED_VALUE.to_string()).unwrap_or(Value::Undefined);
                                        if let Value::Promise(result_promise_rc) = &result_promise {
                                            let mut result_promise_mut = result_promise_rc.borrow_mut();
                                            result_promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("fulfilled".to_string()));
                                            result_promise_mut.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), fulfilled_value);
                                        }
                                        return Ok(result_promise);
                                    },
                                    "rejected" => {
                                        let rejected_reason = promise_borrowed.get_inner_property_value(PROMISE_REJECTED_REASON.to_string()).unwrap_or(Value::Undefined);
                                        if let Value::Promise(result_promise_rc) = &result_promise {
                                            let mut result_promise_mut = result_promise_rc.borrow_mut();
                                            result_promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("rejected".to_string()));
                                            result_promise_mut.set_inner_property_value(PROMISE_REJECTED_REASON.to_string(), rejected_reason);
                                        }
                                        return Ok(result_promise);
                                    },
                                    "pending" => {
                                        // Continue checking other promises
                                    },
                                    _ => {}
                                }
                            }
                        },
                        _ => {
                            // Non-promise values are treated as fulfilled promises
                            if let Value::Promise(result_promise_rc) = &result_promise {
                                let mut result_promise_mut = result_promise_rc.borrow_mut();
                                result_promise_mut.set_inner_property_value(PROMISE_STATE.to_string(), Value::String("fulfilled".to_string()));
                                result_promise_mut.set_inner_property_value(PROMISE_FULFILLED_VALUE.to_string(), promise_value);
                            }
                            return Ok(result_promise);
                        }
                    }
                }
            }
            
            // If we get here, all promises are pending
            // For simplicity, leave the result promise pending
            // In a full implementation, we'd add reactions to all input promises
        }
    }
    
    Ok(result_promise)
}

fn get_promise_object_from_this<'a>(this_value: &'a Value) -> Option<&'a Rc<RefCell<Object>>> {
    match &this_value {
        Value::Promise(promise_rc) => Some(promise_rc),
        _ => None,
    }
}