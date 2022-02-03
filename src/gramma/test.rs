#[cfg(test)]
mod lexer {
    use crate::gramma::token::{Token, Op};
    use crate::gramma::lexer::Lexer;

    #[test]
    fn input1() {
        let mut lexer = Lexer::new("1.23 + 2 * 3");
        assert_eq!(lexer.peek(), Token::Number("1.23".into()));
        assert_eq!(lexer.next(), Token::Number("1.23".into()));
        assert_eq!(lexer.next(), Token::Operator(Op::Add));
        assert_eq!(lexer.next(), Token::Number("2".into()));
        assert_eq!(lexer.next(), Token::Operator(Op::Mul));
        assert_eq!(lexer.next(), Token::Number("3".into()));
        assert_eq!(lexer.next(), Token::End);
        assert_eq!(lexer.next(), Token::End);
        assert_eq!(lexer.next(), Token::End);
        assert_eq!(lexer.next(), Token::End);
    }

    #[test]
    fn input2() {
        let mut lexer = Lexer::new(">= < == * / => [");
        assert_eq!(lexer.next(), Token::Operator(Op::Gte));
        assert_eq!(lexer.next(), Token::Operator(Op::Lt));
        assert_eq!(lexer.next(), Token::Operator(Op::Eq));
        assert_eq!(lexer.next(), Token::Operator(Op::Mul));
        assert_eq!(lexer.next(), Token::Operator(Op::Div));
        assert_eq!(lexer.next(), Token::Arrow);
        assert_eq!(lexer.peek(), Token::LeftBracket);
        lexer.prev();
        lexer.prev();
        lexer.prev();
        lexer.prev();
        assert_eq!(lexer.peek(), Token::Operator(Op::Eq));
    }

    #[test]
    fn input3() {
        let mut lexer = Lexer::new("a1.1 pfsd2 _fds");
        assert_eq!(lexer.peek(), Token::Symbol("a1".into()));
        assert_eq!(lexer.next(), Token::Symbol("a1".into()));
        assert_eq!(lexer.next(), Token::Number(".1".into()));
        assert_eq!(lexer.peek(), Token::Symbol("pfsd2".into()));

        lexer.prev();
        assert_eq!(lexer.peek(), Token::Number(".1".into()));
        assert_eq!(lexer.next(), Token::Number(".1".into()));

        assert_eq!(lexer.next(), Token::Symbol("pfsd2".into()));
        assert_eq!(lexer.next(), Token::Symbol("_fds".into()));
        lexer.prev();
        assert_eq!(lexer.next(), Token::Symbol("_fds".into()));
        assert_eq!(lexer.next(), Token::End);
    }
}


#[cfg(test)]
mod evaluator {
    use crate::gramma::lexer::Lexer;
    use crate::gramma::primitive::create_global_environment;
    use crate::gramma::ast::ASTValue;
    use crate::gramma::parser::parse_statement;
    use crate::gramma::evaluator::evaluate_node;

    fn check(inputs: Vec<&str>, expected: ASTValue) {
        let env = create_global_environment();

        let mut result: ASTValue = ASTValue::Number(0.0);
        for input in inputs {
            let mut lexer = Lexer::new(input);
            let ast = parse_statement(& mut lexer).ok().unwrap();
            if let Some(value) = evaluate_node(&ast, env.clone()).ok().unwrap() {
                result = value;
            }
        }

        assert_eq!(result, expected);
    }

    fn ast_array(elements: &[f64]) -> ASTValue {
        let mut results = vec![];
        for element in elements {
            results.push(ASTValue::Number(*element))
        } 
        ASTValue::Array(results.into())
    }

    #[test]
    fn assignment1() {
        let inputs = vec![
            "let a = 3;",
            "let b = 4;",
            "a = a + b"
        ];
        check(inputs, ASTValue::Number(7.0));
    }

    #[test]
    fn assignment2() {
        let inputs = vec![
            "let judge = (x) => { x > 3 };",
            "let f = (x) => {x + 2};",
            "let a = f(3);",
            "let b = f(1);",
            "f = (x) => {if judge(x) { x^2 } else { x/2 }};",
            "f(a) + f(b)"
        ];
        check(inputs, ASTValue::Number(26.5));
    }

    #[test]
    fn scope() {
        let inputs = vec![
            "let a = 3;",
            "let b = 4;",
            "let c = {let c = a + b; let d = a - b; c * d / 2};",
            "let d = {let c = a * b; let d = a / b; c * d / 2};",
            "{ a = c * d; b = c / d; }",
            "a * b"
        ];
        check(inputs, ASTValue::Number(12.25));
    }

    #[test]
    fn array_index1() {
        let inputs = vec![
            "let arr = [1, 2, 3, 4, 5]",
            "arr[2] * arr[3]"
        ];
        check(inputs, ASTValue::Number(12.0));
    }

    #[test]
    fn array_index2() {
        let inputs = vec![
            "let arr = [1, 2, 3, 4, 5]",
            "arr[0, 2, 4]"
        ];
        check(inputs, ast_array(&[1.0, 3.0, 5.0]));
    }

    #[test]
    fn array_of_fun_index1() {
        let inputs = vec![
            "let array_of_fun = [(x, y) => {2 * x + y}, (x, y) => {x ^ 2 + y}]",
            "array_of_fun(3, 4)"
        ];
        check(inputs, ast_array(&[10.0, 13.0]));
    }

    #[test]
    fn fun_return_array_index1() {
        let inputs = vec![
            "let fun_return_array = () => {[(x) => {x + 2}, (x) => {x ^ 2 + 1}, (x) => {x / 2 - 1}, (x) => {x ^ 3 - 1}]}",
            "fun_return_array()[1, 2](3)"
        ];
        check(inputs, ast_array(&[10.0, 0.5]));
    }

    #[test]
    fn fun_return_array_index2() {
        let inputs = vec![
            "let fun_return_array = (y) => {[(x) => {x + y + 2}, (x) => {x ^ 2 + y + 1}, (x) => {x / 2 + y - 1}, (x) => {x ^ 3 + y - 1}]}",
            "fun_return_array(3)[1, 2](3)"
        ];
        check(inputs, ast_array(&[13.0, 3.5]));
    }

    #[test]
    fn array_index3() {
        let inputs = vec![
            "let arr = [[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]]",
            "arr[1][3]"
        ];
        check(inputs, ASTValue::Number(9.0));
    }

    #[test]
    fn numerical() {
        use std::f64::consts;
        //一元函数
        check(vec!["acos(1)"], ASTValue::Number((1.0 as f64).acos()));
        check(vec!["sin(pi / 2)"], ASTValue::Number((consts::PI / 2.0).sin()));
        check(vec!["ln(e)"], ASTValue::Number((consts::E).ln()));
        check(vec!["sqrt(2)"], ASTValue::Number((2.0 as f64).sqrt()));

        //二元函数
        check(vec!["atan2(0.2, 3)"], ASTValue::Number((0.2 as f64).atan2(3 as f64)));
        check(vec!["log(5, 2)"], ASTValue::Number((5 as f64).log(2 as f64)));
    }

    #[test]
    fn map() {
        let inputs = vec![
            "let arr = [1, 2, 3, 4, 5];",
            "map(arr, (x) => { x ^ 2 })"
        ];
        check(inputs, ast_array(&[1.0, 4.0, 9.0, 16.0, 25.0]));
    }

    #[test]
    fn length() {
        check(vec!["length([1, 2, 3, 4, 5])"], ASTValue::Number(5.0));
    }

    #[test]
    fn range() {
        check(vec!["range(0, 6)"], ast_array(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0]));
    }

    #[test]
    fn linespace() {
        check(vec!["linespace(0, 1, 5)"], ast_array(&[0.0, 0.25, 0.5, 0.75, 1.0]));
    }

    #[test]
    fn usrdef() {
        let inputs = vec![
            "let f = (x) => {let a = 3; let b = 4; a = b - 2 * x; a + b + x}",
            "f(3)"
        ];
        check(inputs, ASTValue::Number(5.0));
    }

    #[test]
    fn frac() {
        let inputs = vec![
            "let frac = (n) => {if n == 1 {1} else {n * frac(n - 1)}}",
            "frac(5)"
        ];
        check(inputs, ASTValue::Number(120.0));
    }

    #[test]
    fn fib() {
        let inputs = vec![
            "let fib = (n) => {if n == 1 {1} elseif n == 2 {1} else {fib(n - 1) + fib(n - 2)}}",
            "fib(5)"
        ];
        check(inputs, ASTValue::Number(5.0));
    }

    #[test]
    fn higher_lambda1() {
        let inputs = vec![
            "let f = (g, x, y) => {g(x, y)}",
            "f((x, y) => {if (x > y) {x ^ 2 + y} else {x ^ 2 - y}}, 3, 4)"
        ];
        check(inputs, ASTValue::Number(5.0));
    }

    #[test]
    fn higher_lambda2() {
        let inputs = vec![
            "let f = () => {(x) => { 1 + x ^ 2} }",
            "f()(3)"
        ];
        check(inputs, ASTValue::Number(10.0));
    }

    #[test]
    fn higher_lambda3() {
        let inputs = vec![
            "let f = (z) => {(x, y) => { x + y + z} }",
            "f(1)(2, 3)"
        ];
        check(inputs, ASTValue::Number(6.0));
    }
}