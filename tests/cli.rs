use assert_cmd::Command;

#[test]
fn test_cmd_check_compile_with_allowed_instructions() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_cmd_check_compile_with_allowed_instructions/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_cmd_check_compile_with_allowed_instructions/instructions.json")
        .assert();
    assert.success();
}

#[test]
fn test_cmd_check_compile_with_allowed_instructions_2() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_cmd_check_compile_with_allowed_instructions_2/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_cmd_check_compile_with_allowed_instructions_2/instructions.json")
        .assert();
    assert.success();
}

#[test]
fn test_cmd_check_compile_with_index_memory_cells() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_cmd_check_compile_with_allowed_instructions_2/program.alpha")
        .arg("compile")
        .arg("--index-memory-cells")
        .arg("5-10")
        .assert();
    assert.success();
}