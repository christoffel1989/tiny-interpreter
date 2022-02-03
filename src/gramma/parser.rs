use crate::gramma::token::{Token, Op};
use crate::gramma::lexer::{Lexer, Span};
use crate::gramma::ast::{ASTNode, ASTValue};

//解析错误定位信息
#[derive(Debug)]
pub struct ParseError {
    pub token: Token,
    pub span: Span,
}

//解析完整的一句语句(包括结尾的;)
pub fn parse_statement(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    let node = parse_statment_ignore_end_semi_colon(lexer)?;
    //结尾是否存在分号
    Ok(if lexer.peek() == Token::SemiColon {
        lexer.next();
        ASTNode::Void(Box::new(node))
    } else {
        node
    })
}

//解析完整的一句语句(忽略语句结尾的;)
pub fn parse_statment_ignore_end_semi_colon(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    Ok(match (lexer.next(), lexer.next(), lexer.next()) {
        //(数值/函数)变量定义 格式:let name = expr or lambda
        (Token::Let, Token::Symbol(name), Token::Assign) => {
            ASTNode::Assign(name, Box::new(parse_statment_ignore_end_semi_colon(lexer)?), true)
        },
        //变量赋值
        (Token::Symbol(name), Token::Assign, _) => {
            lexer.prev();
            ASTNode::Assign(name, Box::new(parse_statment_ignore_end_semi_colon(lexer)?), false)
        }
        //语句块
        (Token::LeftBrace, _, _) => {
            lexer.prev(); lexer.prev(); lexer.prev();
            parse_block(lexer)?
        }
        //条件表达式
        (Token::If, _, _) => {
            lexer.prev(); lexer.prev(); lexer.prev();
            parse_cond_expr(lexer)?
        },
        //单独一个分号
        (Token::SemiColon, _, _) => {
            lexer.prev(); lexer.prev();
            ASTNode::Void(Box::new(ASTNode::Empty))
        },
        //表达式
        _ => {
            lexer.prev(); lexer.prev(); lexer.prev();
            parse_expr(lexer)?
        },
    })
}

//解析输入的表达式
pub fn parse_expr(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    //表达式有两种 一种是lambda表达式 一种是包含着一元运算的二元运算表达式
    match (lexer.next(), lexer.next(), lexer.next(), lexer.peek()) {
        //函数表达式
        //() =>
        (Token::LeftParen, Token::RightParen, Token::Arrow, _) |
        //(x) =>  
        (Token::LeftParen, Token::Symbol(_), Token::RightParen, Token::Arrow) |
        //(x, ...
        (Token::LeftParen, Token::Symbol(_), Token::Comma, _) => {
            lexer.prev(); lexer.prev(); lexer.prev();
            parse_lambda(lexer)
        },
        //数值表达式
        _ => {
            lexer.prev(); lexer.prev(); lexer.prev();
            parse_binary_op(lexer, 1)
        },
    }
}

//解析生成双目运算节点
//将原本Express Term Factor多个层级的递归计算通过优先级定义统一成了单一函数
fn parse_binary_op(lexer: &mut Lexer, level: i32) -> Result<ASTNode, ParseError> {
    let mut node = parse_unitary_op(lexer)?;
    loop {
        if let Token::Operator(op) = lexer.peek() {
            //假定level这个优先级的运算符写为+
            //则由level这个优先级的运算符构成的双目表达式为T (+T) (+T) (+T).....
            //其中T包含着更高优先级的运算构成的双目表达式
            if level <= op.priority() {
                lexer.next();
                node = ASTNode::Binary(op, Box::new(node), Box::new(parse_binary_op(lexer, op.priority() + 1)?));
                continue
            }
        }
        break;
    }
    Ok(node)
}

///////////////////////没定义优先级之前的分离式实现方法////////////////////////
/*//解析输入的表达式
pub fn parse_expr(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    parse_binary_op(lexer)
}

//解析生成双目运算节点
fn parse_binary_op(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    parse_add_sub(lexer)
}

//解析生成加减运算节点
fn parse_add_sub(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    let mut node = parse_mul_div_pow_mod(lexer)?;
    loop {
        if let Token::Operator(op) = lexer.peek() {
            if op == Op::Add || op == Op::Sub {
                lexer.next();
                node = ASTNode::Binary(op, Box::new(node), Box::new(parse_mul_div_pow_mod(lexer)?));
                continue
            }
        }
        break;
    }
    Ok(node)
}

//解析生成乘除幂模运算节点
fn parse_mul_div_pow_mod(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    let mut node = parse_unitary_op(lexer)?;
    loop {
        if let Token::Operator(op) = lexer.peek() {
            if op == Op::Mul || op == Op::Div || op == Op::Pow || op == Op::Mod {
                lexer.next();
                node = ASTNode::Binary(op, Box::new(node), Box::new(parse_unitary_op(lexer)?));
                continue
            }
        }
        break;
    }
    Ok(node)
}*/
/////////////////////////////////////////////////////////////////////////////////////

//解析生成单目运算
fn parse_unitary_op(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    if let Token::Operator(op) = lexer.peek() {
        if op == Op::Add || op == Op::Sub || op == Op::Not {
            lexer.next();
            return Ok(ASTNode::Unitary(op, Box::new(parse_unitary_op(lexer)?)))
        }
    }
    parse_minimum_item(lexer)
}

//解析最小表达式单元
//一般形式为A、A(x, ...)、A[i]、或者混合形式A()[]() ....
//其中A的形式包括(expr)、逻辑字面量、数字字面量、变量标识符、数组
fn parse_minimum_item(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    //先解析A
    let mut node = match lexer.next() {
        //逻辑字面量
        Token::Boolean(arg) => ASTNode::Literal(ASTValue::Boolean(arg)),
        //数字字面量
        Token::Number(num) => {
            if let Ok(x) = num.parse() {
                ASTNode::Literal(ASTValue::Number(x))
            } else {
                return unexpected_prev_token(lexer)
            }
        },
        //变量标识符
        Token::Symbol(name) => ASTNode::Var(name),
        //数组[1, 2, 3, ...]
        Token::LeftBracket => ASTNode::Array(parse_list(lexer, &Token::RightBracket)?),
        //(表达式)
        Token::LeftParen => {
            let node = parse_expr(lexer)?;
            expect_token(lexer, &Token::RightParen)?;
            node
        }
        //其余的情况先不管了
        _ => return unexpected_prev_token(lexer),
    };

    //在解析A后面的(x, ...)、[i, j]或者混合形式A()[]() ....
    //这里套循环的目的是为了支持f(3, 4)(4, 5)(5, 1)、A[2][4][6]()[3]等形式的嵌套执行
    loop {
        node = match lexer.next() {
            Token::LeftParen => {
                ASTNode::Apply(Box::new(node), parse_list(lexer, &Token::RightParen)?)
            },
            Token::LeftBracket => {
                let indices = parse_list(lexer, &Token::RightBracket)?;
                if indices.len() > 1 {
                    ASTNode::Index(Box::new(node), Box::new(ASTNode::Array(indices)))
                } else {
                    ASTNode::Index(Box::new(node), Box::new(indices[0].clone()))
                }
            }
            _ => {
                lexer.prev();
                break;
            }
        }
    }

    Ok(node)
}

//解析列表x, y, z, ...) 或者x, y, z, ...]
fn parse_list(lexer: &mut Lexer, closing: &Token) -> Result<Vec<ASTNode>, ParseError> {
    let mut args = vec![];
    while lexer.peek() != *closing {
        args.push(parse_expr(lexer)?);
        if lexer.peek() == Token::Comma {
            lexer.next();
        } else {
            break;
        }
    }
    expect_token(lexer, closing)?;
    Ok(args)
}

//解析lambda表达式
fn parse_lambda(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    Ok(match (lexer.next(), lexer.next(), lexer.next(), lexer.next()) {
        //() => body
        (Token::LeftParen, Token::RightParen, Token::Arrow, _) => {
            lexer.prev();
            ASTNode::Lambda(vec![], Box::new(parse_block(lexer)?))
        },
        //(x) => body
        (Token::LeftParen, Token::Symbol(x), Token::RightParen, Token::Arrow) => {
            ASTNode::Lambda(vec![x], Box::new(parse_block(lexer)?))
        },
        //(x, y, ...) => body
        (Token::LeftParen, Token::Symbol(x), Token::Comma, Token::Symbol(y)) => {
            let mut args = vec![x, y];
            loop {
                match lexer.next() {
                    Token::Comma => (),
                    Token::RightParen => break,
                    _ => return unexpected_prev_token(lexer),
                };

                match lexer.next() {
                    Token::Symbol(z) => args.push(z),
                    _ => return unexpected_prev_token(lexer),
                }
            }
            expect_token(lexer, &Token::Arrow)?;
            ASTNode::Lambda(args, Box::new(parse_block(lexer)?))
        },
        _ => return unexpected_prev_token(lexer),
    })
}

//解析语句块
fn parse_block(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    let mut nodes = vec![];
    if lexer.next() != Token::LeftBrace {
        return unexpected_prev_token(lexer);
    }
    loop {
        nodes.push(parse_statement(lexer)?);
        //一直解析到右大括号出线
        if lexer.peek() == Token::RightBrace {
            lexer.next();
            break
        }
    }
    Ok(ASTNode::Block(nodes))
}

//解析条件表达式
fn parse_cond_expr(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    //解析if条件及分支语句块
    let if_node = match lexer.next() {
        Token::If => {
            let cond = parse_expr(lexer)?;
            Box::new((cond, parse_block(lexer)?))
        },
        _ => {
            return unexpected_prev_token(lexer)
        },
    };
    //解析复数个elseif条件及分支语句块
    let mut elseif_nodes = vec![];
    loop {
        match lexer.peek() {
            Token::ElseIf => {
                lexer.next();
                let cond = parse_expr(lexer)?;
                elseif_nodes.push((cond, parse_block(lexer)?));
            }
            _ => {
                break;
            },
        };
    }
    //解析else分支语句块
    let else_node = match lexer.peek() {
        Token::Else => {
            lexer.next();
            Some(Box::new(parse_block(lexer)?))
        },
        _ => None,
    };

    Ok(ASTNode::Cond(if_node, elseif_nodes, else_node))
}

//错误处理
fn unexpected_token(lexer: &Lexer) -> Result<ASTNode, ParseError> {
    raise!(ParseError {
        token: lexer.peek(),
        span: lexer.span(),
    })
}

fn unexpected_prev_token(lexer: &mut Lexer) -> Result<ASTNode, ParseError> {
    lexer.prev();
    unexpected_token(lexer)
}

fn expect_token(lexer: &mut Lexer, token: &Token) -> Result<(), ParseError> {
    if lexer.next() != *token {
        unexpected_prev_token(lexer)?;
    }
    Ok(())
}