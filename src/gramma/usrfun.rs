use std::rc::Rc;
use std::cell::RefCell;
use crate::gramma::environment::Environment;
use crate::gramma::ast::{ Callable, ASTNode, ASTValue };
use crate::gramma::evaluator::evaluate_node;

//用户在程序执行时自定义的函数
pub struct UsrDefFun {
    //函数名(如果是匿名函数则没有名称)
    pub name: Option<String>,
    //形参
    pub params: Vec<String>,
    //函数体
    //用Rc而非Box的原因是实际存储他的容器也是用的Rc
    pub body: Rc<ASTNode>,
}

impl Callable for UsrDefFun {
    fn name(&self) -> Option<&str> {
        match &self.name {
            Some(x) => Some(x),
            None => None,
        }
    }

    fn call(&self, args: &[ASTValue], env: Rc<RefCell<Environment>>) -> Result<Option<ASTValue>, String> {
        //校验实参和形参数量是否一致
        if self.params.len() != args.len() {
            raise!("wrong number of arguments")
        }

        //创建函数执行时所在的新环境
        let sub_env = Rc::new(RefCell::new(Environment::new(Some(env))));
        for (param, arg) in self.params.iter().zip(args) {
            sub_env.borrow_mut().regist(param, arg.clone());
        }

        //执行函数
        evaluate_node(&self.body.clone(), sub_env.clone())
    }
}