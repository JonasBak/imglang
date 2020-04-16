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
