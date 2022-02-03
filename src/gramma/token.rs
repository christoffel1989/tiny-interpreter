//符号集
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    //变量定义
    Let,
    //条件分支
    If,
    ElseIf,
    Else,
    //小括号
    LeftParen,
    RightParen,
    //中括号
    LeftBracket,
    RightBracket,
    //大括号
    LeftBrace,
    RightBrace,
    //逗号
    Comma,
    //赋值
    Assign,
    //lambda函数定义
    Arrow,
    //冒号
    Colon,
    //分号
    SemiColon,
    //运算符
    Operator(Op),
    //逻辑字面量
    Boolean(bool),
    //数字字面量
    Number(String),
    //值变量或者函数变量的标识符
    Symbol(String),
    //非法符号
    Illegal(char),
    //不存在Token
    End,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
//运算符集
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Not,
    And,
    Or,
}

impl Op {
    //二元算符优先级计算
    pub fn priority(&self) -> i32 {
        match *self {
            Op::Or => 0,
            Op::And => 1,
            Op::Lt | Op::Gt | Op::Lte | Op::Gte => 2,
            Op::Eq | Op::Neq => 3,
            Op::Add | Op::Sub => 4,
            Op::Mul | Op::Div | Op::Pow | Op::Mod => 5,
            _ => -1,
        }
    }
}