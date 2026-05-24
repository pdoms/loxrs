fn print_diagnostic(output: &[u8], expected: &[u8], position: usize) {
    eprintln!(
        "expected {} at position: {}, got: {}",
        expected[position] as char, position, output[position] as char
    );
    eprintln!(
        "...[{}]...",
        str::from_utf8(&output[position.saturating_sub(8)..(position + 8).min(output.len())])
            .unwrap()
    );
}

#[test]
fn test_lox_output() {
    let input = std::fs::read("tests/data/test.lox").unwrap();
    let expected = std::fs::read_to_string("tests/data/test.expected").unwrap();

    let mut output = Vec::new();
    let mut scanner = lox_rs::scanner::Scanner::new(&input);
    scanner.parse().unwrap();

    let mut parser = lox_rs::parser::Parser::new(&scanner.tokens);
    let stmts = parser.parse().unwrap();
    let mut resolver = lox_rs::resolver::Resolver::default();
    for stmt in &stmts {
        resolver.resolve_stmt(stmt).unwrap();
    }

    let mut interpreter = lox_rs::interpreter::Interpreter::new(&mut output);
    interpreter.resolve(resolver.locals);
    interpreter.interpret(&stmts).unwrap();

    let bytes = expected.as_bytes();

    for (i, ch) in output.iter().enumerate() {
        if *ch != bytes[i] {
            print_diagnostic(&output, bytes, i);
            assert_eq!(*ch, bytes[i]);
        }
    }

    let _ = std::fs::remove_file("tmp.txt");
}
