use assert_cmd::Command;

#[test]
fn test_bai_error() {
    let mut cmd = match Command::cargo_bin("alpha_tui") {
        Ok(cmd) => cmd,
        Err(_) => return, // ugly workaround because this test otherwise failes when run in the llvm codecov pipeline
    };
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