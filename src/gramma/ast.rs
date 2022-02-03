use std::rc::Rc;
use std::cell::RefCell;
use crate::gramma::token::Op;
use crate::gramma::environment::Environment;

//抽象语法树节点
//其实只有在evalue lambda时用到了clone
#[derive(Clone, Debug)]
pub enum ASTNode {
    //字面量
    Literal(ASTValue),
    //变量
    Var(String),
    //一元运算
    Unitary(Op, Box<ASTNode>),
    //二元运算
    Binary(Op, Box<ASTNode>, Box<ASTNode>),
    //数组索引
    Index(Box<ASTNode>, Box<ASTNode>),
    //调用f(x, y), ...
    Apply(Box<ASTNode>, Vec<ASTNode>),
    //数组[1, 2, 3, 4, 5, 6]
    Array(Vec<ASTNode>),
    //匿名函数
    Lambda(Vec<String>, Box<ASTNode>),
    //语句块
    Block(Vec<ASTNode>),
    //条件表达式 If(条件)语句块 大于等于0个elseif(条件)语句块 0或1个else语句块
    Cond(Box<(ASTNode, ASTNode)>, Vec<(ASTNode, ASTNode)>, Option<Box<ASTNode>>),
    //定义(true)赋值(false)数值/函数变量
    Assign(String, Box<ASTNode>, bool),
    //空返回值语句(带了分号)
    Void(Box<ASTNode>),
    //空语句
    Empty,
}

//抽象语法树节点值
#[derive(Clone)]
//AST求值结果
pub enum ASTValue {
    //浮点数变量
    Number(f64),
    //布尔变量
    Boolean(bool),
    //数组类型变量(套一层Rc的原因是[]不定长)
    //不用box的原因是env的get函数会拷贝返回
    Array(Rc<[ASTValue]>),
    //函数对象(套一层Rc的原因是Trait类似于C++基类 无实体)
    Function(Rc<dyn Callable>),
}

//函数对象trait
pub trait Callable {
    fn name(&self) -> Option<&str>;
    fn call(&self, args: &[ASTValue], env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String>;
}

//结果类型转换函数
impl ASTValue {
    pub fn f64(&self) -> Result<f64, String> {
        match self {
            ASTValue::Number(x) => Ok(*x),
            ASTValue::Boolean(true) => Ok(1.0),
            ASTValue::Boolean(false) => Ok(0.0),
            _ => raise!("illegal casting to f64"),
        }
    }

    pub fn boolean(&self) -> Result<bool, String> {
        match self {
            ASTValue::Number(x) => Ok(*x != 0.0),
            ASTValue::Boolean(x) => Ok(*x),
            _ => raise!("illegal casting to boolean"),
        }
    }
}

//因为trait没法derive debug 只要手动实现fmt::Debug
use std::fmt;
impl fmt::Debug for ASTValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ASTValue::Number(value) => write!(f, "{}", value),
            ASTValue::Boolean(value) => write!(f, "{}", value),
            ASTValue::Array(values) => write!(f, "{:?}", values),
            ASTValue::Function(fun) => write!(f, "fn-{}", fun.name().unwrap_or("anonymous")),
        }
    }
}

//用于单元测试assert比较
use std::cmp;
impl cmp::PartialOrd for ASTValue {
    fn partial_cmp(&self, other: &ASTValue) -> Option<cmp::Ordering> {
        match (self, other) {
            (ASTValue::Number(x), ASTValue::Number(y)) => {
                if let Some(x) = x.partial_cmp(y) {
                    Some(x)
                } else {
                    y.is_nan().partial_cmp(&x.is_nan())
                }
            }
            (ASTValue::Boolean(x), ASTValue::Boolean(y)) => x.partial_cmp(y),
            (ASTValue::Array(x), ASTValue::Array(y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}
impl cmp::PartialEq for ASTValue {
    fn eq(&self, other: &ASTValue) -> bool {
        self.partial_cmp(other) == Some(cmp::Ordering::Equal)
    }
}