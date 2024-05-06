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
fn test_cmd_check_compile_with_allowed_instructions_comparisons_operations() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_cmd_check_compile_with_allowed_instructions_comparisons_operations/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_cmd_check_compile_with_allowed_instructions_comparisons_operations/instructions.json")
        .arg("--allowed-operations")
        .arg("add,sub")
        .arg("--allowed-comparisons")
        .arg("lt,le,eq")
        .assert();
    assert.success();
}
