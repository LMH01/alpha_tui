use assert_cmd::Command;

#[test]
fn test_example_program_compile_faculty() {
    test_example_program_compile("faculty.alpha")
}

#[test]
fn test_example_program_compile_index_memory_cells() {
    test_example_program_compile("index_memory_cells.alpha")
}

#[test]
fn test_example_program_compile_loop_example() {
    test_example_program_compile("loop_example.alpha")
}

#[test]
fn test_example_program_compile_matrix_mult() {
    test_example_program_compile("matrix_mult.alpha")
}

#[test]
fn test_example_program_compile_stack_loop() {
    test_example_program_compile("stack_loop.alpha")
}

#[test]
fn test_example_program_compile_stack() {
    test_example_program_compile("stack.alpha")
}

fn test_example_program_compile(file_name: &str) {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg(format!("examples/programs/{}", file_name))
        .arg("compile")
        .assert();
    assert.success();
}