use std::{cell::RefCell, collections::HashMap, rc::{Rc, Weak}};

use crate::{ast::Program, ast_node::{ArrayLiteral, AssignExpression, BinaryExpression, CallContext, CallExpression, ClassType, Declaration, Expression, ForStatement, IdentifierLiteral, Keywords, NewExpression, ObjectLiteral, PostfixUnaryExpression, PrefixUnaryExpression, Statement, SwitchStatement, VariableFlag}, ast_token::Token, builtins::{array::create_array, console::create_console, function::{create_function, create_function_with_bytecode, get_builtin_function_name, get_function_this}, global::{bind_global, get_global_object, new_global_this, IS_GLOABL_OBJECT}, object::{create_object, Object, Property}}, bytecode::{self, ByteCode, EByteCodeop}, constants::{GLOBAL_OBJECT_NAME_LIST, PROTO_PROPERTY_NAME}, error::{JSIError, JSIErrorType, JSIResult}, scope::{get_value_and_scope, get_value_info_and_scope, Scope}, value::{CallStatementOptions, Value, ValueInfo}};


use super::ast::AST;
pub struct Context {
  pub global: Rc<RefCell<Object>>,
  pub strict: bool,
  scope: Rc<RefCell<Scope>>,
  pub cur_scope: Rc<RefCell<Scope>>,
  // 调用栈
  stack: Vec<ValueInfo>,
  // label index map
  label_index_map: HashMap<String, usize>,
}

impl Context {
    pub fn new() -> Context {
      let scope = Rc::new(RefCell::new(Scope::new()));
      let cur_scope = Rc::clone(&scope);
      let global = new_global_this();
      let mut ctx = Context {
        global,
        strict: true,
        scope,
        cur_scope,
        stack: vec![],
        label_index_map: HashMap::new(),
      };
      bind_global(&mut ctx);
      ctx.init();
      return ctx;
    }

    pub fn set_strict(&mut self,strict: bool) {
      self.strict = strict;
    }
    
    // 运行一段 JS 代码
    pub fn run(&mut self, code: String) -> JSIResult<Value> {
      let program = self.parse(code)?;
      // print!("program: {:?}", program);
      self.call(program)
    }

    pub fn run_with_bytecode(&mut self, code: String) -> JSIResult<Value> {
       let program = self.parse(code)?;
        let bytecode = program.bytecode;

        println!("AST bytecode: {:?}", bytecode);

        self.run_with_bytecode_list(0, &bytecode)?;

        if self.stack.is_empty() {
          return Ok(Value::Undefined);
        }
        // 返回栈顶的值
        let result = self.stack.pop().unwrap();
        Ok(result.value)
    }

    pub fn run_with_bytecode_list(&mut self, cur_index: usize, bytecode: &Vec<ByteCode>) -> JSIResult<Value> {
     
      let mut bytecode_index = cur_index;
      // for bytecode_item in bytecode.iter() {
      while bytecode_index < bytecode.len() {
        let cur_index = bytecode_index;
        let bytecode_item = &bytecode[cur_index];
        bytecode_index += 1;
        println!("");
        // println!(">>>> cur_stack {:?} {:?} {:?}", self.stack.len(), self.stack.iter().map(|item| item.name.clone().unwrap_or(String::from(""))).collect::<Vec<String>>(), bytecode_item.op);
        println!(">>>> cur_stack {:?} {:?}", self.stack, bytecode_item.op);
        match bytecode_item.op {
          EByteCodeop::OpUndefined => {
            self.stack.push(Value::Undefined.to_value_info());
          },
          EByteCodeop::OpNull => {
            self.stack.push(Value::Null.to_value_info());
          },
          EByteCodeop::OpTrue => {
            self.stack.push(Value::Boolean(true).to_value_info());
          },
          EByteCodeop::OpFalse => {
            self.stack.push(Value::Boolean(false).to_value_info());
          },
          EByteCodeop::OpString => {
            if let Some(arg) = bytecode_item.args.get(0) {
              self.stack.push(Value::String(arg.clone()).to_value_info());
            } else {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("string arg is required"), 0, 0));
            }
          },
          EByteCodeop::OpNumber => {
            if let Some(arg) = bytecode_item.args.get(0) {
              if let Ok(num) = arg.parse::<f64>() {
                self.stack.push(Value::Number(num).to_value_info());
              } else {
                return Err(JSIError::new(JSIErrorType::SyntaxError, format!("number arg is invalid: {}", arg), 0, 0));
              }
            } else {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("number arg is required"), 0, 0));
            }
          },
          EByteCodeop::OpArray => {
            let length = if let Some(length) = bytecode_item.args.get(0) {
              length.parse::<usize>().unwrap_or(0)
            } else {
              0
            };
            let array = create_array(self, 0);
            if let Value::Array(arr_obj) = &array {
              if length > self.stack.len() {
                return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("array length is greater than stack length"), 0, 0));
              }
              let arguments = self.stack.drain(self.stack.len() - length..).map(|item| item.value).collect();
              
              let call_ctx = &mut CallContext {
                ctx: self,
                this: Value::Array(Rc::clone(arr_obj)),
                reference: None,
                func_name: String::from("push")
              };
              Object::call(call_ctx, String::from("push"), arguments)?;
            }
            self.stack.push(array.to_value_info());
          },
          EByteCodeop::OpScopePutVarInit | EByteCodeop::OpScopePutVar => {
            if let Some(arg) = bytecode_item.args.get(0) {
              // 获取栈顶的值
              let value = self.stack.pop().unwrap();
              let name = arg.clone();
              // 设置到当前作用域
              // TODO: const
              (*self.cur_scope).borrow_mut().set_value(name, value.value, false);
            } else {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("variable name arg is required"), 0, 0));
            }
          },
          EByteCodeop::OpScopeGetVar => {
            if let Some(arg) = bytecode_item.args.get(0) {
              // 获取当前作用域的值
              let name = arg.to_string();
              let (value, _, _) = get_value_info_and_scope(Rc::clone(&self.cur_scope), name.clone());
              if let Some(val) = value {
                self.stack.push(val);
              } else {
                return Err(JSIError::new(JSIErrorType::ReferenceError, format!("{} is not defined", name), 0, 0));
              }
            } else {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("variable name arg is required"), 0, 0));
            }
          },
          EByteCodeop::OpGetProperty => {
            // 从栈中弹出一个值作为对象
            let object = self.stack.pop().unwrap();
            if let Some(property_name) = bytecode_item.args.get(0) {
              // 获取属性名
              let left = object.value;
              if left.is_equal_to(self, &Value::Null, true) {
                return Err(JSIError::new( JSIErrorType::TypeError, format!("Cannot read properties of null (reading '{}')", property_name), 0, 0))
              }
              if left.is_equal_to(self, &Value::Undefined, true) {
                return Err(JSIError::new( JSIErrorType::TypeError, format!("Cannot read properties of undefined (reading '{}')", property_name), 0, 0))
              }

              let left_clone = left.clone();
              let left_obj = left.to_object(self);
              let value = (*left_obj).borrow().get_value(property_name.clone());
              let property_valueinfo = ValueInfo {
                is_const: false,
                value,
                name: Some(property_name.clone()),
                access_path: format!("{}.{}", object.access_path, property_name),
                reference: Some(left_clone) 
              };
              self.stack.push(property_valueinfo);
            } else {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("property name arg is required"), 0, 0));
            }
          },
          EByteCodeop::OpAssign => {
            let right = self.stack.pop().unwrap();
            let mut left = self.stack.pop().unwrap();
            left.set_value(self, right.value)?;
            // 将左值的值推入栈
            self.stack.push(left);
          },
          EByteCodeop::OpFuncStart => {
            // 函数定义
            bytecode_index = self.run_function_def_bytecode(1, bytecode_index, bytecode_item, &bytecode);
          },
          EByteCodeop::OpCall => {
            // 函数调用
            let mut args = vec![];
            // 从栈中弹出参数
            let arg_count = bytecode_item.args.get(0).unwrap().parse::<usize>().unwrap_or(0);
            if arg_count > self.stack.len() {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("call args not enough"), 0, 0));
            }
            args  = self.stack.drain(self.stack.len() - arg_count..).collect();
              
            // 从栈中弹出函数
            if let Some(callee) = self.stack.pop() {
                match &callee.value {
                  Value::Function(function_object) => {
                    let mut reference = None;
                    if let Some(call_ref) = &callee.reference {
                      reference = call_ref.to_weak_rc_object();
                    }
                    let _ = self.call_function_with_bytecode(function_object.to_owned(), callee.reference, reference, args);
                  },
                  _ => {}
                }
            } else {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("call args not enough"), 0, 0));
            }
            
          },
          EByteCodeop::OpGetArg => {
            // OpCall 的时候 arg 是按照从左到右顺序插入的
            let arg = self.cur_scope.borrow_mut().function_call_args.remove(0);
            self.stack.push(arg);
          },
          EByteCodeop::OpReturn => {
            // 从栈中弹出一个值作为返回值
            if self.stack.len() > 0 {
              // 中断执行
              return Ok(Value::Interrupt(Token::Return, Expression::Unknown));
            } else {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("return value is required"), 0, 0));
            }
          },
          EByteCodeop::OpIfFalse => {
            // 从栈顶获取一个数据，转换为布尔值
            let condition = self.stack.pop().unwrap();
            let bool_value = condition.value.to_boolean(self);
            if !bool_value {
              // 如果是 false，那么跳转到下一个 bytecode_index
              if let Some(label) = bytecode_item.args.get(0) {
                // 将 label 和当前 bytecode_index 绑定
                return self.run_goto(cur_index, label.to_owned(), &bytecode);
              } else {
                return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("if false label arg is required"), 0, 0));
              }
            }
          },
          EByteCodeop::OpLabel => {
            if let Some(label) = bytecode_item.args.get(0) {
              // 将 label 和当前 bytecode_index 绑定
              self.label_index_map.insert(label.clone(), cur_index);
            } else {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("label arg is required"), 0, 0));
            }
          },
          EByteCodeop::OpGoto => {
            if let Some(label) = bytecode_item.args.get(0) {
              // 将 label 和当前 bytecode_index 绑定
              return self.run_goto(cur_index, label.to_owned(), &bytecode)
            }
            return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("goto label arg is required"), 0, 0));
          },
          EByteCodeop::OpAdd  | EByteCodeop::OpSub | EByteCodeop::OpMul | EByteCodeop::OpDiv |
          EByteCodeop::OpEqual | EByteCodeop::OpNotEqual | EByteCodeop::OpStrictEqual | EByteCodeop::OpStrictNotEqual |
          EByteCodeop::OpLessThan | EByteCodeop::OpLessThanOrEqual | EByteCodeop::OpGreaterThan | EByteCodeop::OpGreaterThanOrEqual
           => {
            let right = self.stack.pop().unwrap();
            let left = self.stack.pop().unwrap();
            let operator = match bytecode_item.op {
              EByteCodeop::OpAdd => Token::Plus,
              EByteCodeop::OpSub => Token::Subtract,
              EByteCodeop::OpMul => Token::Multiply,
              EByteCodeop::OpDiv => Token::Slash,
              EByteCodeop::OpEqual => Token::Equal,
              EByteCodeop::OpNotEqual => Token::NotEqual,
              EByteCodeop::OpStrictEqual => Token::StrictEqual,
              EByteCodeop::OpStrictNotEqual => Token::StrictNotEqual,
              EByteCodeop::OpLessThan => Token::Less,
              EByteCodeop::OpLessThanOrEqual => Token::LessOrEqual,
              EByteCodeop::OpGreaterThan => Token::Greater,
              EByteCodeop::OpGreaterThanOrEqual => Token::GreaterOrEqual,
              _ => {
                return Err(JSIError::new(JSIErrorType::SyntaxError, format!("Unsupported bytecode operation: {:?}", bytecode_item.op), 0, 0));
              }
            };
            // 执行二元运算
            let result = self.execute_binary_expression(&BinaryExpression {
              left: Box::new(Expression::Value(Box::new(left.value))),
              right: Box::new(Expression::Value(Box::new(right.value))),
              operator,
            })?;
            self.stack.push(result.to_value_info());
          },
          EByteCodeop::OpPrefixUnary => {
            if let Some(arg) = bytecode_item.args.get(0) {
              // 执行一元运算
              let result = self.execute_prefix_unary_bytecode(arg);
              self.stack.push(result);
            } else {
              return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("prefix unary arg is required"), 0, 0));
            }
          },
          _ => {
            println!("Unsupported bytecode operation: {:?}", bytecode_item.op);
          }
        }
      }
      return Ok(Value::Undefined);

    }

    pub fn dump_byte_code(&mut self, code: String) -> JSIResult<String> {
      let program = self.parse(code)?;
      let bytecode = program.bytecode;
      // 遍历 bytecode，生成文本
      let mut result = String::new();
      for (_, byte) in bytecode.iter().enumerate() {
       // 如果 arg 有值
        if let Some(arg) = byte.args.get(0) {
          result.push_str(&format!("{} {}\n", byte.op.to_string(), arg));
        } else {
          result.push_str(&format!("{}\n", byte.op.to_string()));
        }
      }
      Ok(result)
    }

    // 运行一段 JS 代码
    pub fn parse(&mut self, code: String) -> JSIResult<Program> {
      let mut ast = AST::new(code);
      ast.set_strict(self.strict);
      ast.parse()
    }


    pub fn run_function_def_bytecode(&mut self, level:usize, next_bytecode_index: usize, cur_bytecode: &ByteCode, bytecode_list: &Vec<ByteCode>) -> usize {
      let mut function_name = String::new();
      if let Some(name) = cur_bytecode.args.get(0) {
        function_name = name.clone();
      }
      // if let Some(index) = cur_bytecode.args.get(1) {
      //   function_index = index.clone();
      // }

      // let mut function_bytecode = vec![];
      let mut bytecode_index = next_bytecode_index;

      while bytecode_index < bytecode_list.len() {
        let next_bytecode = &bytecode_list[bytecode_index];
        bytecode_index += 1;
        match next_bytecode.op {
          EByteCodeop::OpFuncStart => {
            // 嵌套函数，递归调用
            let next_index = self.run_function_def_bytecode(level + 1, bytecode_index, 
              next_bytecode, bytecode_list);
            bytecode_index = next_index;
          },
          EByteCodeop::OpFuncEnd => {
            // 结束函数
            break;
          },
          _ => {}
        }
      }
      if level == 1 {
        // bytecode_index 是 OpFuncEnd
        let function_bytecode = bytecode_list[next_bytecode_index..bytecode_index - 1].to_vec();
        let function = create_function_with_bytecode(self, function_name.clone(), vec![], function_bytecode, Rc::downgrade(&self.cur_scope));
        (*self.cur_scope).borrow_mut().set_value(function_name.clone(), function.clone(), false);
      }
      
      return bytecode_index
    }


    fn run_goto(&mut self, cur_index: usize, label: String, bytecode: &Vec<ByteCode>) -> JSIResult<Value> {
      println!("goto label: {}", label);
      // hashMap 中寻找不到
      let label_start_index: usize = match self.label_index_map.get(&label) {
        Some(index) => *index,
        None => {
          let mut tmp_index = cur_index + 1;
          let mut found = false;
          while tmp_index < bytecode.len() {
            let tmp_bytecode = &bytecode[tmp_index];
            if tmp_bytecode.op == EByteCodeop::OpLabel && tmp_bytecode.args.get(0).map_or(false, |arg| arg == &label) {
              found = true;
              break;
            }
            tmp_index += 1;
          }
          if found {
            self.label_index_map.insert(label.clone(), tmp_index);
            tmp_index
          } else {
            return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("label not found"), 0, 0));
          }
        }
      };
      self.run_with_bytecode_list(label_start_index, bytecode)
    }

    fn call(&mut self, program: Program) -> JSIResult<Value> {
      let block_result = self.call_block(&program.declarations, &program.body)?;
      Ok(block_result.1)
    }

    fn call_block(&mut self, declarations: &Vec<Declaration>, body: &Vec<Statement>) -> JSIResult<(Value, Value, Value)> {
       // 绑定函数声明
       for declaration in declarations.iter() {
        match  declaration {
            Declaration::Function(function_statement) => {
              let function = create_function(self, &function_statement, Rc::downgrade(&self.cur_scope));
             (*self.cur_scope).borrow_mut().set_value(function_statement.name.literal.clone(), function, false)
            }
        };
      }
      // 函数声明需要添加 prototype、constrctor、__proto__
      // 绑定变量声明
      // 执行 statement
      // result_value 是 block 的返回值
      let mut result_value = Value::Undefined;
      let mut last_statement_value = Value::Undefined;
      // 中断，类似于 break 、continue 等
      let mut interrupt = Value::Undefined;
      for statement in body.iter() {
        let call_options = CallStatementOptions {
          label: None,
        };
        self.call_statement(statement, &mut result_value, &mut last_statement_value, &mut interrupt, call_options)?;
        if !interrupt.is_equal_to(self, &Value::Undefined, true) {
          // 如果是 return，那么 block 的返回值(result_value) 在处理 return 时已经赋值了
          break;
        }
      }
      Ok((result_value, last_statement_value, interrupt))
    }

    fn call_statement(&mut self, statement: &Statement, result_value: &mut Value, last_statement_value: &mut Value, interrupt: &mut Value, call_options: CallStatementOptions) -> JSIResult<bool> {
      match statement {
        Statement::Var(var_statement) => {
          // var_statement.flag 是 var 还是 let，在上层调用链路中处理
          for variable in var_statement.list.iter() {
            if let Expression::Var(let_var) = variable {
              let name = let_var.name.clone();
              let mut value = self.execute_expression(&let_var.initializer)?;
              value.bind_name(name.clone());
              (*last_statement_value) = value.clone();
              (*self.cur_scope).borrow_mut().set_value(name, value, var_statement.flag == VariableFlag::Const);
            }
          }
          Ok(true)
        },
        Statement::Expression(expression) => {
          (*last_statement_value) = self.execute_expression(&expression.expression)?;
          Ok(true)
        },
        Statement::Return(return_statement) => {
          let result = self.execute_expression(&return_statement.expression)?;
          (*result_value) = result;
          (*last_statement_value) = result_value.clone();
          (*interrupt) = Value::Interrupt(Token::Return, Expression::Unknown);
          Ok(true)
        },
        Statement::Function(_) => {
          // skip, 因为函数声明前置了
          Ok(true)
        },
        Statement::If(if_statement) => {
          let condition = self.execute_expression(&if_statement.condition)?;
          let call_options = CallStatementOptions {
            label: None,
          };
          if condition.to_boolean(self) {
            if let Statement::Unknown = *if_statement.then_statement {
              // no then
            } else {
              self.call_statement(&if_statement.then_statement, result_value, last_statement_value, interrupt, call_options)?;
            }
          } else {
            if let Statement::Unknown = *if_statement.else_statement {
              // no else
            } else {
              self.call_statement(&if_statement.else_statement, result_value, last_statement_value, interrupt, call_options)?;
            }
          }
          Ok(true)
        },
        Statement::Label(label_statement) => {
          let call_options = CallStatementOptions {
            label: Some(label_statement.label.literal.clone()),
          };
          self.cur_scope.borrow_mut().labels.push(label_statement.label.literal.clone());
          self.call_statement(&label_statement.statement, result_value, last_statement_value, interrupt, call_options)?;
          self.cur_scope.borrow_mut().labels.pop();
          Ok(true)
        },
        Statement::For(for_statment) => {
          self.execute_for(for_statment, result_value, last_statement_value, interrupt, call_options)
        },
        Statement::While(for_statment) => {
          self.execute_for(for_statment, result_value, last_statement_value, interrupt, call_options)
        },
        Statement::Switch(switch_statement) => {
          self.execute_switch(switch_statement, result_value, last_statement_value, interrupt, call_options)
        },
        Statement::Try(try_statement) => {
          self.switch_scope(Some(Rc::clone(&self.cur_scope)));
          let result = self.call_block(&vec![], &try_statement.body.statements);
          self.close_scope();
          if let Ok(value) = &result {
            (*result_value) = value.0.clone();
            (*last_statement_value) = value.1.clone();
            (*interrupt) = value.2.clone();
          } else if let Err(err) = &result {
            if let Some(catch) = &try_statement.catch {
              self.switch_scope(Some(Rc::clone(&self.cur_scope)));
              if let Some(error_decl) =&catch.declaration {
                let err_obj = err.to_error_object(self);
                (*self.cur_scope).borrow_mut().set_value(error_decl.literal.clone(), Value::Object(err_obj), false);
              }
              let result = self.call_block(&vec![], &catch.body.statements)?;
              (*result_value) = result.0;
              (*last_statement_value) = result.1;
              (*interrupt) = result.2;
              self.close_scope();
            }
          }
          // TODO: finaly
          
          Ok(true)
        },
        Statement::Throw(throw) => {
          let throw_value = self.execute_expression(&throw.expression)?;
          let mut err = JSIError::new(JSIErrorType::Unknown, format!(""), 0, 0);
          err.set_value(throw_value);
          Err(err)
        },
        Statement::Block(block) => {
          self.switch_scope(Some(Rc::clone(&self.cur_scope)));
          let result = self.call_block(&vec![], &block.statements)?;
          (*result_value) = result.0;
          (*interrupt) = result.2;
          self.close_scope();
          Ok(true)
        },
        Statement::Break(break_statement) => {
          if let Some(label) = &break_statement.label {
            let label_exists = self.cur_scope.borrow_mut().labels.contains(&label.literal);
            if !label_exists {
              // TODO: throw error label not exists
            }
            (*interrupt) = Value::Interrupt(Token::Break,Expression::Identifier(IdentifierLiteral {literal: label.literal.clone()}));
          } else {
            (*interrupt) = Value::Interrupt(Token::Break, Expression::Unknown);
          }
          Ok(true)
        },
        Statement::Continue(continue_statement) => {
          if let Some(label) = &continue_statement.label {
            let label_exists = self.cur_scope.borrow_mut().labels.contains(&label.literal);
            if !label_exists {
              // TODO: throw error label not exists
            }
            (*interrupt) = Value::Interrupt(Token::Continue,Expression::Identifier(IdentifierLiteral {literal: label.literal.clone()}));
          } else {
            (*interrupt) = Value::Interrupt(Token::Continue, Expression::Unknown);
          }
          Ok(true)
        },
        _ => {
          Err(JSIError::new(JSIErrorType::Unknown, format!("unknown statement: {:?}", statement), 0, 0))
        }
      }
    }

    fn execute_expression(&mut self, expression: &Expression) -> JSIResult<Value> {
      let expr = self.execute_expression_info(expression)?;
      Ok(expr.value)
    }

    fn execute_expression_info(&mut self, expression: &Expression) -> JSIResult<ValueInfo> {
      // println!("expression: {:?}", expression);
      match expression {
        Expression::Value(valuebox) => {
          let value = *valuebox.clone();
          Ok(ValueInfo { is_const: false, value, name: None, access_path: String::from(""), reference: None })
        },
        Expression::Binary(binary) => {
          Ok(ValueInfo { is_const: false, value:self.execute_binary_expression(binary)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::PrefixUnary(expr) => {
          Ok(ValueInfo { is_const: false, value: self.execute_prefix_unary_expression(expr)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::PostfixUnary(expr) => {
          Ok(ValueInfo { is_const: false, value: self.execute_postfix_unary_expression(expr)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::Call(call) => {
          Ok(ValueInfo { is_const: false, value: self.execute_call_expression(call)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::Object(object) => {
          Ok(ValueInfo { is_const: false, value: self.new_object(object)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::Array(array) => {
          Ok(ValueInfo { is_const: false, value: self.new_array(array)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::Function(function_declaration) => {
          let func = create_function(self, function_declaration, Rc::downgrade(&self.cur_scope));
          Ok(ValueInfo { is_const: false, value: func, name: None, access_path: String::from(""), reference: None })
        },
        Expression::PropertyAccess(property_access) => {
          // expression.name
          let left_info = self.execute_expression_info(&property_access.expression)?;
          let left = left_info.value;
          if left.is_equal_to(self, &Value::Null, true) {
            return Err(JSIError::new( JSIErrorType::TypeError, format!("Cannot read properties of null (reading '{}')", property_access.name.literal), 0, 0))
          }
          if left.is_equal_to(self, &Value::Undefined, true) {
            return Err(JSIError::new( JSIErrorType::TypeError, format!("Cannot read properties of undefined (reading '{}')", property_access.name.literal), 0, 0))
          }
          let left_clone = left.clone();
          let left_obj = left.to_object(self);
          let right = &property_access.name.literal;
          let value = (*left_obj).borrow().get_value(right.clone());
          Ok(ValueInfo { is_const: false, value, name: Some(right.clone()), access_path: format!("{}.{}", left_info.access_path, property_access.name.literal), reference: Some(left_clone) })
        },
        Expression::ComputedPropertyName(property_name) => {
          Ok(ValueInfo { is_const: false, value: self.execute_expression(&property_name.expression)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::ElementAccess(element_access) => {
          // expression[argument]
          let left = self.execute_expression(&element_access.expression)?;
          
          let left_obj = left.to_object(self);
          let right = self.execute_expression(&element_access.argument)?.to_string(self);
          if left.is_equal_to(self, &Value::Null, true) {
            return Err(JSIError::new( JSIErrorType::TypeError, format!("Cannot read properties of null (reading '{}')", right), 0, 0))
          }
          if left.is_equal_to(self, &Value::Undefined, true) {
            return Err(JSIError::new( JSIErrorType::TypeError, format!("Cannot read properties of undefined (reading '{}')", right), 0, 0))
          }
          let value = (*left_obj).borrow().get_value(right.clone());
          Ok(ValueInfo { is_const: false, value, name: Some(right.clone()),  access_path: String::from(""),reference: Some(Value::Object(left_obj)) })
        },
        Expression::Conditional(condition) => {
          let condition_res = self.execute_expression(&condition.condition)?;
          let value = if condition_res.to_boolean(self) {
            self.execute_expression(&condition.when_true)?
          } else {
            self.execute_expression(&condition.when_false)?
          };
          Ok(ValueInfo { is_const: false, value, name: None,  access_path: String::from(""), reference: None })
        },
        Expression::Identifier(identifier) => {
          let name = identifier.literal.clone();
          let (value, scope, is_const) = get_value_and_scope(Rc::clone(&self.cur_scope), name.clone());
          if let Some(val) = value {
            Ok(ValueInfo { is_const, value: val, name: Some(name.clone()),  access_path: name.clone(),reference: Some(Value::Scope(Rc::downgrade(&scope))) })
          } else {
            Err(JSIError::new(JSIErrorType::ReferenceError, format!("{} is not defined", name), 0, 0))
          }
        },
        Expression::Assign(assign) => {
          Ok(ValueInfo { is_const: false, value: self.execute_assign_expression(assign)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::String(string) => {
          Ok(ValueInfo {is_const: false, value: Value::String(string.value.clone()), name: None, access_path: string.value.clone(), reference: None })
        },
        Expression::TemplateLiteral(template) => {
          let mut strings: Vec<String> = vec![];
          for span in template.spans.iter() {
            let value = self.execute_expression(span)?;
            strings.push(value.to_string(self));
          }
          let string = strings.join("");
          Ok(ValueInfo {is_const: false, value: Value::String(string.clone()), name: None, access_path:string, reference: None })
        },
        Expression::Number(number) => {
          Ok(ValueInfo {is_const: false, value: Value::Number(number.value.clone()), name: None, access_path: number.literal.clone(), reference: None })
        },
        Expression::New(new_object) => {
          Ok(ValueInfo { is_const: false, value: self.execute_new_expression(new_object)?, name: None, access_path: String::from(""),reference: None })
        },
        Expression::Keyword(keyword) => {
          Ok(ValueInfo {
            is_const: false,
            value: match *keyword {
              Keywords::False => Value::Boolean(false),
              Keywords::True => Value::Boolean(true),
              Keywords::Null => Value::Null,
              Keywords::This => {
                let scope = self.cur_scope.borrow();
                if let Some(this) = &scope.this {
                  this.clone()
                } else {
                  Value::Undefined
                }
              },
              _ => Value::Undefined,
            },
            name: None,
            reference: None,
            access_path: keyword.to_string(),
          })
        },
        Expression::Group(group) => {
          self.execute_expression_info(&group.expression)
        },
        Expression::Sequence(sequence) => {
          let mut last_result = ValueInfo {
            is_const: false,
            value: Value::Undefined,
            name: None,
            reference: None,
            access_path: String::from(""),
          };
          for expr in sequence.expressions.iter() {
            last_result = self.execute_expression_info(&expr)?;
          }
          Ok(last_result)
        },
        _ => {
          Err(JSIError::new(JSIErrorType::Unknown, format!("expression unsupported {:?}", expression), 0, 0))
        },
      }
    }

    // 执行基础四则运算
    fn execute_binary_expression(&mut self, expression: &BinaryExpression) -> JSIResult<Value> {
      let left = self.execute_expression(expression.left.as_ref())?;
      // 逻辑运算 左值
      if expression.operator == Token::LogicalAnd {
        // false &&
        if !left.to_boolean(self) {
          return Ok(Value::Boolean(false));
        }
      } else if expression.operator == Token::LogicalOr {
        // true ||
        if left.to_boolean(self) {
          return Ok(Value::Boolean(true));
        }
      }

      let right = self.execute_expression(expression.right.as_ref())?;

      // 逻辑运算 右值
      if expression.operator == Token::LogicalAnd || expression.operator == Token::LogicalOr {
        // true && false / false || false
        if !right.to_boolean(self) {
          return Ok(Value::Boolean(false));
        } else {
          // true && true / false || true
          return Ok(Value::Boolean(true));
        }
      }
      match expression.operator {
        Token::Equal => {
          return Ok(Value::Boolean(left.is_equal_to(self, &right, false)));
        },
        Token::NotEqual => {
          return Ok(Value::Boolean(!left.is_equal_to(self, &right, false)));
        },
        Token::StrictEqual => {
          return Ok(Value::Boolean(left.is_equal_to(self, &right, true)));
        },
        Token::StrictNotEqual => {
          return Ok(Value::Boolean(!left.is_equal_to(self, &right, true)));
        },
        Token::Plus | Token::Subtract | Token::Multiply | Token::Slash |Token::Remainder => {
          // 数字处理
          if left.is_nan() || right.is_nan() {
            return Ok(Value::NAN);
          }
          
          // 加法的特殊处理
          if expression.operator == Token::Plus {
            // 如果有一个是字符串，那就返回字符串
            if left.is_string() || right.is_string() {
              return Ok(Value::String(left.to_string(self) + right.to_string(self).as_str()));
            }
          }

          // 除法的特殊处理
          if expression.operator == Token::Slash {
            if left.is_infinity() && right.is_infinity() {
              return Ok(Value::NAN);
            }
          }

          // 计算数字运算
          // TODO: JSIResult
          Ok(self.execute_number_operator_expression(&left, &right, &expression.operator))
        },
        Token::Less | Token::Greater | Token::LessOrEqual | Token::GreaterOrEqual => {
          self.execute_compare_operator_expression(&left, &right, &expression.operator)
        },
        _ =>  {
          Err(JSIError::new(JSIErrorType::Unknown, format!("unsupport binary {:?}", expression), 0, 0))
        }
      }

    }

    // 执行数字运算表达式
    fn execute_number_operator_expression(&mut self, left: &Value, right: &Value, operator: &Token) -> Value {
      let left_number: f64;
      let right_number: f64;
      if let Some(num) = left.to_number(self) {
        left_number = num;
      } else {
        return Value::NAN;
      }
      if let Some(num) = right.to_number(self) {
        right_number = num;
      } else {
        return Value::NAN;
      }
      match operator {
        Token::Plus => Value::Number(left_number + right_number),
        Token::Subtract => Value::Number(left_number - right_number),
        Token::Multiply => Value::Number(left_number * right_number),
        Token::Slash => Value::Number(left_number / right_number),
        Token::Remainder => Value::Number(left_number % right_number),
        _=> Value::NAN,
      }
    }

    // 执行方法调用表达式
    fn execute_compare_operator_expression(&mut self, left: &Value, right: &Value, operator: &Token) -> JSIResult<Value> {
      let left_number: f64;
      let right_number: f64;
      if let Some(num) = left.to_number(self) {
        left_number = num;
      } else {
        return Err(JSIError::new(JSIErrorType::SyntaxError, format!("Unexpected token '{:?}'", operator), 0, 0))
      }
      if let Some(num) = right.to_number(self) {
        right_number = num;
      } else {
        return Err(JSIError::new(JSIErrorType::SyntaxError, format!("Unexpected token '{:?}'", operator), 0, 0))
      }
      match operator {
        Token::Greater => Ok(Value::Boolean(left_number > right_number)),
        Token::GreaterOrEqual => Ok(Value::Boolean(left_number >= right_number)),
        Token::Less => Ok(Value::Boolean(left_number < right_number)),
        Token::LessOrEqual => Ok(Value::Boolean(left_number <= right_number)),
        _=> Err(JSIError::new(JSIErrorType::SyntaxError, format!("Unexpected token '{:?}'", operator), 0, 0)),
      }
    }

    // 执行方法调用表达式
    fn execute_call_expression(&mut self, expression: &CallExpression) -> JSIResult<Value> {
      let callee = self.execute_expression_info(expression.expression.as_ref())?;
      let mut arguments: Vec<Value> = vec![];
      for arg in expression.arguments.iter() {
        arguments.push(self.execute_expression(arg)?);
      }
      match &callee.value {
        Value::Function(function_object) => {
          let mut reference = None;
          if let Some(call_ref) = &callee.reference {
            reference = call_ref.to_weak_rc_object();
          }
          // TODO: call this
          return self.call_function_object(function_object.to_owned(), callee.reference, reference, arguments);
        },
        Value::RefObject(obj_ref) => {
          let obj = obj_ref.upgrade();
          if let Some(obj_rc) = obj {
            // 全局对象调用，可以认为是 new 
            let is_global_object = {
              let function_mut = obj_rc.borrow_mut();
              function_mut.get_inner_property_value(IS_GLOABL_OBJECT.to_string())
            };
            if let Some(_) = is_global_object {
              return callee.value.instantiate_object(self, arguments);
            }
          }
          Err(JSIError::new(JSIErrorType::TypeError, format!("{:?} is not a function", callee.access_path), 0, 0))
        },
        _ => {
          Err(JSIError::new(JSIErrorType::TypeError, format!("{:?} is not a function", callee.access_path), 0, 0))
        }
      }
    }

    // 执行赋值表达式
    fn execute_assign_expression(&mut self, expression: &AssignExpression) -> JSIResult<Value> {
      let mut left_info = self.execute_expression_info(&expression.left)?;
      let mut right_value = self.execute_expression(&expression.right)?;
      let mut oper = expression.operator.clone();
      let binary = match &oper {
        Token::AddAssign => {
          Some(Token::Plus)
        },
        Token::SubtractAssign => {
          Some(Token::Subtract)
        },
        Token::MultiplyAssign => {
          Some(Token::Multiply)
        },
        Token::SlashAssign => {
          Some(Token::Slash)
        },
        Token::RemainderAssign => {
          Some(Token::Remainder)
        },
        _ => {
          None
        }
      };
      if let Some(operator) = binary {
        oper = Token::Assign;
        let binary_expression = BinaryExpression {
          left: expression.left.clone(),
          right: expression.right.clone(),
          operator: operator
        };
        right_value = self.execute_binary_expression(&binary_expression)?;
      }
      match oper {
        Token::Assign => {
          left_info.set_value(self, right_value.clone())?;
          Ok(left_info.value)
        },
        _ => {
          Err(JSIError::new(JSIErrorType::SyntaxError, String::from("todo: unsupported operator"), 0, 0))
        }
      }
    }

    fn execute_prefix_unary_bytecode(&mut self, operator: &String) -> ValueInfo {
      // 获取栈顶的值
      let mut value_info = self.stack.pop().unwrap();
      let value = value_info.value.clone();
      let new_value_info =  match operator.as_str() {
        // Token::Not | Token::BitwiseNot | Token::Plus | Token::Subtract
        "!" => {
          Value::Boolean(!value.to_boolean(self)).to_value_info()
        },
        "+" => {
          let value_number = value.to_number(self);
          if let Some(num) = value_number {
            Value::Number(num).to_value_info()
          } else {
            Value::NAN.to_value_info()
          }
        },
        "-" => {
          let value_number = value.to_number(self);
          if let Some(num) = value_number {
            Value::Number(-num).to_value_info()
          } else {
            Value::NAN.to_value_info()
          }
        },
        // Token::Typeof | Token::Void
        "typeof" => {
          Value::String(value.type_of()).to_value_info()
        },
        "void" => {
          Value::Undefined.to_value_info()
        },
        "++" => {
          let value_number = value.to_number(self);
          if let Some(num) = value_number {
            let new_value = Value::Number(num + 1f64);
            value_info.set_value(self, new_value.clone());
            new_value.to_value_info()
          } else {
            Value::NAN.to_value_info()
          }
        },
        "--" => {
          let value_number = value.to_number(self);
          if let Some(num) = value_number {
            let new_value = Value::Number(num - 1f64);
            value_info.set_value(self, new_value.clone());
            new_value.to_value_info()
          } else {
            Value::NAN.to_value_info()
          }
        },
       // TODO:  | Token::Delete | Token::Await
        _ => {
          println!("Unsupported prefix unary operator: {}", operator);
          Value::Undefined.to_value_info()
        }
      };
      return new_value_info;
    }
    // 执行 ++i --i
    fn execute_prefix_unary_expression(&mut self, expression: &PrefixUnaryExpression) -> JSIResult<Value> {
      
      let operand_info = self.execute_expression_info(&expression.operand);
      if &Token::Typeof == &expression.operator {
        if let Err(_) = operand_info {
          return Ok(Value::String(String::from("undefined")));
        }
      }
      let mut operand_info = operand_info?;
      match &expression.operator {
        Token::Typeof => {
          Ok(Value::String(operand_info.value.type_of()))
        },
        Token::Void => {
          Ok(Value::Undefined)
        },
        Token::Not => {

          Ok(Value::Boolean(!operand_info.value.to_boolean(self)))
        },
        // TODO: delete
        // Token::Void => {
        //   Ok(Value::Undefined)
        // },
        // TODO: await
        _ => {
          let value_number = operand_info.value.to_number(self);
          let value = if let Some(new_value) = value_number {
            let mut new_value = new_value;
            let mut is_need_set_value = false;
            match &expression.operator {
              Token::Increment => {
                new_value = new_value + 1f64;
                is_need_set_value = true;
              },
              Token::Decrement => {
                new_value = new_value - 1f64;
                is_need_set_value = true;
              },
              Token::Subtract => {
                new_value = -new_value;
              },
              Token::Plus => {
                new_value = new_value;
              },
              _ => {}
            }
            let value = Value::Number(new_value);
            if is_need_set_value {
              operand_info.set_value(self, value.clone())?;
            }
            value
          } else {
            Value::NAN
          };
          
          Ok(value)
        }
      }

      
    }

    // 执行 i++ i--
    fn execute_postfix_unary_expression(&mut self, expression: &PostfixUnaryExpression) -> JSIResult<Value> {
      let mut operand_info = self.execute_expression_info(&expression.operand)?;
      let origin_value = operand_info.value.clone();
      let value_number = origin_value.to_number(self);
      let value = if let Some(new_value) = value_number {
        let mut new_value = new_value;
        match &expression.operator {
            Token::Increment => {
              new_value = new_value + 1f64;
            },
            Token::Decrement => {
              new_value = new_value - 1f64;
            },
            _ => {}
        }
        Value::Number(new_value)
      } else {
        Value::NAN
      };
      operand_info.set_value(self, value)?;
      Ok(origin_value)
    }

    // 执行循环
    fn execute_for(&mut self, for_statment: &ForStatement, _: &mut Value, _: &mut Value, interrupt: &mut Value, call_options: CallStatementOptions) -> JSIResult<bool> {
      let mut is_change_scope = false;
      let initializer = *for_statment.initializer.clone();
      let mut for_result = Value::Undefined;
      let mut for_last_statement_value = Value::Undefined;
      let mut for_interrupt = Value::Undefined;
      let for_call_options = CallStatementOptions {
        label: None,
      };
      if let Statement::Var(var) = &initializer  {
        if var.flag == VariableFlag::Var {
          self.call_statement(&initializer, &mut for_result, &mut for_last_statement_value, &mut for_interrupt, for_call_options)?;
        } else if var.flag == VariableFlag::Let || var.flag == VariableFlag::Const {
          self.switch_scope(Some(Rc::clone(&self.cur_scope)));
          is_change_scope = true;
          self.call_statement(&initializer, &mut for_result, &mut for_last_statement_value, &mut for_interrupt, for_call_options)?;
        }
      } else if let Statement::Unknown = &initializer {
        // nothing to do
      } else {
        self.call_statement(&initializer, &mut for_result, &mut for_last_statement_value, &mut for_interrupt, for_call_options)?;
      }

      if !is_change_scope {
        self.switch_scope(Some(Rc::clone(&self.cur_scope)));
      }

      loop {
        if !for_statment.post_judgment {
          let condition = &for_statment.condition;
          
          if let Expression::Unknown = condition {
            // nothing to do
          } else {
            let value = self.execute_expression(condition)?;
            if !value.to_boolean(self) {
              break;
            }
          }
        }
        
        // TODO: expression
        if let Statement::Block(block) = for_statment.statement.as_ref() {
          let result = self.call_block(&vec![], &block.statements)?;
         
          let for_interrupt = result.2.clone();
          if let Value::Interrupt(token, expr) = for_interrupt {
            if token == Token::Break {
              if let Expression::Identifier(identifier) = expr {
                if let Some(last_label_str) = &call_options.label {
                  if *last_label_str == identifier.literal {
                    // break 当前循环，无需做任何处理
                  } else {
                    // 向上走，让上层循环继续处理
                    (*interrupt) = result.2;
                  }
                } else {
                  // 向上走，让上层循环继续处理
                  (*interrupt) = result.2;
                }
              }
              break;
            } else if token == Token::Continue {
              if let Expression::Identifier(identifier) = expr {
                if let Some(last_label_str) = &call_options.label {
                  if *last_label_str == identifier.literal {
                    // continue 当前循环，无需做任何处理
                  } else {
                    // 有 label ，但是不一样，向上走，让上层循环继续处理
                    (*interrupt) = result.2;
                    break;
                  }
                } else {
                  // 当前循环没有 label，向上走，让上层循环继续处理
                  (*interrupt) = result.2;
                  break;
                }
              }
            }
          }
        }

        let incrementor = &for_statment.incrementor;
        if let Expression::Unknown = incrementor {
          // nothing to do
        } else {
          self.execute_expression(incrementor)?;
        }

        // post judegment: for do while
        if for_statment.post_judgment {
          let condition = &for_statment.condition;
        
          if let Expression::Unknown = condition {
            // nothing to do
          } else {
            let value = self.execute_expression(condition)?;
            if !value.to_boolean(self) {
              break;
            }
          }
        }
      }
      self.close_scope();
      Ok(true)
    }


    // 执行循环
    fn execute_switch(&mut self, switch_statment: &SwitchStatement, _: &mut Value, _: &mut Value, _: &mut Value, _: CallStatementOptions) -> JSIResult<bool> {
      let value = self.execute_expression(&switch_statment.condition).unwrap();
      
      let mut matched: i32 = switch_statment.default_index;
      let clause_len = switch_statment.clauses.len();
      for case_index in 0..clause_len {
        if case_index as i32 == switch_statment.default_index {
          continue;
        }
        let case = &switch_statment.clauses[case_index];
        if let Some(condition) = &case.condition {
          let case_value = self.execute_expression(condition).unwrap();
          if case_value.is_equal_to(self, &value, true) {
            matched = case_index as i32;
          }
        }
      }

      for case_index in (matched as usize)..clause_len {
        let case = &switch_statment.clauses[case_index];
        let result = self.call_block(&vec![], &case.statements)?;
        if let Value::Interrupt(token, _) = result.2 {
          if token == Token::Break {
            break;
          }
        }
      }
      Ok(true)
    }

    fn execute_new_expression(&mut self, new_object: &NewExpression) -> JSIResult<Value> {
      let constructor = self.execute_expression_info(new_object.expression.as_ref())?;
      let mut arguments: Vec<Value> = vec![];
      for element in &new_object.arguments {
        arguments.push(self.execute_expression(element)?);
      }

      // new function
      if let Value::Function(function_declare) = &constructor.value {
       
        let prototype = {
          let func_clone = Rc::clone(function_declare);
          let func_borrow = func_clone.borrow();
          func_borrow.prototype.clone()
        };
        
        if let Some(proto) = prototype {
          let obj = create_object(self, ClassType::Object, None);
           // 绑定当前对象的原型
           {
            let obj_clone = Rc::clone(&obj);
            let mut obj_borrowed = obj_clone.borrow_mut();
            obj_borrowed.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&proto)));
           }
           
           // 执行构造函数
          self.call_function_object(Rc::clone(function_declare), Some(Value::Object(Rc::clone(&obj))), None, arguments)?;
          return Ok(Value::Object(obj))
        }
      }

      let obj = constructor.value.instantiate_object(self, arguments);
      if let Ok(obj) = obj {
        return Ok(obj)
      }

      return Err(JSIError::new(JSIErrorType::TypeError, format!("{} is not a constructor", constructor.access_path), 0, 0))
    }

    fn new_object(&mut self, expression: &ObjectLiteral) -> JSIResult<Value> {
      // 获取 object 实例
      let object = create_object(self,ClassType::Array, None);
      let object_clone = Rc::clone(&object);
      let mut object_mut = (*object_clone).borrow_mut();
      // 绑定属性
      let mut normal_propertys: Vec<(String, Value)> = vec![];
      for property_index in 0..expression.properties.len() {
        let property = &expression.properties[property_index];
        let x = self.execute_expression(&property.name);
        let name = self.execute_expression(&property.name)?.to_string(self);
        let mut initializer = self.execute_expression(&property.initializer)?;
        initializer.bind_name(name.clone());
        // ComputedPropertyName 优先级更高，影响 object 的属性顺序
        if let Expression::ComputedPropertyName(_) = *property.name {
          object_mut.define_property(name, Property {
            enumerable: true,
            value: initializer,
          });
        } else {
          normal_propertys.push((name, initializer));
        }
      }

      for property_index in 0..normal_propertys.len() {
        let property = normal_propertys[property_index].to_owned();
        object_mut.define_property(property.0, Property {
          enumerable: true,
          value: property.1,
        });
      }
      Ok(Value::Object(object))
    }

    fn new_array(&mut self, expression: &ArrayLiteral) -> JSIResult<Value> {
      let array = create_array(self, 0);
      if let Value::Array(arr_obj) = &array {
        let mut arguments: Vec<Value> = vec![];
        for element in &expression.elements {
          arguments.push(self.execute_expression(element)?);
        }
        let call_ctx = &mut CallContext {
          ctx: self,
          this: Value::Array(Rc::clone(arr_obj)),
          reference: None,
          func_name: String::from("push")
        };
        Object::call(call_ctx, String::from("push"), arguments)?;
      }
      Ok(array)
    }

    pub fn call_function_with_bytecode(&mut self, function_define: Rc<RefCell<Object>>, call_this: Option<Value>, reference: Option<Weak<RefCell<Object>>>, args: Vec<ValueInfo>) -> JSIResult<Value> {
      // 获取 function 定义
      let function_define_value = (*function_define).borrow_mut().get_initializer().unwrap();
      // 获取 function 调用的 this
      let this_obj = match (*function_define).borrow_mut().get_inner_property_value(String::from("this")) {
        Some(bind_this_value) => bind_this_value,
        _ =>{
          if let Some(call_this_value) = call_this {
            call_this_value
          } else {
            Value::Undefined
          }
        }
      };
      // 内置方法
      if let Statement::BuiltinFunction(builtin_function) = *function_define_value {
        let func_name = get_builtin_function_name(self, &function_define);
        let mut ctx = CallContext{
          ctx: self,
          this: this_obj,
          reference: reference,
          func_name,
        };
        let arguments = args.iter().map(|info| info.value.clone()).collect::<Vec<Value>>();
        let builtin_res = (builtin_function)(&mut ctx, arguments)?;
        self.stack.push(builtin_res.to_value_info());
        return Ok(Value::Undefined);
      }
      // 获取 function 的 bytecode
      let bytecode_list_value =  (*function_define).borrow_mut().get_inner_property_value(String::from("bytecode")).unwrap();
      let bytecode_list = match bytecode_list_value {
        Value::ByteCode(bytecode_list) => bytecode_list,
        _ => vec![],
      };
       // 创建新的执行作用域
      let define_scope = (*function_define).borrow_mut().get_inner_property_value(String::from("define_scope"));
      let mut define_scope_value = None;
      if let Some(scope_value) = define_scope {
        if let Value::Scope(scope) = scope_value {
          define_scope_value = scope.upgrade();
        }
      }
      self.switch_scope(define_scope_value);

      let argument_object = create_object(self, ClassType::Object, None);
      {
        let argument_object_rc = Rc::clone(&argument_object);
        let mut argument_object_mut = argument_object_rc.borrow_mut();
        argument_object_mut.define_property(String::from("length"),  Property { enumerable: false, value: Value::Number(args.len() as f64) });
        let mut arguments_index = 0;
        for value_info in args.iter() { 
          argument_object_mut.define_property(arguments_index.to_string(), Property { enumerable: true, value: value_info.value.clone() });
          arguments_index += 1
        }
      }

      (*self.cur_scope).borrow_mut().set_value(String::from("arguments"), Value::Object(argument_object), false);
      (*self.cur_scope).borrow_mut().this = Some(this_obj);
      (*self.cur_scope).borrow_mut().function_call_args = args.clone();
      let _ =  self.run_with_bytecode_list(0, &bytecode_list);
      self.close_scope();
      // 这个return 其实没啥用，都是走 stack 
      Ok(Value::Undefined)
    }

    // 调用方法
    // call_this 指向调用时的 this
    // reference 指向
    pub fn call_function_object(&mut self, function_define: Rc<RefCell<Object>>, call_this: Option<Value>, reference: Option<Weak<RefCell<Object>>>, arguments: Vec<Value>) -> JSIResult<Value> {
      // 获取 function 定义
      let function_define_value = (*function_define).borrow_mut().get_initializer().unwrap();
      // 获取 function 调用的 this
      let this_obj = match (*function_define).borrow_mut().get_inner_property_value(String::from("this")) {
        Some(bind_this_value) => bind_this_value,
        _ =>{
          if let Some(call_this_value) = call_this {
            call_this_value
          } else {
            Value::Undefined
          }
        }
      };
     
      // 内置方法
      if let Statement::BuiltinFunction(builtin_function) = *function_define_value {
        let func_name = get_builtin_function_name(self, &function_define);
        let mut ctx = CallContext{
          ctx: self,
          this: this_obj,
          reference: reference,
          func_name,
        };
        return (builtin_function)(&mut ctx, arguments);
      }

      let function_declaration =  match *function_define_value {
        Statement::Function(function_declaration) => Some(function_declaration),
        _ => None,
      }.unwrap();
      // 创建新的执行作用域
      let define_scope = (*function_define).borrow_mut().get_inner_property_value(String::from("define_scope"));
      let mut define_scope_value = None;
      if let Some(scope_value) = define_scope {
        if let Value::Scope(scope) = scope_value {
          define_scope_value = scope.upgrade();
        }
      }
      self.switch_scope(define_scope_value);

      let argument_object = create_object(self, ClassType::Object, None);
      {
        let argument_object_rc = Rc::clone(&argument_object);
        let mut argument_object_mut = argument_object_rc.borrow_mut();
        argument_object_mut.define_property(String::from("length"),  Property { enumerable: false, value: Value::Number(arguments.len() as f64) });
        let mut arguments_index = 0;
        for value in arguments.iter() { 
          argument_object_mut.define_property(arguments_index.to_string(), Property { enumerable: true, value: value.clone() });
          arguments_index += 1
        }
      }
      (*self.cur_scope).borrow_mut().set_value(String::from("arguments"), Value::Object(argument_object), false);
      (*self.cur_scope).borrow_mut().this = Some(this_obj);
      // 绑定参数
      for parameter_index in 0..function_declaration.parameters.len() {
        if parameter_index < arguments.len() {
          // TODO: 参数引用
          (*self.cur_scope).borrow_mut().set_value(function_declaration.parameters[parameter_index].name.literal.clone(), arguments[parameter_index].clone(), false);
        } else {
          (*self.cur_scope).borrow_mut().set_value(function_declaration.parameters[parameter_index].name.literal.clone(), Value::Undefined, false);
        }
      }
      // 执行 body
      let result = self.call_block(&function_declaration.declarations, &function_declaration.body.statements)?;
      self.close_scope();
      Ok(result.0)
    }

    // 切换作用域
    fn switch_scope(&mut self, define_scope: Option<Rc<RefCell<Scope>>>) {
      // 创建新的作用域
      let mut new_scope = Scope::new();
      // 作用域的父级为定义是的作用域，而不是调用时的作用域
      new_scope.parent = define_scope;
      if let Some(scope) = &new_scope.parent {
        let rc = Rc::clone(scope);
        let scope = rc.borrow();
        new_scope.labels = scope.labels.clone();
      }
      // 添加调用时的来源作用域，调用完成之后得关闭当前作用域，回到调用时的作用域
      new_scope.from = Some(Rc::clone(&self.cur_scope));
      let scope_rc = Rc::new(RefCell::new(new_scope));
      let rc = Rc::clone(&scope_rc);
      (*self.cur_scope).borrow_mut().childs.push(scope_rc);
      self.cur_scope = rc;
    }

    // 退出作用域
    fn close_scope(&mut self) {
      let cur = Rc::clone(&self.cur_scope);
      let cur_rc = cur.borrow();
      if let Some(parent_scope_rc) = &cur_rc.from {
        let len = (*parent_scope_rc).borrow_mut().childs.len();
        let mut cur_scope_index = len;
        for index in 0..len {
          if (*parent_scope_rc).borrow_mut().childs[index].borrow().id == self.cur_scope.borrow().id {
            cur_scope_index = index;
          }
        }
        if cur_scope_index != len {
          (*parent_scope_rc).borrow_mut().childs.remove(cur_scope_index);
        }
        // 回到父级作用域
        self.cur_scope = Rc::clone(parent_scope_rc);
      }
    }

    // 初始化，主要是挂载全局对象
    fn init(&mut self) {
      // 挂载全局对象
      {
        let mut global_scope = self.scope.borrow_mut();
        global_scope.set_value(String::from("globalThis"), Value::RefObject(Rc::downgrade(&self.global)), true);
      }

      for name in GLOBAL_OBJECT_NAME_LIST.iter() {
        let object_type_name = name.to_string();
        let object = get_global_object(self, object_type_name.clone());
        let mut global_scope = self.scope.borrow_mut();
        global_scope.set_value(object_type_name, Value::RefObject(Rc::downgrade(&object)), true);
      }
      let console = create_console(self);
      let mut global_scope = self.scope.borrow_mut();
      global_scope.set_value(String::from("console"), Value::Object(console), true);
    }
    // 获取当前调用栈
    // fn get_current_stack() {

    // }
} 



pub type CallbackFunction = dyn Fn(&mut Context)-> JSIResult<Value>;