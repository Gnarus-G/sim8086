use std::{io::Write, process::Command};

use insta::assert_display_snapshot;

fn test_with(file: &str) {
    let _ = Command::new("nasm")
        .arg(format!("./{}.asm", file))
        .status()
        .unwrap();

    let app_output = assert_cmd::Command::cargo_bin("sim8086")
        .unwrap()
        .arg(format!("./{}", file))
        .output()
        .map(|out| String::from_utf8(out.stdout).unwrap())
        .unwrap();

    let test_file_path = format!("./{}_test.asm", file);

    let mut test_file = std::fs::File::create(&test_file_path).unwrap();

    write!(test_file, "{}", app_output).unwrap();

    let _ = Command::new("nasm").arg(test_file_path).status().unwrap();

    assert_cmd::Command::new("diff")
        .arg(format!("./{}", file))
        .arg(format!("./{}_test", file))
        .assert()
        .success();

    assert_display_snapshot!(app_output);
}

#[test]
fn movs() {
    test_with("many_register_mov");

    test_with("more_movs");

    test_with("challenge_movs");
}

#[test]
fn add_sub_cmp() {
    test_with("add_sub_cmp");
}
