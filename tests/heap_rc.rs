use imglang::*;

fn run_script(input: &'static str) -> VM {
    let mut lexer = Lexer::new(&input.to_string()).unwrap();
    let mut ast = parse(&mut lexer).unwrap();

    TypeChecker::annotate_types(&mut ast).unwrap();
    let chunks = Compiler::compile(&ast);

    let mut output: Vec<u8> = vec![];

    let mut vm = VM::new(chunks);
    vm.run(&mut output);
    vm
}

#[test]
fn count_objects_multiple_assignments() {
    let vm = run_script(
        r#"
        var a = "only object";
        var b = a;
        var c = a;
        a = a = a = a = a;
    "#,
    );
    assert_eq!(vm.heap_ptr().count_objects(), 1);
}

#[test]
fn count_objects_reassign() {
    let vm = run_script(
        r#"
        var a = "fist object";
        var b = "second object";
        a = b;
    "#,
    );
    assert_eq!(vm.heap_ptr().count_objects(), 1);
}

#[test]
fn count_objects_nested_scopes() {
    let vm = run_script(
        r#"
        {
            var a = "heap 1";
            var b = "heap 2";
            var c = "heap 3";
            a = b = a = b = a;
        }
    "#,
    );
    assert_eq!(vm.heap_ptr().count_objects(), 0);
}

#[test]
fn count_objects_combined_scopes() {
    let vm = run_script(
        r#"
        var a = "outer";
        {
            var b = "inner 1";
            var c = "inner 2";
            a = b;
        }
    "#,
    );
    assert_eq!(vm.heap_ptr().count_objects(), 1);
}

#[test]
fn count_objects_expr_statements() {
    let vm = run_script(
        r#"
        var a = "outer";
        "string literal";
        a;
    "#,
    );
    assert_eq!(vm.heap_ptr().count_objects(), 1);
}

#[test]
fn count_objects_function_return() {
    let vm = run_script(
        r#"
        fun returnString() str {
            var a = "string 1";
            var b = "string 2";
            var c = "string 3";
            return "string 4";
        }
        returnString();
        returnString();
        returnString();
        returnString();
        returnString();
        returnString();
        var a = returnString();
    "#,
    );
    assert_eq!(vm.heap_ptr().count_objects(), 1);
}

#[test]
fn count_objects_function_argument() {
    let vm = run_script(
        r#"
        fun stringArg(a str) {
            print a;
        }
        stringArg("string 1");
        var a = "string 2";
        stringArg(a);
    "#,
    );
    assert_eq!(vm.heap_ptr().count_objects(), 1);
}

#[test]
fn count_objects_function_return_argument() {
    let vm = run_script(
        r#"
        fun stringArg(a str) str {
            var b = "some var";
            return a;
        }
        var a = "string 2";
        stringArg(a);
        var b = stringArg(a);
        a = stringArg(a);
    "#,
    );
    assert_eq!(vm.heap_ptr().count_objects(), 1);
}
