use assert_cmd::Command;

#[test]
fn test_memory_config_no_gamma() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_memory_config_no_gamma/program.alpha")
        .arg("run")
        .arg("--memory-config-file")
        .arg("tests/input/test_memory_config_no_gamma/memory_config.json")
        .assert();
    assert.success();
}

#[test]
fn test_memory_config_no_gamma_2() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("check")
        .arg("tests/input/test_memory_config_no_gamma_2/program.alpha")
        .arg("run")
        .arg("--disable-memory-detection")
        .arg("-m")
        .arg("a,b,c,d,w,x,y,z,h1,h2,h3,h4")
        .assert();
    assert.success();
}
