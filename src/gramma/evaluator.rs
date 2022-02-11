use std::rc::Rc;
use std::cell::RefCell;
use crate::gramma::token::Op;
use crate::gramma::environment::Environment;
use crate::gramma::ast::{ASTNode, ASTValue};
use crate::gramma::usrfun::UsrDefFun;

//对语法树节点求值
pub fn evaluate_node(root: &ASTNode, env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String> {
    match root {
        //空节点
        ASTNode::Empty => Ok(None),
        //无返回值节点(将返回值设置为None 实现忽略)
        ASTNode::Void(node) => evaluate_node(node, env).map(|_| None),
        //字面量节点
        ASTNode::Literal(val) => Ok(Some(val.clone())),
        //数值/函数变量
        ASTNode::Var(name) => {
            if let Some(value) = env.borrow().get(name, false) {
                Ok(Some(value))
            } else {
                raise!("variable not define")
            }
        },
        //数组元素索引
        ASTNode::Index(arr, index) => evaluate_index(&evaluate_node(arr, env.clone())?, evaluate_node(index, env.clone())?, env),
        //数组节点
        ASTNode::Array(elements) => {
            let mut results = vec![];
            for element in elements {
                if let Some(result) = evaluate_node(element, env.clone())? {
                    results.push(result);
                }
            }
            Ok(Some(ASTValue::Array(results.into())))
        }
        //单目运算表达式
        ASTNode::Unitary(op, node) => {
            if let Some(arg) = evaluate_node(node, env)? {
                evaluate_unitary_op(op.clone(), arg)
            } else {
                raise!("Error evaluate unitary op")
            }
        },
        //双目运算节点
        ASTNode::Binary(op, lhs, rhs) => {
            if let (Some(lvalue), Some(rvalue)) = (evaluate_node(lhs, env.clone())?, evaluate_node(rhs, env)?) {
                evaluate_binary_op(op.clone(), lvalue, rvalue)
            } else {
                raise!("Error evaluate binary op")
            }
        },
        //定义(true)赋值(false)数值/函数变量节点
        ASTNode::Assign(name, body, define) => evaluate_assign(name, body, define.clone(), env),
        //匿名函数节点
        ASTNode::Lambda(args, body) => evaluate_lambda(args, body, env),
        //条件表达式节点
        ASTNode::Cond(if_node, elseif_nodes, else_node) => evaluate_cond(if_node, elseif_nodes, else_node, env),
        //调用节点
        ASTNode::Apply(fun, args) => evaluate_apply(fun, args, env),
        //语句块节点
        ASTNode::Block(nodes) => evaluate_block(nodes, env),
    }
}

//单目运算节点求值
fn evaluate_unitary_op(op: Op, arg: ASTValue) -> Result<Option<ASTValue>, String> {
    match (op, arg) {
        (Op::Add, ASTValue::Number(x)) => Ok(Some(ASTValue::Number(x))),
        (Op::Sub, ASTValue::Number(x)) => Ok(Some(ASTValue::Number(-1.0 * x))),
        (Op::Not, ASTValue::Boolean(x)) => Ok(Some(ASTValue::Boolean(!x))),
        _ => raise!("Error evaluate unitary op"),
    }
}

//双目运算节点求值
fn evaluate_binary_op(op: Op, lhs: ASTValue, rhs: ASTValue) -> Result<Option<ASTValue>, String> {
    match (op, lhs, rhs) {
        (Op::Add, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Number(x + y))),
        (Op::Sub, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Number(x - y))),
        (Op::Mul, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Number(x * y))),
        (Op::Div, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Number(x / y))),
        (Op::Pow, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Number(x.powf(y)))),
        (Op::Mod, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Number(x % y))),
        (Op::Eq, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Boolean(x == y))),
        (Op::Neq, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Boolean(x != y))),
        (Op::Lt, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Boolean(x < y))),
        (Op::Gt, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Boolean(x > y))),
        (Op::Lte, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Boolean(x <= y))),
        (Op::Gte, ASTValue::Number(x), ASTValue::Number(y)) => Ok(Some(ASTValue::Boolean(x >= y))),
        (Op::And, ASTValue::Boolean(x), ASTValue::Boolean(y)) => Ok(Some(ASTValue::Boolean(x && y))),
        (Op::Or, ASTValue::Boolean(x), ASTValue::Boolean(y)) => Ok(Some(ASTValue::Boolean(x || y))),
        _ => raise!("Error evaluate binary op"),
    }
}

#[allow(unused)]
fn evaluate_index(arr_node: &Option<ASTValue>, index_node: Option<ASTValue>, env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String> {
    match (arr_node, index_node) {
        //arr[i], arr = [1, 2, 3, 4]形式
        (Some(ASTValue::Array(arr)), Some(ASTValue::Number(index))) => {
            Ok(Some(match arr.get(index.round() as usize) {
                Some(v) => v.clone(),
                _ => raise!("Error index out of bound"),
            }))
        }
        //arr[i, j, ...], arr = [[1, 2, 3], [4, 5, 6], ...]形式
        (arr_node, Some(ASTValue::Array(indices))) => {
            let mut results = vec![];
            for index in indices.iter() {
                if let Some(result) = evaluate_index(arr_node, Some(index.clone()), env.clone())? {
                    results.push(result)
                }
            }
            Ok(Some(ASTValue::Array(results.into())))
        },
        _ => raise!("Error evaluate array index"),
    }
}

//赋值表达式求值
fn evaluate_assign(name: &str, body: &Box<ASTNode>, define: bool, env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String> {
    if define && env.borrow().get(name, true).is_some() {
        raise!("redefine variable")
    } else if !define && env.borrow().get(name, false).is_none() {
        raise!("undefine variable")
    }
    if let Some(value) = evaluate_node(body, env.clone())? {
        if define {
            env.borrow_mut().regist(name, value.clone());
        } else {
            env.borrow_mut().set(name, value.clone());
        }
        Ok(Some(value))
    } else {
        raise!("can not assign variable with ()")
    }
}

//条件表达式求值
fn evaluate_cond(if_node: &Box<(ASTNode, ASTNode)>, 
                          elseif_nodes: &Vec<(ASTNode, ASTNode)>, 
                          else_node: &Option<Box<ASTNode>>, 
                          env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String> {
    if let Some(flag1) = evaluate_node(&if_node.0, env.clone())? {
        //分支
        if flag1.boolean()? {
            evaluate_node(&if_node.1, env.clone())
        } else {
            //else if 分支
            for (cond, branch) in elseif_nodes {
                if let Some(flag2) = evaluate_node(cond, env.clone())? {
                    if flag2.boolean()? {
                        //命中某个elseif分支直接返回
                        return evaluate_node(branch, env)
                    } 
                }
            }
            //else 分支
            if else_node.is_some() {
                return evaluate_node(else_node.as_ref().unwrap(), env)
            }
            //没有else分支 或者 elseif所有分支都没有命中
            Ok(None)
        }
    } else {
        raise!("evaluate condition failed")
    }
}

//匿名函数求值 (目前这个函数的实现存在问题 高阶lambda调用存在问题 以后会改进)
#[allow(unused)]
fn evaluate_lambda(args: &Vec<String>, body: &Box<ASTNode>, env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String> {
    /*Ok(Some(ASTValue::Function(Rc::new(UsrDefFun {
        name: None,
        params: args.to_owned(),
        body: Rc::new(*body.clone()),
    }))))*/

    Ok(Some(ASTValue::Function(Rc::new(UsrDefFun {
        name: None,
        params: args.to_owned(),
        body: Rc::new(capture_outside_variable(body, args, env)?),
    }))))
}

//调用节点求值
//支持[(x) => {x + 2}, (x) => {x^2 + 2}](2)形式的调用
fn evaluate_apply(fun_node: &ASTNode, arg_nodes: &Vec<ASTNode>, env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String> {
    match evaluate_node(fun_node, env.clone())? {
        //单一函数
        Some(ASTValue::Function(fun)) => {
            //计算各个实参
            let mut args = vec![];
            for arg_node in arg_nodes {
                if let Some(result) = evaluate_node(arg_node, env.clone())? {
                    args.push(result);
                } else {
                    //语法错误
                    raise!("undefine behavior")
                }
            }
            fun.call(&args, env)
        },
        //多个函数放在一个队列中
        //这边的逻辑需要优化
        Some(ASTValue::Array(fun_nodes)) => {
            let mut results = vec![];
            for fun_node in fun_nodes.iter() {
                if let ASTValue::Function(ref fun) = fun_node {
                    //计算各个实参
                    let mut args = vec![];
                    for arg_node in arg_nodes {
                        if let Some(result) = evaluate_node(arg_node, env.clone())? {
                            args.push(result);
                        } else {
                            //语法错误
                            raise!("undefine behavior")
                        }
                    }
                    if let Some(result) = fun.call(&args, env.clone())? {
                        results.push(result)
                    }
                }
            }
            Ok(Some(ASTValue::Array(results.into())))
        }
        _ => raise!("evaluate apply failed"),
    }
}

fn evaluate_block(nodes: &Vec<ASTNode>, env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String> {
    //创建子环境
    let sub_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
    for node in nodes {
        //遇到有值返回的则直接返回
        if let Some(result) = evaluate_node(node, sub_env.clone())? {
            return Ok(Some(result))
        }
    }
    Ok(None)
}

//此函数的应用于以下场景
//let f = (z) => {(x, y) => (x + y + z)}
//当执行f(3)时返回的lambda中的z应该替换为z
//这种机制的存在意味着lambda表达式中包含的外部变量不能出现在赋值符号左侧
//针对上面的案例 这意味着lambda的实体内不能出现z = xxx或者let z = xxx的语句
fn capture_outside_variable(lambda: &ASTNode, bound: &[String], env: Rc<RefCell<Environment>>) -> Result<ASTNode, String> {
    Ok(match lambda {
        //空节点
        ASTNode::Empty => ASTNode::Empty,
        //无返回值节点(将返回值设置为None 实现忽略)
        ASTNode::Void(node) => ASTNode::Void(Box::new(capture_outside_variable(node, bound, env)?)),
        //字面量节点
        ASTNode::Literal(val) => ASTNode::Literal(val.clone()),
        //数值/函数变量
        ASTNode::Var(name) => {
            if let Some(val) = env.borrow().get(name, false) {
                ASTNode::Literal(val.clone())
            } else {
                //有些环境表找不到的变量是生命在lambda内部的 运行的时候再做判断即可
                ASTNode::Var(name.clone())
            }
        },
        //数组元素索引
        ASTNode::Index(arr, index) => {
            let new_arr = Box::new(capture_outside_variable(arr, bound, env.clone())?);
            let new_index = Box::new(capture_outside_variable(index, bound, env.clone())?);
            ASTNode::Index(new_arr, new_index)
        }
        //数组节点
        ASTNode::Array(elements) => {
            let mut new_elements = vec![];
            for element in elements {
                new_elements.push(capture_outside_variable(element, bound, env.clone())?)
            }
            ASTNode::Array(new_elements)
        }
        //单目运算表达式
        ASTNode::Unitary(op, node) => ASTNode::Unitary(*op, Box::new(capture_outside_variable(node, bound, env)?)),
        //双目运算节点
        ASTNode::Binary(op, lhs, rhs) => {
            let new_lhs = Box::new(capture_outside_variable(lhs, bound, env.clone())?);
            let new_rhs = Box::new(capture_outside_variable(rhs, bound, env)?);
            ASTNode::Binary(*op, new_lhs, new_rhs)
        },
        //定义(true)赋值(false)数值/函数变量节点
        ASTNode::Assign(name, body, define) => {
            if env.borrow().get(name, false).is_none() {
                ASTNode::Assign(name.clone(), body.clone(), *define)
            } else {
                raise!("capture variable are const, can't re-assign/re-definition")
            }
        },
        //匿名函数节点
        ASTNode::Lambda(args, body) => {
            let mut new_bound = vec![];
            for arg in args {
                new_bound.push(arg.clone());
            }
            for arg in bound {
                new_bound.push(arg.clone());
            }
            ASTNode::Lambda(args.clone(), Box::new(capture_outside_variable(body, &new_bound[..], env.clone())?))
        }
        //条件表达式节点
        ASTNode::Cond(if_node, elseif_nodes, else_node) => {
            let new_if_node = Box::new((capture_outside_variable(&if_node.0, bound, env.clone())?, capture_outside_variable(&if_node.1, bound, env.clone())?));
            let mut new_elseif_nodes = vec![];
            for elseif_node in elseif_nodes {
                let new_elseif_node = (capture_outside_variable(&elseif_node.0, bound, env.clone())?, capture_outside_variable(&elseif_node.1, bound, env.clone())?);
                new_elseif_nodes.push(new_elseif_node);
            }
            let new_else_node = if else_node.is_some() {
                Some(Box::new(capture_outside_variable(&else_node.as_ref().unwrap(), bound, env)?))
            } else {
                None
            };
            ASTNode::Cond(new_if_node, new_elseif_nodes, new_else_node)
        },
        //调用节点
        ASTNode::Apply(fun, args) => {
            //由于存在递归调用的可能 只能把函数体本身的外部绑定这种行为给禁止掉
            //这个问题还是有希望解决的 只要在函数定义部分绑定函数时bound多加上函数名字本身即可以后再解决
            //let new_fun = Box::new(capture_outside_variable(fun, bound, env.clone())?);
            let mut new_args = vec![];
            for arg in args {
                new_args.push(capture_outside_variable(arg, bound, env.clone())?)
            }
            ASTNode::Apply(fun.clone(), new_args)
        },
        //语句块节点
        ASTNode::Block(nodes) => {
            let mut new_nodes = vec![];
            for node in nodes {
                new_nodes.push(capture_outside_variable(node, bound, env.clone())?)
            }
            ASTNode::Block(new_nodes)
        }
    })
}