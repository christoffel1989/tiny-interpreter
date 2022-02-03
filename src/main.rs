mod gramma;
use gramma::lexer::Lexer;
use gramma::ast::ASTValue;
use gramma::parser::parse_statement;
use gramma::primitive::create_global_environment;
use gramma::evaluator::evaluate_node;

use std::io;
use std::io::prelude::*;

fn main() -> io::Result<()> {
    //创建父环境
    let env = create_global_environment();

    loop {
        io::stdout().write_all(b">>> ")?;
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        //去除input中存在的换行符
        input = String::from(input.strip_suffix("\r\n").or(input.strip_suffix("\n")).unwrap_or(&input));
        let mut lexer = Lexer::new(&input);

        match parse_statement(& mut lexer) {
            Ok(root) => {
                //println!("{:?}", &root);
                match evaluate_node(&root, env.clone()) {
                    Ok(Some(result)) => {
                        println!("ans = {}", format_value(&result));
                    },
                    Err(msg) => {
                        println!("evaluate error: {}", msg);
                    },
                    _ => (),
                }
            }
            Err(_) => {
                println!("parse error");
            }
        }
    }
}

fn format_value(val: &ASTValue) -> String {
    match val {
        ASTValue::Number(result) => {
            format!("{}", result)
        },
        ASTValue::Boolean(result) => {
            format!("{}", result)
        },
        ASTValue::Function(_) => {
            format!("lambda")
        },
        ASTValue::Array(elements) => {
            let mut result = String::from("[");
            for element in elements.iter() {
                result.push_str(&format_value(element));
                result.push_str(", ")
            }
            result.pop(); result.pop();
            result.push_str("]");
            result
        },
    }
}
