use assert_cmd::Command;

#[test]
fn test_example_allowed_instructions_file() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_example_allowed_instructions_file_program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("examples/allowed_instructions.json")
        .assert();
    assert.success();
}
