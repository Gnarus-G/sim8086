use std::process::Command;

use insta::assert_display_snapshot;

#[test]
fn estimating_cycles() {
    let _ = Command::new("nasm")
        .arg("./fixtures/estimating_cycles.asm")
        .status()
        .unwrap();

    let app_output = assert_cmd::Command::cargo_bin("sim8086")
        .unwrap()
        .arg("./fixtures/estimating_cycles")
        .arg("--exec")
        .arg("-c")
        .output()
        .map(|out| String::from_utf8(out.stdout).unwrap())
        .unwrap();

    insta::with_settings!({ description => "estimating_cycles" }, {
        assert_display_snapshot!(app_output);
    });

    std::fs::remove_file("./fixtures/estimating_cycles").unwrap();
}
