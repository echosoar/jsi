use std::{rc::{Rc, Weak}, cell::RefCell};

use crate::{ast::Program, ast_node::{Statement, Declaration, ObjectLiteral, AssignExpression, CallContext, ArrayLiteral, ClassType, ForStatement, VariableFlag, PostfixUnaryExpression, IdentifierLiteral, PrefixUnaryExpression, SwitchStatement}, ast_node::{Expression, CallExpression, Keywords, BinaryExpression, NewExpression}, value::{Value, ValueInfo, CallStatementOptions}, scope::{Scope, get_value_and_scope}, ast_token::Token, builtins::{object::{Object, Property, create_object}, function::{create_function, get_function_this}, global::{new_global_this, get_global_object, get_global_object_by_name, IS_GLOABL_OBJECT}, array::create_array}, error::{JSIResult, JSIError, JSIErrorType}, constants::{GLOBAL_ERROR_NAME, GLOBAL_BOOLEAN_NAME, GLOBAL_STRING_NAME, GLOBAL_NUMBER_NAME, GLOBAL_OBJECT_NAME_LIST}};


use super::ast::AST;
pub struct Context {
  scope: Rc<RefCell<Scope>>,
  global: Rc<RefCell<Object>>,
  cur_scope: Rc<RefCell<Scope>>
}

impl Context {
    pub fn new() -> Context {
      let scope = Rc::new(RefCell::new(Scope::new()));
      let cur_scope = Rc::clone(&scope);
      let global = new_global_this();
      let mut ctx = Context {
        global,
        scope,
        cur_scope,
      };
      ctx.init();
      return ctx;
    }
    
    // 运行一段 JS 代码
    pub fn run(&mut self, code: String) -> JSIResult<Value> {
      let program = self.parse(code)?;
      // print!("program: {:?}", program);
      self.call(program)
    }

    // 运行一段 JS 代码
    pub fn parse(&mut self, code: String) -> JSIResult<Program> {
      let mut ast = AST::new(code);
      ast.parse()
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
              let function = create_function(&self.global, &function_statement, Rc::downgrade(&self.cur_scope));
             (*self.cur_scope).borrow_mut().set_value(function_statement.name.literal.clone(), function)
            }
        };
      }
      // 函数声明需要添加 prototype、constrctor、__proto__
      // 绑定变量声明
      // 执行 statement
      let mut result_value = Value::Undefined;
      let mut last_statement_value = Value::Undefined;
      // 中断，类似于 break 、continue 等
      let mut interrupt = Value::Undefined;
      for statement in body.iter() {
        let call_options = CallStatementOptions {
          label: None,
        };
        self.call_statement(statement, &mut result_value, &mut last_statement_value, &mut interrupt, call_options)?;
        if interrupt != Value::Undefined {
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
              (*self.cur_scope).borrow_mut().set_value(name, value);
            }
          }
          Ok(true)
        },
        Statement::Expression(expression) => {
          (*last_statement_value) = self.execute_expression(&expression.expression)?;
          Ok(true)
        },
        Statement::Return(return_statement) => {
          (*result_value) = self.execute_expression(&return_statement.expression)?;
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
          if condition.to_boolean(&self.global) {
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
            (*last_statement_value) = value.1.clone();
            (*interrupt) = value.2.clone();
          } else if let Err(err) = &result {
            if let Some(catch) = &try_statement.catch {
              self.switch_scope(Some(Rc::clone(&self.cur_scope)));
              if let Some(error_decl) =&catch.declaration {
                (*self.cur_scope).borrow_mut().set_value(error_decl.literal.clone(), Value::Object(err.to_error_object(&self.global)));
              }
              let result = self.call_block(&vec![], &catch.body.statements)?;
              (*interrupt) = result.2;
              (*last_statement_value) = result.1;
              self.close_scope();
            }
          }
          // TODO: finaly
          
          Ok(true)
        },
        Statement::Block(block) => {
          self.switch_scope(Some(Rc::clone(&self.cur_scope)));
          let result = self.call_block(&vec![], &block.statements)?;
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
          println!("unknown statement {:?}", statement);
          Ok(true)
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
        Expression::Binary(binary) => {
          Ok(ValueInfo { value: self.execute_binary_expression(binary)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::PrefixUnary(expr) => {
          Ok(ValueInfo { value: self.execute_prefix_unary_expression(expr)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::PostfixUnary(expr) => {
          Ok(ValueInfo { value: self.execute_postfix_unary_expression(expr)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::Call(call) => {
          Ok(ValueInfo { value: self.execute_call_expression(call)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::Object(object) => {
          Ok(ValueInfo { value: self.new_object(object), name: None, access_path: String::from(""), reference: None })
        },
        Expression::Array(array) => {
          Ok(ValueInfo { value: self.new_array(array), name: None, access_path: String::from(""), reference: None })
        },
        Expression::Function(function_declaration) => {
          let func = create_function(&self.global, function_declaration, Rc::downgrade(&self.cur_scope));
          Ok(ValueInfo { value: func, name: None, access_path: String::from(""), reference: None })
        },
        Expression::PropertyAccess(property_access) => {
          // expression.name
          let left = self.execute_expression(&property_access.expression)?;
          if left == Value::Null {
            return Err(JSIError::new( JSIErrorType::TypeError, format!("Cannot read properties of null (reading '{}')", property_access.name.literal), 0, 0))
          }
          let left_obj = left.to_object(&self.global);
          let right = &property_access.name.literal;
          let value = (*left_obj).borrow().get_value(right.clone());
          // println!("PropertyAccess: {:?} {:?}",left_obj, right);
          Ok(ValueInfo { value, name: Some(right.clone()), access_path: String::from(""), reference: Some(Value::Object(left_obj)) })
        },
        Expression::ComputedPropertyName(property_name) => {
          Ok(ValueInfo { value: self.execute_expression(&property_name.expression)?, name: None, access_path: String::from(""), reference: None })
        },
        Expression::ElementAccess(element_access) => {
          // expression[argument]
          let left = self.execute_expression(&element_access.expression)?;
          
          let left_obj = left.to_object(&self.global);
          let right = self.execute_expression(&element_access.argument)?.to_string(&self.global);
          if left == Value::Null {
            return Err(JSIError::new( JSIErrorType::TypeError, format!("Cannot read properties of null (reading '{}')", right), 0, 0))
          }
          let value = (*left_obj).borrow().get_value(right.clone());
          Ok(ValueInfo { value, name: Some(right.clone()),  access_path: String::from(""),reference: Some(Value::Object(left_obj)) })
        },
        Expression::Identifier(identifier) => {
          let name = identifier.literal.clone();
          let (value, scope) = get_value_and_scope(Rc::clone(&self.cur_scope), name.clone());
          if let Some(val) = value {
            Ok(ValueInfo{ value: val, name: Some(name.clone()),  access_path: name.clone(),reference: Some(Value::Scope(Rc::downgrade(&scope))) })
          } else {
            Err(JSIError::new(JSIErrorType::ReferenceError, format!("{} is not defined", name), 0, 0))
          }
        },
        Expression::Assign(assign) => {
          Ok(ValueInfo{ value: self.execute_assign_expression(assign), name: None, access_path: String::from(""), reference: None })
        },
        Expression::String(string) => {
          Ok(ValueInfo {value: Value::String(string.value.clone()), name: None, access_path:string.value.clone(), reference: None })
        },
        Expression::Number(number) => {
          Ok(ValueInfo {value: Value::Number(number.value.clone()), name: None, access_path: number.literal.clone(), reference: None })
        },
        Expression::New(new_object) => {
          Ok(ValueInfo { value: self.execute_new_expression(new_object)?, name: None, access_path: String::from(""),reference: None })
        },
        Expression::Keyword(keyword) => {
          Ok(ValueInfo {
            value: match *keyword {
              Keywords::False => Value::Boolean(false),
              Keywords::True => Value::Boolean(true),
              Keywords::Null => Value::Null,
              _ => Value::Undefined,
            },
            name: None,
            reference: None,
            access_path: keyword.to_string(),
          })
        },
        _ => {
          println!("expression: {:?}", expression);
          Ok(ValueInfo {
            value: Value::Undefined,
            name: None,
            reference: None,
            access_path: String::from(""),
          })
        },
      }
    }

    // 执行基础四则运算
    fn execute_binary_expression(&mut self, expression: &BinaryExpression) -> JSIResult<Value> {
      let left = self.execute_expression(expression.left.as_ref())?;

      
      // 逻辑运算 左值
      if expression.operator == Token::LogicalAnd {
        // false &&
        if !left.to_boolean(&self.global) {
          return Ok(Value::Boolean(false));
        }
      } else if expression.operator == Token::LogicalOr {
        // true ||
        if left.to_boolean(&self.global) {
          return Ok(Value::Boolean(true));
        }
      }

      let right = self.execute_expression(expression.right.as_ref())?;

      // 逻辑运算 右值
      if expression.operator == Token::LogicalAnd || expression.operator == Token::LogicalOr {
        // true && false / false || false
        if !right.to_boolean(&self.global) {
          return Ok(Value::Boolean(false));
        } else {
          // true && true / false || true
          return Ok(Value::Boolean(true));
        }
      }

      match expression.operator {
        Token::Equal => {
          return Ok(Value::Boolean(left.is_equal_to(&self.global, &right, false)));
        },
        Token::StrictEqual => {
          return Ok(Value::Boolean(left.is_equal_to(&self.global, &right, true)));
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
              return Ok(Value::String(left.to_string(&self.global) + right.to_string(&self.global).as_str()));
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
        Token::Less | Token::Greater => {
           // TODO: JSIResult
          Ok(self.execute_compare_operator_expression(&left, &right, &expression.operator))
        },
        _ =>  {
          println!("unsupport binary {:?}", expression);
          Ok(Value::Undefined)
        }
      }

    }

    // 执行方法调用表达式
    fn execute_number_operator_expression(&mut self, left: &Value, right: &Value, operator: &Token) -> Value {
      let left_number: f64;
      let right_number: f64;
      if let Some(num) = left.to_number(&self.global) {
        left_number = num;
      } else {
        return Value::NAN;
      }
      if let Some(num) = right.to_number(&self.global) {
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
    fn execute_compare_operator_expression(&mut self, left: &Value, right: &Value, operator: &Token) -> Value {
      let left_number: f64;
      let right_number: f64;
      if let Some(num) = left.to_number(&self.global) {
        left_number = num;
      } else {
        return Value::NAN;
      }
      if let Some(num) = right.to_number(&self.global) {
        right_number = num;
      } else {
        return Value::NAN;
      }
      match operator {
        Token::Greater => Value::Boolean(left_number > right_number),
        Token::Less => Value::Boolean(left_number < right_number),
        _=> Value::NAN,
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
          return self.call_function_object(function_object.to_owned(), callee.reference, reference, arguments);
        },
        Value::RefObject(obj_ref) => {
          let obj = obj_ref.upgrade();
          if let Some(obj_rc) = obj {
            let is_global_object = {
              let function_mut = obj_rc.borrow_mut();
              function_mut.get_inner_property_value(IS_GLOABL_OBJECT.to_string())
            };
            if let Some(_) = is_global_object {
              return Ok(callee.value.instantiate_object(&self.global, arguments).unwrap());
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
    fn execute_assign_expression(&mut self, expression: &AssignExpression) -> Value {
      let mut left_info = self.execute_expression_info(&expression.left).unwrap();
      let right_value = self.execute_expression(&expression.right).unwrap();
      // TODO: more operator
      if expression.operator == Token::Assign {
        left_info.set_value(right_value.clone());
        right_value
      } else {
        Value::Undefined
      }
    }

    // 执行 ++i --i
    fn execute_prefix_unary_expression(&mut self, expression: &PrefixUnaryExpression) -> JSIResult<Value> {
      let mut operand_info = self.execute_expression_info(&expression.operand)?;
      let value_number = operand_info.value.to_number(&self.global);
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
      operand_info.set_value(value.clone());
      Ok(value)
    }

    // 执行 i++ i--
    fn execute_postfix_unary_expression(&mut self, expression: &PostfixUnaryExpression) -> JSIResult<Value> {
      let mut operand_info = self.execute_expression_info(&expression.operand)?;
      let origin_value = operand_info.value.clone();
      let value_number = origin_value.to_number(&self.global);
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
      operand_info.set_value(value);
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
        } else if var.flag == VariableFlag::Let {
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
            if !value.to_boolean(&self.global) {
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
            if !value.to_boolean(&self.global) {
              break;
            }
          }
        }
      }
      self.close_scope();
      Ok(true)
    }


    // 执行循环
    fn execute_switch(&mut self, switch_statment: &SwitchStatement, result_value: &mut Value, last_statement_value: &mut Value, interrupt: &mut Value, call_options: CallStatementOptions) -> JSIResult<bool> {
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
          if case_value.is_equal_to(&self.global, &value, true) {
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
      let obj = constructor.value.instantiate_object(&self.global, arguments);
      if let Some(obj) = obj {
        Ok(obj)
      } else {
        Err(JSIError::new(JSIErrorType::TypeError, format!("{} is not a constructor", constructor.access_path), 0, 0))
      }
    }

    fn new_object(&mut self, expression: &ObjectLiteral) -> Value {
      // 获取 object 实例
      let object = create_object(&self.global,ClassType::Array, None);
      let object_clone = Rc::clone(&object);
      let mut object_mut = (*object_clone).borrow_mut();
      // TODO:
      // object.prototype = global.object_prototype;
      // 绑定属性
      for property_index in 0..expression.properties.len() {
        let property = &expression.properties[property_index];
        let name = self.execute_expression(&property.name).unwrap().to_string(&self.global);
        let mut initializer = self.execute_expression(&property.initializer).unwrap();
        initializer.bind_name(name.clone());
        object_mut.define_property(name, Property {
          enumerable: true,
          value: initializer,
        });
      }
      Value::Object(object)
    }

    fn new_array(&mut self, expression: &ArrayLiteral) -> Value {
      let array = create_array(&self.global, 0);
      if let Value::Array(arr_obj) = &array {
        let mut arguments: Vec<Value> = vec![];
        for element in &expression.elements {
          arguments.push(self.execute_expression(element).unwrap());
        }
        let weak = Rc::downgrade(arr_obj);
        let call_ctx = &mut CallContext {
          global: Rc::downgrade(&self.global),
          this: weak,
          reference: None,
        };
        Object::call(call_ctx, String::from("push"), arguments);
      }
      array
    }

    fn call_function_object(&mut self, function_define: Rc<RefCell<Object>>, call_this: Option<Value>, reference: Option<Weak<RefCell<Object>>>, arguments: Vec<Value>) -> JSIResult<Value> {
      // 获取 function 定义
      let function_define_value = (*function_define).borrow_mut().get_initializer().unwrap();
      // 获取 function 调用的 this
      let mut this_obj = Rc::clone(&function_define);
      if let Some(call_this_value) = call_this {
        if let Value::Object(obj) = call_this_value {
          this_obj = obj;
        } else if let Value::Array(obj) = call_this_value {
          this_obj = obj;
        } else if let Value::Function(func) = call_this_value {
          this_obj = get_function_this(&self.global, func);
        }
      }
      // 内置方法
      if let Statement::BuiltinFunction(builtin_function) = *function_define_value {
        let mut ctx = CallContext{
          global:  Rc::downgrade(&self.global),
          this: Rc::downgrade(&this_obj),
          reference: reference,
        };
        let result = (builtin_function)(&mut ctx, arguments);
        if let Value::FunctionNeedToCall(function_define, args) = result {
          return self.call_function_object(function_define.clone(), Some(Value::Function(function_define)), None, args);
        }
        return Ok(result);
      }

      let function_declaration =  match *function_define_value {
        Statement::Function(function_declaration) => Some(function_declaration),
        _ => None,
      }.unwrap();
      // 创建新的执行作用域
      let define_scope =  (*function_define).borrow_mut().get_inner_property_value(String::from("define_scope"));
      let mut define_scope_value = None;
      if let Some(scope_value) = define_scope {
        if let Value::Scope(scope) = scope_value {
          define_scope_value = scope.upgrade();
        }
      }
      self.switch_scope(define_scope_value);
      // 绑定参数
      for parameter_index in 0..function_declaration.parameters.len() {
        if parameter_index < arguments.len() {
          // TODO: 参数引用
          (*self.cur_scope).borrow_mut().set_value(function_declaration.parameters[parameter_index].name.literal.clone(), arguments[parameter_index].clone());
        } else {
          (*self.cur_scope).borrow_mut().set_value(function_declaration.parameters[parameter_index].name.literal.clone(), Value::Undefined);
        }
      }
      // 执行 body
      let result = self.call_block(&function_declaration.declarations, &function_declaration.body.statements)?;
      self.close_scope();
      Ok(result.0)
    }

    // 切换作用域
    fn switch_scope(&mut self, define_scope: Option<Rc<RefCell<Scope>>>) {
      let mut new_scope = Scope::new();
      new_scope.parent = define_scope;
      if let Some(scope) = &new_scope.parent {
        let rc = Rc::clone(scope);
        let scope = rc.borrow();
        new_scope.labels = scope.labels.clone();
      }
      new_scope.from = Some(Rc::clone(&self.cur_scope));
      let scope_rc = Rc::new(RefCell::new(new_scope));
      let rc = Rc::clone(&scope_rc);
      (*self.cur_scope).borrow_mut().childs.push(scope_rc);
      self.cur_scope = rc;
    }

    // 退出作用域
    fn close_scope(&mut self) {
      if let Some(parent_scope_rc) = &self.cur_scope.borrow().from {
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
      }
    }

    // 初始化，主要是挂载全局对象
    fn init(&mut self) {
      // 挂载全局对象
      let mut global_scope = self.scope.borrow_mut();

      global_scope.set_value(String::from("globalThis"), Value::RefObject(Rc::downgrade(&self.global)));

      for name in GLOBAL_OBJECT_NAME_LIST.iter() {
        let object_type_name = name.to_string();
        let object = get_global_object(&self.global, object_type_name.clone());
        global_scope.set_value(object_type_name, Value::RefObject(Rc::downgrade(&object)));
      }
    }

    // 获取当前调用栈
    fn get_current_stack() {

    }
} 