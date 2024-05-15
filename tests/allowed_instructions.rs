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
fn test_allowed_instructions_file_imc() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_file_imc/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_file_imc/instructions.json")
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

#[test]
fn test_allowed_instructions_only_instructions_fail() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_only_instructions_fail/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_only_instructions_fail/instructions.json")
        .assert();
    assert.stdout(
        r#"Building instructions
Building runtime
Check unsuccessful, program did not compile.
Error: build_program_error

  × when building program
  ╰─▶ build_program::instruction_not_allowed_error
      
        × instruction 'p(h1) := 20' in line '2' is not allowed
        help: Make sure that you include this type ('M := C') of instruction
      in
              the whitelist or use a different instruction.
              These types of instructions are allowed:
      
              A := C
      

"#,
    );
}

#[test]
fn test_allowed_instructions_only_comparisons() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_only_comparisons/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_only_comparisons/instructions.json")
        .assert();
    assert.success();
}

#[test]
fn test_allowed_instructions_only_comparisons_fail() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_only_comparisons_fail/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_only_comparisons_fail/instructions.json")
        .assert();
    assert.stdout(
        r#"Building instructions
Building runtime
Check unsuccessful, program did not compile.
Error: build_program_error

  × when building program
  ╰─▶ build_program::comparison_not_allowed_error
      
        × comparison '==' in line '1' is not allowed
        help: Make sure that you include this comparison ('==') in the allowed
              comparisons or use a different instruction.
              To mark this comparison as allowed you can use: '--allowed-
              comparisons "eq"'
      

"#,
    );
}

#[test]
fn test_allowed_instructions_no_comparisons() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_no_comparisons/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_no_comparisons/instructions.json")
        .assert();
    assert.stdout(
        r#"Building instructions
Building runtime
Check unsuccessful, program did not compile.
Error: build_program_error

  × when building program
  ╰─▶ build_program::comparison_not_allowed_error
      
        × comparison '==' in line '1' is not allowed
        help: Make sure that you include this comparison ('==') in the allowed
              comparisons or use a different instruction.
              To mark this comparison as allowed you can use: '--allowed-
              comparisons "eq"'
      

"#,
    );
}

#[test]
fn test_allowed_instructions_only_operations() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_only_operations/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_only_operations/instructions.json")
        .assert();
    assert.success();
}

#[test]
fn test_allowed_instructions_only_operations_fail() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_only_operations_fail/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_only_operations_fail/instructions.json")
        .assert();
    assert.stdout(
        r#"Building instructions
Building runtime
Check unsuccessful, program did not compile.
Error: build_program_error

  × when building program
  ╰─▶ build_program::operation_not_allowed_error
      
        × operation '-' in line '2' is not allowed
        help: Make sure that you include this operation ('-') in the allowed
              operations or use a different instruction.
              To mark this operation as allowed you can use: '--allowed-
      operations
              "sub"'
      

"#,
    );
}

#[test]
fn test_allowed_instructions_no_operations() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_allowed_instructions_no_operations/program.alpha")
        .arg("compile")
        .arg("--allowed-instructions-file")
        .arg("tests/input/test_allowed_instructions_no_operations/instructions.json")
        .assert();
    assert.stdout(
        r#"Building instructions
Building runtime
Check unsuccessful, program did not compile.
Error: build_program_error

  × when building program
  ╰─▶ build_program::operation_not_allowed_error
      
        × operation '+' in line '1' is not allowed
        help: Make sure that you include this operation ('+') in the allowed
              operations or use a different instruction.
              To mark this operation as allowed you can use: '--allowed-
      operations
              "add"'
      

"#,
    );
}
