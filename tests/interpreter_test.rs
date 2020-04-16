use imglang::*;

#[test]
fn extern_function_call_and_variable_declaration() {
    let mut env = Scope::new();
    env.declare(
        &"five".to_string(),
        Value::ExternFunction(|_| Value::Number(5.0)),
    )
    .unwrap();
    let source = "var abc = five();".to_string();
    let tokens = parse_string(&source).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.eval(&mut env).unwrap();
    assert_eq!(env.get(&"abc".to_string()), Some(Value::Number(5.0)));
}

#[test]
fn number_of_arguments() {
    let mut env = Scope::new();
    env.declare(
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
    ast.eval(&mut env).unwrap();
    assert_eq!(env.get(&"a".to_string()), Some(Value::Number(0.0)));
    assert_eq!(env.get(&"b".to_string()), Some(Value::Number(1.0)));
    assert_eq!(env.get(&"c".to_string()), Some(Value::Number(2.0)));
    assert_eq!(env.get(&"d".to_string()), Some(Value::Number(3.0)));
    assert_eq!(env.get(&"e".to_string()), Some(Value::Number(4.0)));
}
