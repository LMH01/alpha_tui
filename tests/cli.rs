use assert_cmd::Command;


#[test]
fn test_cmd_check_compile_with_allowed_instructions_2() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/test_cmd_check_compile_with_allowed_instructions_2/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions")
        .arg("tests/test_cmd_check_compile_with_allowed_instructions_2/instructions.txt")
        .assert();
    assert.success();
}
