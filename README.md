# tiny-interpreter

Tiny command-line-based interpreter is written by Rust. Basic framework and coding are copied and developed according to reposes in https://github.com/stijnh/rust-calculator and https://github.com/christoffel1989/C_Interpreter.

## Problem Need to Solve
* Index operation can't be placed on the left side of `=` to modify the specified array element.
* Lambda can't capture outer lambdas in outside scope.

## Future Plan
* Support `While`, `For` statement.

## Value types
* Numbers (examples: `1`, `3.2`, `-200`, `1.3333`) 
* Booleans (`true` and `false`)
* Premitive Functions (examples: `sin`, `cos`, `map`, 'range')
* Lambdas (expample: `(x, y) => {x + y}`, `(z) => { (x, y) => { x + y + z } }`)
* Arrays: (`[1, true, -4.1]`, `[]`, `[sin, cos, tan]`, `[(x) => { 2 * x }, (x) => { x ^ 2 }, (x) => { x + 2 }]`)

## Arithmetic Options
Supports the following operations:
* Arithmetic
    * Addition: `a + b`
    * Subtraction: `a - b`
    * Multiplication: `a * b`
    * Division: `a / b`
    * Power: `a ^ 2`
    * Mod: `a % b`
* Relational
    * Equal: `a == b`, `a != b`
    * Compare: `a < b`, `a <= b`
* Logic
    * Conjunction: `a && b`
    * Disjunction: `a || b`
    * Negation: `!a`

## Basic
The interpreter can evaluate one-line expression directly and print the evaluated result to the console.
```
>>> 3 + 5 * 6
ans = 33
>>> 3 > 2
ans = true
>>> [1, 2, 3, 4]
ans = [1, 2, 3, 4]
```
If you don't want to see the evaluated result, just add a single ';' at the end of the expression. The tiny interpreter will not return the evaluated result of one-line statement with ';' at the end.
```
>>> 1 + 2;
>>> 1 + 2
ans = 3
```

# Use Variable to Hold a Number, Boolean, or Array
If you want to hold some value for later use, you can use `let` statement to declare a valued variable. The tiny interpreter can not evaluate a named variable without declaration.
```
>>> let a = 1
ans = 1
>>> let b = 2;
>>> a + b
ans = 3
>>> c = 5
evaluate error: variable not define
>>> let d = true
ans = true
>>> let arr = [1, 2, 3, 4, 5]
ans = [1, 2, 3, 4, 5]
>>> let arr = [[1, 2, 3], [4, 5, 6]]
ans = [[1, 2, 3], [4, 5, 6]]
```
A declared variable can be rebonded to the other number, boolean or, array use assignment operation `=`.
```
>>> let a = 1
ans = 1
>>> a = false
ans = false
>>> a = [1, 2, 3, 4]
ans = [1, 2, 3, 4]
```

# Use Primitive Function
The interpreter allows you to operate with number, boolean, list or, declared variable holding them with primitive function.
```
>>> sin(pi / 2)
ans = 1
>>> let arr = [1, 2, 3, 4, 5];
>>> map(arr, sqrt)
ans = [1, 1.4142135623730951, 1.7320508075688772, 2, 2.23606797749979]
```

# Array Indexing
The interpreter allows you to index one or more elements inside an array.
```
>>> let arr1 = [1, 2, 3, 4, 5];
>>> arr1[1, 2, 1]
ans = [2, 3, 2]
>>> let arr2 = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
>>> arr2[1][2]
ans = 6
```

# Use Block to Chain a Bunch of Statements
A bunch of statements can be chained inside a block surrounded by `{` and `}.` The Interpreter will evaluate all statements in order. The interpreter will evaluate all statements in-order until meeting a statement with `;` at the end or `}`. The evaluated result of the entire Block is equal to the last evaluated statement.
```
>>> {let a = 3; let b = 4; a + b}
>>> {let a = 3; let b = 4; a + b;}
ans = 7
>>> {let a = 3; let b = 4; 2 * a; a + b}
ans = 6
```

# Scoping Shielding
Every Block has its own lifetime area. Variable declared in current scope will shield variable with the same name but declared in the outer scope.
```
>>> let a = 3;
>>> let b = 4;
>>> let c = {let c = a + b; let d = a - b; c * d / 2};
>>> c
ans = -3.5
>>> d
evaluate error: variable not define
>>> let d = {let c = a * b; let d = a / b; c * d / 2}
ans = 4.5
```

# Branch Control
The interpreter will evaluate just a part of statements if condition statement is used.
```
>>> let a = 3;
>>> let b = 4;
>>> let c = if a > b {5} elseif a == b {4} else {3}
ans == 3
>>> let d = if a == b {5} elseif a < b {4} else {3}
ans == 4
```

# Use Variable to Hold a Lambda
If you want to define a function for later use, you can use to `let` statement to declare a functional variable. The tiny interpreter can not evaluate a named variable without declaration. The result of a lambda is equal to the result of lambda body block.
```
>>> let f1 = (x) => { x ^ 2 + 1 }
ans = lambda
>>> f1(3)
ans = 10
>>> let f2 = (x) => { [x + 2, x ^ 2] };
>>> f2(5)
ans = [7, 25]
```
The variable used to hold lambda can be rebonded to the other value type.
```
>>> let f = (x) => { x ^ 2 + 1 }
ans = lambda
>>> f = 5
ans = 5
```
In this way, the variable of number, boolean and, array can be regarded as lambda with the constant result.

# Use Recursion Function
Intepretor supports to define a recursion fuction.
```
>>> let frac = (n) => { if n == 1 { 1 } else { n * frac(n - 1) } };
>>> frac(5)
ans = 120
```

# Use High-Order Function
The interpreter supports defining a function with another lambda as input or output.
```
>>> let f1 = (g, x) => { g(x) };
>>> f1((x) => { sin(x) }, pi / 2)
ans = 1
>>> let f2 = (z) => {(x, y) => { x + y + z} };
>>> f2(1)(2, 3)
ans = 6
```

# Use Array to hold multiple Lambda
The interpreter allows you to declare an array that holds multiple lambdas. Index operation `[]` and function call operation `()` can be combined. 
```
>>> let f = [(x, y) => {2 * x + y}, (x, y) => {x ^ 2 + y}];
>>> f(3, 4)[1]
ans = 13
```