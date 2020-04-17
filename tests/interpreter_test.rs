use imglang::*;

#[test]
fn extern_function_call_and_variable_declaration() {
    let scope = Environment::new();
    declare(
        &scope,
        &"five".to_string(),
        Value::ExternFunction(|_| Value::Number(5.0)),
    )
    .unwrap();
    let source = "var abc = five();".to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"abc".to_string()),
        Some(Value::Number(5.0))
    );
}

#[test]
fn number_of_arguments() {
    let scope = Environment::new();
    declare(
        &scope,
        &"len".to_string(),
        Value::ExternFunction(|v| Value::Number(v.len() as f64)),
    )
    .unwrap();
    let source = "
        var a = len();
        var b = len(1);
        var c = len(1,2);
        var d = len(1,2,3);
        var e = len(1,2,3,4);
    "
    .to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"a".to_string()),
        Some(Value::Number(0.0))
    );
    assert_eq!(
        get_value(&scope, &"b".to_string()),
        Some(Value::Number(1.0))
    );
    assert_eq!(
        get_value(&scope, &"c".to_string()),
        Some(Value::Number(2.0))
    );
    assert_eq!(
        get_value(&scope, &"d".to_string()),
        Some(Value::Number(3.0))
    );
    assert_eq!(
        get_value(&scope, &"e".to_string()),
        Some(Value::Number(4.0))
    );
}

#[test]
fn while_loop() {
    let scope = Environment::new();
    let source = "
        var a = 0;
        while (a < 10) {
            a = a + 1;
        }
        "
    .to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"a".to_string()),
        Some(Value::Number(10.0))
    );
}

#[test]
fn nested_scopes() {
    let scope = Environment::new();
    let source = "
        var a = 0;
        var b = 0;
        var c = 0;
        {
            var a = 1;
            {
                b = a;
                var c = 10;
            }
            var a = a + b + c;
            c = a;
        }
        "
    .to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"a".to_string()),
        Some(Value::Number(0.0))
    );
    assert_eq!(
        get_value(&scope, &"b".to_string()),
        Some(Value::Number(1.0))
    );
    assert_eq!(
        get_value(&scope, &"c".to_string()),
        Some(Value::Number(2.0))
    );
}

#[test]
fn test_if() {
    let scope = Environment::new();
    let source = "
        var a = 0;
        if (true) a = 1;
        if (false) a = 10;
        "
    .to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"a".to_string()),
        Some(Value::Number(1.0))
    );
}

#[test]
fn test_if_else() {
    let scope = Environment::new();
    let source = "
        var a = 0;
        var b = 0;
        if (true) a = 1;
        else b = 10;
        if (false) a = 10;
        else b = 1;
        "
    .to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"a".to_string()),
        Some(Value::Number(1.0))
    );
    assert_eq!(
        get_value(&scope, &"b".to_string()),
        Some(Value::Number(1.0))
    );
}

#[test]
fn multiple_assignments() {
    let scope = Environment::new();
    let source = "
        var a = 10;
        var a = 5;
        "
    .to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"a".to_string()),
        Some(Value::Number(5.0))
    );
}

#[test]
fn test_fibonacci() {
    let scope = Environment::new();
    let source = "
        fun fib(n) {
            if (n < 1) return 0;
            if (n == 1) return 1;
            return fib(n-1) + fib(n-2);
        }
        var f0 = fib(0);
        var f1 = fib(1);
        var f2 = fib(2);
        var f3 = fib(3);
        var f4 = fib(4);
        var f5 = fib(5);
        "
    .to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"f0".to_string()),
        Some(Value::Number(0.0))
    );
    assert_eq!(
        get_value(&scope, &"f1".to_string()),
        Some(Value::Number(1.0))
    );
    assert_eq!(
        get_value(&scope, &"f2".to_string()),
        Some(Value::Number(1.0))
    );
    assert_eq!(
        get_value(&scope, &"f3".to_string()),
        Some(Value::Number(2.0))
    );
    assert_eq!(
        get_value(&scope, &"f4".to_string()),
        Some(Value::Number(3.0))
    );
    assert_eq!(
        get_value(&scope, &"f5".to_string()),
        Some(Value::Number(5.0))
    );
}

#[test]
fn nested_scopes_closures() {
    let scope = Environment::new();
    let source = "
        fun test() {
            var i = 0;
            fun f() {
                i = i + 1;
                return i;
            }
            return f;
        }
        var f = test();
        var a0 = f();
        var a1 = f();
        var a2 = f();
        var a3 = f();
        "
    .to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"a0".to_string()),
        Some(Value::Number(1.0))
    );
    assert_eq!(
        get_value(&scope, &"a1".to_string()),
        Some(Value::Number(2.0))
    );
    assert_eq!(
        get_value(&scope, &"a2".to_string()),
        Some(Value::Number(3.0))
    );
    assert_eq!(
        get_value(&scope, &"a3".to_string()),
        Some(Value::Number(4.0))
    );
}

#[test]
fn scopes_and_functions() {
    let scope = Environment::new();
    let source = "
        var i = 0;
        fun f() {
            var b = 11;
            i = i + 1;
            return i;
        }
        var a0 = f();
        var a1 = f();
        i = i + 10;
        var a2 = f();
        "
    .to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&scope).unwrap();
    assert_eq!(
        get_value(&scope, &"a0".to_string()),
        Some(Value::Number(1.0))
    );
    assert_eq!(
        get_value(&scope, &"a1".to_string()),
        Some(Value::Number(2.0))
    );
    assert_eq!(
        get_value(&scope, &"a2".to_string()),
        Some(Value::Number(13.0))
    );
    assert_eq!(
        get_value(&scope, &"i".to_string()),
        Some(Value::Number(13.0))
    );
    assert_eq!(get_value(&scope, &"b".to_string()), None);
}
