use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::gramma::ast::ASTValue;

//环境表
pub struct Environment {
    //变量表
    table: HashMap<String, ASTValue>,
    //父环境
    //子环境生命周期与父环境声明周期不同 且还需要能够对父环境进行修改 生命周期难以控制。。
    //不太懂rust的情况下只能采用Rc<RefCell<>>一把梭。。。
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            table: HashMap::new(),
            parent: parent,
        }
    }

    //current_only为true时只在当前环境寻找变量 不继续往父环境寻找
    pub fn get(&self, key: &str, current_only: bool) -> Option<ASTValue> {
        if let Some(x) = self.table.get(key) {
            Some(x.clone())
        } else if !current_only && self.parent.is_some() {
            self.parent.clone().unwrap().borrow().get(key, false)
        } else {
            None
        }
    }

    //会一直往父环境中搜索
    pub fn set(&mut self, key: &str, val: ASTValue) {
        //先找当前环境
        if self.table.get(key).is_some() {
            self.table.insert(key.into(), val);
        } else if self.parent.is_some() {
            self.parent.clone().unwrap().borrow_mut().set(key, val);
        }
    }

    //在当前环境注册新的变量
    pub fn regist(&mut self, key: &str, val: ASTValue) {
        self.table.insert(key.into(), val);
    }
}