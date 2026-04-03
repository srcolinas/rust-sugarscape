use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn invalid_config_exits_with_code_1() {
    let output = tempfile::NamedTempFile::new().expect("temp output file");
    cargo_bin_cmd!("sugarscape-sim")
        .args([
            "--config",
            fixture("invalid_config.yaml").to_str().unwrap(),
            "--output",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .code(1);
}

#[test]
fn valid_config_exits_with_code_0() {
    let config = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.example.yaml");
    let output = tempfile::NamedTempFile::new().expect("temp output file");
    cargo_bin_cmd!("sugarscape-sim")
        .args([
            "--config",
            config.to_str().unwrap(),
            "--output",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .success();
}
