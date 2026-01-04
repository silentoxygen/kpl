use assert_cmd::prelude::*;
use std::process::Command;

fn bin() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("kube-podlog"))
}

#[test]
fn dev_smoke_human_runs_and_exits() {
    let mut cmd = bin();

    cmd.env("RUST_LOG", "off")
        .args([
            "--dev",
            "-n",
            "default",
            "-l",
            "app=web",
            "--dev-rate-ms",
            "1",
            "--dev-lines",
            "3",
        ])
        .assert()
        .success();
}

#[test]
fn dev_smoke_json_is_valid_ndjson_and_nonempty() {
    let mut cmd = bin();

    let assert = cmd
        .env("RUST_LOG", "off")
        .args([
            "--dev",
            "-n",
            "default",
            "-l",
            "app=web",
            "--dev-rate-ms",
            "1",
            "--dev-lines",
            "3",
            "--json",
            "--no-color",
        ])
        .assert()
        .success();

    let out = String::from_utf8_lossy(&assert.get_output().stdout).to_string();

    let mut count = 0usize;
    for line in out.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let v: serde_json::Value =
            serde_json::from_str(line).expect("each line must be valid JSON");
        for k in ["ts", "namespace", "pod", "container", "message"] {
            assert!(v.get(k).is_some(), "missing key {k} in {v}");
        }
        count += 1;
    }

    assert!(
        count > 0,
        "expected some JSON lines, got 0. stdout was empty."
    );
}
