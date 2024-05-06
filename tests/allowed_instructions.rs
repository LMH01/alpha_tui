use assert_cmd::Command;

#[test]
fn test_allowed_instructions_file() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_file_program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("examples/allowed_instructions.json")
        .assert();
    assert.success();
}

#[test]
fn test_allowed_instructions_file_full() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_file_full/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_file_full/instructions.json")
        .arg("--allowed-operations")
        .arg("add,sub")
        .arg("--allowed-comparisons")
        .arg("lt,le,eq")
        .assert();
    assert.success();
}

#[test]
fn test_allowed_instructions_only_instructions() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_only_instructions/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_only_instructions/instructions.json")
        .assert();
    assert.success();
}