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
