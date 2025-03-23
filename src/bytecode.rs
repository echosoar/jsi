// 定义 bytecode operator 列表

#[derive(Clone)]
pub enum EByteCode {

}

// ByteCode Info 结构体定义
#[derive(Debug, Clone)]
pub struct ByteCodeInfo {
    // 读取的 bytecode 列表
    pub bytecodes: Vec<ByteCode>,
}

// emit 方法定义
impl ByteCodeInfo {
    pub fn emit(&mut self, op: EByteCode, arg: Option<String>, line: usize) {
        self.bytecodes.push(ByteCode {
            op,
            arg,
            line,
        });
    }
}

// ByteCode 结构体定义
#[derive(Debug, Clone)]
pub struct ByteCode {
    // 指令
    pub op: EByteCode,
    // 操作数
    pub arg: Option<String>,
    // 行号
    pub line: usize,
}