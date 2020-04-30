use imglang::*;
use std::fs;

fn load_files(prefix: &'static str) -> (String, String) {
    let input = String::from_utf8(fs::read(format!("{}.input", prefix)).unwrap()).unwrap();
    let output = String::from_utf8(fs::read(format!("{}.output", prefix)).unwrap()).unwrap();
    (input, output)
}

fn test_script(prefix: &'static str) {
    let (input, expected) = load_files(prefix);

    let mut lexer = Lexer::new(&input).unwrap();
    let mut ast = parse(&mut lexer).unwrap();

    TypeChecker::annotate_types(&mut ast).unwrap();
    let chunks = Compiler::compile(&ast);

    let mut output: Vec<u8> = vec![];

    let mut vm = VM::new(chunks);
    vm.run(&mut output);

    let output = String::from_utf8(output).unwrap();

    assert_eq!(expected, output);
}

#[test]
fn variables() {
    test_script("tests/scripts/variables");
}

#[test]
fn if_else() {
    test_script("tests/scripts/if_else");
}

#[test]
fn test_while() {
    test_script("tests/scripts/while");
}

#[test]
fn functions() {
    test_script("tests/scripts/functions");
}

#[test]
fn fibonacci() {
    test_script("tests/scripts/fibonacci");
}

#[test]
fn nested_scopes() {
    test_script("tests/scripts/nested_scopes");
}

#[test]
fn closure_counter() {
    test_script("tests/scripts/counter");
}
