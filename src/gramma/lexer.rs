use std::str::Chars;
use std::iter::{Fuse, Peekable};
use crate::gramma::token::{Token, Op};

//词法分析器
pub struct Lexer {
    icurrent: usize,
    tokens: Vec<Token>,
    //可以用来指示错误发生的位置
    spans: Vec<Span>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Span(pub usize, pub usize);

impl Lexer {
    pub fn new(line: &str) -> Lexer {
        let mut stream = CharStream::new(line);
        let mut tokens = vec![];
        let mut spans = vec![];

        //把line中的字符转化为token存储在向量tokens中
        loop {
            let c = stream.peek();
            if c == '\0' {
                //空输入直接退出
                break;
            } else if c.is_whitespace() {
                //空格则继续loop
                stream.next();
                continue;
            } else {
                let begin = stream.icurrent;
                tokens.push(Self::parse_token(&mut stream));
                spans.push(Span(begin, stream.icurrent));
            }
        }

        Lexer {
            icurrent: 0,
            tokens,
            spans,
        }
    }

    pub fn peek(&self) -> Token {
        self.tokens.get(self.icurrent).cloned().unwrap_or(Token::End)
    }

    pub fn next(&mut self) -> Token {
        let tok = self.peek();
        self.icurrent += 1;
        tok
    }

    pub fn prev(&mut self) {
        assert!(self.icurrent > 0);
        self.icurrent -= 1;
    }

    pub fn span(&self) -> Span {
        if self.icurrent < self.tokens.len() {
            self.spans[self.icurrent]
        } else {
            *self.spans.last().unwrap()
        }
    }
}

impl Lexer {
    fn parse_token(stream: &mut CharStream) -> Token {
        const DIGITS: &'static str = "0123456789";
        const DIGITS_DOT: &'static str = "0123456789.";
        const SYMBOLS: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_";

        //这里mut的原因是下面会多次改变该值
        let mut c = stream.peek();
        
        if SYMBOLS.contains(c) {
            //符号
            let mut buffer = String::new();
            while SYMBOLS.contains(c) || DIGITS.contains(c) {
                buffer.push(c);
                stream.next();
                c = stream.peek();
            }

            //关键词
            return match buffer.as_ref() {
                "let" => Token::Let,
                "if" => Token::If,
                "elseif" => Token::ElseIf,
                "else" => Token::Else,
                "true" => Token::Boolean(true),
                "false" => Token::Boolean(false),
                _ => Token::Symbol(buffer),
            };
        } else if DIGITS_DOT.contains(c) {
            //数字
            let mut buffer = String::new();
            while DIGITS_DOT.contains(c) {
                buffer.push(c);
                stream.next();
                c = stream.peek();
            }
            return Token::Number(buffer)
        } else {
            //其他符号
            stream.next();

            let tk = match (c, stream.peek()) {
                ('=', '=') => Some(Token::Operator(Op::Eq)),
                ('!', '=') => Some(Token::Operator(Op::Neq)),
                ('<', '=') => Some(Token::Operator(Op::Lte)),
                ('>', '=') => Some(Token::Operator(Op::Gte)),
                ('&', '&') => Some(Token::Operator(Op::And)),
                ('|', '|') => Some(Token::Operator(Op::Or)),
                ('=', '>') => Some(Token::Arrow),
                _ => None,
            };

            if tk.is_some() {
                //由两个符号构成的运算符
                stream.next();
                tk.unwrap()
            } else {
                //由单个符号构成的运算符
                match c {
                    '(' => Token::LeftParen,
                    ')' => Token::RightParen,
                    '[' => Token::LeftBracket,
                    ']' => Token::RightBracket,
                    '{' => Token::LeftBrace,
                    '}' => Token::RightBrace,
                    ',' => Token::Comma,
                    '=' => Token::Assign,
                    ':' => Token::Colon,
                    ';' => Token::SemiColon,
                    '+' => Token::Operator(Op::Add),
                    '-' => Token::Operator(Op::Sub),
                    '*' => Token::Operator(Op::Mul),
                    '/' => Token::Operator(Op::Div),
                    '^' => Token::Operator(Op::Pow),
                    '%' => Token::Operator(Op::Mod),
                    '>' => Token::Operator(Op::Gt),
                    '<' => Token::Operator(Op::Lt),
                    '!' => Token::Operator(Op::Not),
                    c => Token::Illegal(c),
                }
            }
        }
    }
}

struct CharStream<'a> {
    icurrent: usize,
    iterator: Peekable<Fuse<Chars<'a>>>,
}

impl<'a> CharStream<'a> {
    fn new(line: &'a str) -> CharStream<'a> {
        Self {
            icurrent: 0,
            //fuse生成可以无限次next的iterator 出现一次None后再next还是None
            //peakable让迭代器无需调用next便可以查看顶部元素
            iterator: line.chars().fuse().peekable(),
        }
    }

    fn next(&mut self) -> char {
        self.icurrent += 1;
        self.iterator.next().unwrap_or('\0')
    }

    fn peek(&mut self) -> char {
        self.iterator.peek().cloned().unwrap_or('\0')
    }
}

#[cfg(test)]
mod test_charstream {
    use super::{ CharStream };

    #[test]
    fn input1() {
        let line = "abc";
        let mut stream = CharStream::new(line);

        assert_eq!(stream.peek(), 'a');
        assert_eq!(stream.next(), 'a');
        assert_eq!(stream.peek(), 'b');
        assert_eq!(stream.next(), 'b');
        assert_eq!(stream.peek(), 'c');
        assert_eq!(stream.next(), 'c');
        assert_eq!(stream.peek(), '\0');
        assert_eq!(stream.next(), '\0');
    }
}