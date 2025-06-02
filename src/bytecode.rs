// 定义 bytecode operator 列表

use core::fmt;

#[derive(Debug, Clone)]
pub enum EByteCodeop {
    // 将 undefined 推入栈
    OpUndefined,
    OpNull,
    OpTrue,
    OpFalse,
    OpString,
    OpNumber,
    // 从栈中弹出 n 个值初始化数组，然后把数组推入栈
    OpArrayFrom,
    // 向当前作用于推入一个变量，并弹出一个值作为变量的初始值
    OpScopePutVarInit,
    OpScopePutVar,
    OpScopeGetVar,
    OpPushConst,
    // 从栈中弹出 2 值，进行运算后的值推入栈
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    // function xxx 函数定义，匿名函数的名称为 `""`
    OpFuncStart,
    OpFuncEnd,
    // 获取函数参数，弹出一个值作为参数名，推入栈
    OpGetArg,
    // 将函数入栈，传入的是函数名，如果是匿名函数, 函数名是 `""`
    OpGetFunc,
    // 函数调用，弹出 n 个值作为参数、弹出一个值作为 function，进行执行，结果入栈
    OpCall,
}

impl fmt::Display for EByteCodeop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EByteCodeop::OpUndefined => write!(f, "Undefined"),
            EByteCodeop::OpNull => write!(f, "Null"),
            EByteCodeop::OpTrue => write!(f, "True"),
            EByteCodeop::OpFalse => write!(f, "False"),
            EByteCodeop::OpString => write!(f, "String"),
            EByteCodeop::OpNumber => write!(f, "Number"),
            EByteCodeop::OpArrayFrom => write!(f, "ArrayFrom"),
            EByteCodeop::OpScopePutVarInit => write!(f, "ScopePutVarInit"),
            EByteCodeop::OpScopePutVar => write!(f, "ScopePutVar"),
            EByteCodeop::OpScopeGetVar => write!(f, "ScopeGetVar"),
            EByteCodeop::OpPushConst => write!(f, "PushConst"),
            EByteCodeop::OpAdd => write!(f, "Add"),
            EByteCodeop::OpSub => write!(f, "Sub"),
            EByteCodeop::OpMul => write!(f, "Mul"),
            EByteCodeop::OpDiv => write!(f, "Div"),
            EByteCodeop::OpCall => write!(f, "Call"),
            _ => write!(f, "Unknown"),
        }
    }
}

// ByteCode 结构体定义
#[derive(Debug, Clone)]
pub struct ByteCode {
    // 指令
    pub op: EByteCodeop,
    // 操作数
    pub args: Vec<String>,
    // 行号
    pub line: usize,
}