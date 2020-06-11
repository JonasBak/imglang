use imglang::*;

fn test_script(externals: Externals, input: &'static str, expected: &'static str) {
    let mut lexer = Lexer::new(&input.to_string()).unwrap();
    let mut ast = parse(&mut lexer).unwrap();

    TypeChecker::annotate_types(&mut ast, Some(&externals)).unwrap();
    let chunks = Compiler::compile(&ast, Some(&externals));

    let mut output: Vec<u8> = vec![];

    let mut vm = VM::new(chunks, Some(&externals));
    vm.run(&mut output);

    let output = String::from_utf8(output).unwrap();

    assert_eq!(expected.to_string(), output);
}

#[test]
fn external_function_call() {
    let mut externals = Externals::new();
    externals.add_function(
        "externalFunction".to_string(),
        ExternalFunction {
            args_t: vec![],
            ret_t: AstType::Float,
            dispatch: |stack: &mut Stack| {
                stack.push(123.345);
            },
        },
    );

    test_script(externals, "print externalFunction();", "123.345\n");
}

#[test]
fn external_function_with_args() {
    let mut externals = Externals::new();
    externals.add_function(
        "externalFunction".to_string(),
        ExternalFunction {
            args_t: vec![AstType::Float],
            ret_t: AstType::Float,
            dispatch: |stack: &mut Stack| {
                let arg: f64 = stack.pop();
                stack.push(arg);
            },
        },
    );

    test_script(
        externals,
        "
            print externalFunction(12.34);
            print externalFunction(56.78);
        ",
        "12.34\n56.78\n",
    );
}
