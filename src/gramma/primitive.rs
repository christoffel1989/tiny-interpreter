use std::rc::Rc;
use std::cell::RefCell;
use crate::gramma::environment::Environment;
use crate::gramma::ast::{ASTValue, Callable};

//系统函数(非用户定义)
struct PrimitiveFun<F>(String, F);
impl<F> Callable for PrimitiveFun<F>
where
    F: Fn(&[ASTValue], Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String>,
{
    fn name(&self) -> Option<&str> {
        Some(&self.0)
    }
    fn call(&self, args: &[ASTValue], env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String> {
        self.1(args, env)
    }
}

//创建解释器全局环境
use std::f64::{self, consts};
pub fn create_global_environment() -> Rc<RefCell<Environment>> {
    let env = Rc::new(RefCell::new(Environment::new(None)));

    //常量
    regist_const(env.clone(), "pi", consts::PI);
    regist_const(env.clone(), "e", consts::E);
    regist_const(env.clone(), "nan", f64::NAN);
    regist_const(env.clone(), "inf", f64::INFINITY);

    //基础一元数值函数
    regist_unitary_fun(env.clone(), "abs", |x| x.abs());
    regist_unitary_fun(env.clone(), "sqrt", |x| x.sqrt());
    regist_unitary_fun(env.clone(), "ln", |x| x.ln());
    regist_unitary_fun(env.clone(), "log2", |x| x.log2());
    regist_unitary_fun(env.clone(), "log10", |x| x.log10());
    regist_unitary_fun(env.clone(), "round", |x| x.round());
    regist_unitary_fun(env.clone(), "floor", |x| x.floor());
    regist_unitary_fun(env.clone(), "sin", |x| x.sin());
    regist_unitary_fun(env.clone(), "cos", |x| x.cos());
    regist_unitary_fun(env.clone(), "tan", |x| x.tan());
    regist_unitary_fun(env.clone(), "asin", |x| x.asin());
    regist_unitary_fun(env.clone(), "acos", |x| x.acos());
    regist_unitary_fun(env.clone(), "atan", |x| x.atan());

    //基础二元数值函数
    regist_binary_fun(env.clone(), "log", |x, y| x.log(y));
    regist_binary_fun(env.clone(), "atan2", |x, y| x.atan2(y));

    //注册一些有用的广义函数
    regist_genneral_fun(env.clone());

    env
}

//注册一些有用的广义函数
fn regist_genneral_fun(env: Rc<RefCell<Environment>>) {
    //数组映射函数
    //类似于map([1, 2, 3, 4], (x) => { 2 * x}) = [2, 4, 6, 8]
    regist_primivitive_fun(env.clone(), "map", |args, env| {
        let args = check_args_num(args, 2)?;
        match (&args[0], &args[1]) {
            (ASTValue::Array(elements), ASTValue::Function(fun)) => {
                let mut results = vec![];
                for element in elements.iter() {
                    if let Some(result) = fun.call(&[element.clone()], env.clone())? {
                        results.push(result)
                    }
                }
                Ok(Some(ASTValue::Array(results.into())))
            },
            _ => {
                raise!("map error")
            }
        }
    });

    //数组长度
    regist_primivitive_fun(env.clone(), "length", |args, _| {
        let args = check_args_num(args, 1)?;
        if let ASTValue::Array(arr) = &args[0] {
            Ok(Some(ASTValue::Number(arr.len() as f64)))
        } else {
            raise!("not an array")
        }
    });

    //range
    regist_primivitive_fun(env.clone(), "range", |args, _| {
        let args = check_args_num(args, 2)?;
        let i0 = args[0].f64()?.round() as i64;
        let i1 = args[1].f64()?.round() as i64;

        let mut i = 0 as i64;
        let mut arr = vec![];

        while i0 + i < i1 {
            arr.push(ASTValue::Number((i0 + i) as f64));
            i += 1;
        }

        Ok(Some(ASTValue::Array(arr.into())))
    });

    //linespace
    regist_primivitive_fun(env.clone(), "linespace", |args, _| {
        let args = check_args_num(args, 3)?;
        let t0 = args[0].f64()?;
        let t1 = args[1].f64()?;
        let n = args[2].f64()?.floor() as i64;

        if n < 2 {
            raise!("number of steps cannot be less than 2")
        }

        let arr = (0..n)
            .map(|i| (i as f64) / ((n - 1) as f64))
            .map(|v| (1.0 - v) * t0 + v * t1)
            .map(|v| ASTValue::Number(v))
            .collect::<Vec<_>>();

            Ok(Some(ASTValue::Array(arr.into())))
    });
}

//注册常量
fn regist_const(env: Rc<RefCell<Environment>>, key: &str, val: f64) {
    env.borrow_mut().regist(key, ASTValue::Number(val));
}

//注册一元函数
fn regist_unitary_fun<F: 'static>(env: Rc<RefCell<Environment>>, key: &str, fun: F)
where
    F: Fn(f64) -> f64,
{
    regist_primivitive_fun(env, key, move |args: &[ASTValue], _| {
        let args = check_args_num(args, 1)?;
        Ok(Some(ASTValue::Number(fun(args[0].f64()?))))
    });
}

//注册二元函数
fn regist_binary_fun<F: 'static>(env: Rc<RefCell<Environment>>, key: &str, fun: F)
where
    F: Fn(f64, f64) -> f64,
{
    regist_primivitive_fun(env, key, move |args: &[ASTValue], _| {
        let args = check_args_num(args, 2)?;
        Ok(Some(ASTValue::Number(fun(args[0].f64()?, args[1].f64()?))))
    });
}

//校验输入参数的数量
fn check_args_num<'a>(args: &'a [ASTValue], num: usize) -> Result<&'a [ASTValue], String> {
    if args.len() == num {
        Ok(args)
    } else {
        raise!("arguement num wrong")
    }
}

//Rc要求F必须加上'static的声明周期约束
fn regist_primivitive_fun<F: 'static>(env: Rc<RefCell<Environment>>, key: &str, fun: F)
where
    F: Fn(&[ASTValue], Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String>, {
        env.borrow_mut().regist(key, ASTValue::Function(Rc::new(PrimitiveFun(key.to_string(), fun))));
}

