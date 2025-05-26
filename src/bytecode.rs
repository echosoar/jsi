// 定义 bytecode operator 列表

#[derive(Debug, Clone)]
pub enum EByteCodeop {
    // 将 undefined 推入栈
    OpUndefined,
    OpNull,
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
}

// ByteCode 结构体定义
#[derive(Debug, Clone)]
pub struct ByteCode {
    // 指令
    pub op: EByteCodeop,
    // 操作数
    pub arg: Option<String>,
    // 行号
    pub line: usize,
}