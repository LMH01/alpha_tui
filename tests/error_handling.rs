use assert_cmd::Command;

#[test]
fn test_bai_error() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("load")
        .arg("tests/input/test_bai_error/program.alpha")
        .arg("--allowed-instructions")
        .arg("tests/input/test_bai_error/allowed_instructions_a.txt")
        .assert();
    assert.stderr(
        r#"Error: build_program_error

  × when building program
  ╰─▶ build_program::instruction_not_allowed_error
      
        × instruction 'pop' in line '2' is not allowed
        help: Make sure that you include this type ('pop') of instruction
      in the
              whitelist or use a different instruction.
              These types of instructions are allowed:
      
              push
      

"#,
    );
}

#[test]
fn test_bpe_operation_not_allowed() {

    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("load")
        .arg("tests/input/test_bpe_operation_not_allowed/program.alpha")
        .arg("--allowed-operations")
        .arg("add")
        .assert();
    assert.stderr(
        r#"Error: build_program_error

  × when building program
  ╰─▶ build_program::operation_not_allowed_error
      
        × operation '-' in line '1' is not allowed
        help: Make sure that you include this operation ('-') in the allowed
              operations or use a different instruction.
              To mark this operation as allowed you can use: '--allowed-
      operations
              "sub"'
      

"#,
    );

}