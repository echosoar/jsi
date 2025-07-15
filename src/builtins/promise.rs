use std::cell::RefCell;
use std::rc::Rc;

use crate::ast_node::ClassType;
use crate::constants::GLOBAL_PROMISE_NAME;
use crate::value::{Value, INSTANTIATE_OBJECT_METHOD_NAME};
use crate::context::Context;
use crate::error::{JSIResult, JSIError, JSIErrorType};
use super::function::builtin_function;
use super::global::{get_global_object, get_global_object_prototype_by_name};
use super::object::{Object, Property, create_object};

// Promise states
#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState {
    Pending,
    Fulfilled(Value),
    Rejected(Value),
}

pub fn create_promise(ctx: &mut Context, state: PromiseState) -> Value {
    let global_promise = get_global_object(ctx, GLOBAL_PROMISE_NAME.to_string());
    let promise = create_object(ctx, ClassType::Promise, None);
    
    // Set up promise object
    {
        let promise_rc = Rc::clone(&promise);
        let mut promise_mut = promise_rc.borrow_mut();
        
        // Bind constructor to global Promise
        promise_mut.constructor = Some(Rc::downgrade(&global_promise));
        
        // Set promise state as internal property
        match state {
            PromiseState::Pending => {
                promise_mut.set_inner_property_value("[[PromiseState]]".to_string(), Value::String("pending".to_string()));
                promise_mut.set_inner_property_value("[[PromiseValue]]".to_string(), Value::Undefined);
            },
            PromiseState::Fulfilled(value) => {
                promise_mut.set_inner_property_value("[[PromiseState]]".to_string(), Value::String("fulfilled".to_string()));
                promise_mut.set_inner_property_value("[[PromiseValue]]".to_string(), value);
            },
            PromiseState::Rejected(value) => {
                promise_mut.set_inner_property_value("[[PromiseState]]".to_string(), Value::String("rejected".to_string()));
                promise_mut.set_inner_property_value("[[PromiseValue]]".to_string(), value);
            },
        }
        
        // Initialize callback arrays
        promise_mut.set_inner_property_value("[[PromiseFulfillReactions]]".to_string(), Value::Array(Rc::new(RefCell::new(Object::new(ClassType::Array, None)))));
        promise_mut.set_inner_property_value("[[PromiseRejectReactions]]".to_string(), Value::Array(Rc::new(RefCell::new(Object::new(ClassType::Array, None)))));
    }
    
    Value::Promise(promise)
}

pub fn bind_global_promise(ctx: &mut Context) {
    // Get references first
    let global_promise = get_global_object(ctx, GLOBAL_PROMISE_NAME.to_string());
    let promise_prototype = get_global_object_prototype_by_name(ctx, GLOBAL_PROMISE_NAME);
    
    // Bind global Promise static methods
    {
        let global_promise_clone = Rc::clone(&global_promise);
        let mut global_promise_borrowed = global_promise_clone.borrow_mut();
        
        // Promise constructor instantiate method
        let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 1f64, create);
        global_promise_borrowed.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);
        
        // Promise constructor
        global_promise_borrowed.define_property(String::from("length"), Property {
            enumerable: false,
            value: Value::Number(1f64),
        });
        
        // Promise.resolve
        let promise_resolve = builtin_function(ctx, String::from("resolve"), 1f64, promise_resolve_call);
        global_promise_borrowed.define_property(String::from("resolve"), Property {
            enumerable: false,
            value: promise_resolve,
        });
        
        // Promise.reject
        let promise_reject = builtin_function(ctx, String::from("reject"), 1f64, promise_reject_call);
        global_promise_borrowed.define_property(String::from("reject"), Property {
            enumerable: false,
            value: promise_reject,
        });
        
        // Promise.all
        let promise_all = builtin_function(ctx, String::from("all"), 1f64, promise_all_call);
        global_promise_borrowed.define_property(String::from("all"), Property {
            enumerable: false,
            value: promise_all,
        });
        
        // Promise.race
        let promise_race = builtin_function(ctx, String::from("race"), 1f64, promise_race_call);
        global_promise_borrowed.define_property(String::from("race"), Property {
            enumerable: false,
            value: promise_race,
        });
    }
    
    // Bind prototype methods
    {
        let promise_prototype_clone = Rc::clone(&promise_prototype);
        let mut promise_prototype_borrowed = promise_prototype_clone.borrow_mut();
        
        // Promise.prototype.then
        let promise_then = builtin_function(ctx, String::from("then"), 2f64, promise_then_call);
        promise_prototype_borrowed.define_property(String::from("then"), Property {
            enumerable: false,
            value: promise_then,
        });
        
        // Promise.prototype.catch
        let promise_catch = builtin_function(ctx, String::from("catch"), 1f64, promise_catch_call);
        promise_prototype_borrowed.define_property(String::from("catch"), Property {
            enumerable: false,
            value: promise_catch,
        });
        
        // Promise.prototype.finally
        let promise_finally = builtin_function(ctx, String::from("finally"), 1f64, promise_finally_call);
        promise_prototype_borrowed.define_property(String::from("finally"), Property {
            enumerable: false,
            value: promise_finally,
        });
    }
}

// Promise.resolve implementation
fn promise_resolve_call(call_ctx: &mut crate::ast_node::CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let value = if args.len() > 0 {
        args[0].clone()
    } else {
        Value::Undefined
    };
    
    Ok(create_promise(call_ctx.ctx, PromiseState::Fulfilled(value)))
}

// Promise.reject implementation
fn promise_reject_call(call_ctx: &mut crate::ast_node::CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let value = if args.len() > 0 {
        args[0].clone()
    } else {
        Value::Undefined
    };
    
    Ok(create_promise(call_ctx.ctx, PromiseState::Rejected(value)))
}

// Promise.all implementation (simplified)
fn promise_all_call(call_ctx: &mut crate::ast_node::CallContext, args: Vec<Value>) -> JSIResult<Value> {
    if args.len() == 0 {
        return Err(JSIError::new(JSIErrorType::TypeError, "Promise.all requires an iterable".to_string(), 0, 0));
    }
    
    // For now, just return a resolved promise with empty array
    // Full implementation would require iterating over the promises
    Ok(create_promise(call_ctx.ctx, PromiseState::Fulfilled(Value::Array(Rc::new(RefCell::new(Object::new(ClassType::Array, None)))))))
}

// Promise.race implementation (simplified)  
fn promise_race_call(call_ctx: &mut crate::ast_node::CallContext, args: Vec<Value>) -> JSIResult<Value> {
    if args.len() == 0 {
        return Err(JSIError::new(JSIErrorType::TypeError, "Promise.race requires an iterable".to_string(), 0, 0));
    }
    
    // For now, just return a pending promise
    // Full implementation would require racing the promises
    Ok(create_promise(call_ctx.ctx, PromiseState::Pending))
}

// Promise.prototype.then implementation
fn promise_then_call(call_ctx: &mut crate::ast_node::CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let promise_value = call_ctx.this.clone();
    
    match promise_value {
        Value::Promise(promise_obj) => {
            let promise_obj_clone = Rc::clone(&promise_obj);
            let promise_borrowed = promise_obj_clone.borrow();
            
            // Get promise state
            let state = promise_borrowed.get_inner_property_value("[[PromiseState]]".to_string());
            let value = promise_borrowed.get_inner_property_value("[[PromiseValue]]".to_string());
            
            if let (Some(Value::String(state_str)), Some(promise_value)) = (state, value) {
                match state_str.as_str() {
                    "fulfilled" => {
                        // If we have an onFulfilled callback, call it
                        if args.len() > 0 {
                            if let Value::Function(_callback) = &args[0] {
                                // For now, just return the promise value directly
                                // Full implementation would call the callback
                                return Ok(create_promise(call_ctx.ctx, PromiseState::Fulfilled(promise_value)));
                            }
                        }
                        Ok(create_promise(call_ctx.ctx, PromiseState::Fulfilled(promise_value)))
                    },
                    "rejected" => {
                        // If we have an onRejected callback, call it
                        if args.len() > 1 {
                            if let Value::Function(_callback) = &args[1] {
                                // For now, just return a fulfilled promise
                                // Full implementation would call the callback
                                return Ok(create_promise(call_ctx.ctx, PromiseState::Fulfilled(Value::Undefined)));
                            }
                        }
                        Ok(create_promise(call_ctx.ctx, PromiseState::Rejected(promise_value)))
                    },
                    _ => {
                        // Pending - return a new pending promise
                        Ok(create_promise(call_ctx.ctx, PromiseState::Pending))
                    }
                }
            } else {
                Err(JSIError::new(JSIErrorType::TypeError, "Invalid promise state".to_string(), 0, 0))
            }
        },
        _ => {
            Err(JSIError::new(JSIErrorType::TypeError, "then called on non-promise".to_string(), 0, 0))
        }
    }
}

// Promise.prototype.catch implementation
fn promise_catch_call(call_ctx: &mut crate::ast_node::CallContext, args: Vec<Value>) -> JSIResult<Value> {
    // catch(onRejected) is equivalent to then(undefined, onRejected)
    let mut new_args = vec![Value::Undefined];
    if args.len() > 0 {
        new_args.push(args[0].clone());
    }
    
    promise_then_call(call_ctx, new_args)
}

// Promise.prototype.finally implementation
fn promise_finally_call(call_ctx: &mut crate::ast_node::CallContext, _args: Vec<Value>) -> JSIResult<Value> {
    let promise_value = call_ctx.this.clone();
    
    match promise_value {
        Value::Promise(promise_obj) => {
            let promise_obj_clone = Rc::clone(&promise_obj);
            let promise_borrowed = promise_obj_clone.borrow();
            
            // Get promise state and value
            let state = promise_borrowed.get_inner_property_value("[[PromiseState]]".to_string());
            let value = promise_borrowed.get_inner_property_value("[[PromiseValue]]".to_string());
            
            if let (Some(Value::String(state_str)), Some(promise_value)) = (state, value) {
                // For now, just return the original promise
                // Full implementation would call the finally callback
                match state_str.as_str() {
                    "fulfilled" => Ok(create_promise(call_ctx.ctx, PromiseState::Fulfilled(promise_value))),
                    "rejected" => Ok(create_promise(call_ctx.ctx, PromiseState::Rejected(promise_value))),
                    _ => Ok(create_promise(call_ctx.ctx, PromiseState::Pending)),
                }
            } else {
                Err(JSIError::new(JSIErrorType::TypeError, "Invalid promise state".to_string(), 0, 0))
            }
        },
        _ => {
            Err(JSIError::new(JSIErrorType::TypeError, "finally called on non-promise".to_string(), 0, 0))
        }
    }
}

// Promise constructor implementation
fn create(call_ctx: &mut crate::ast_node::CallContext, args: Vec<Value>) -> JSIResult<Value> {
    if args.len() == 0 {
        return Err(JSIError::new(JSIErrorType::TypeError, "Promise resolver is not a function".to_string(), 0, 0));
    }
    
    let executor = &args[0];
    if !matches!(executor, Value::Function(_)) {
        return Err(JSIError::new(JSIErrorType::TypeError, "Promise resolver is not a function".to_string(), 0, 0));
    }
    
    // Create a pending promise
    let promise = create_promise(call_ctx.ctx, PromiseState::Pending);
    
    // For now, we'll implement a simplified version that doesn't actually execute the function
    // In a full implementation, we would call the executor function with resolve/reject callbacks
    
    Ok(promise)
}