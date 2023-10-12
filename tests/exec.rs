use std::process::Command;

use insta::assert_display_snapshot;

macro_rules! test_with {
    ($file:literal) => {
        let _ = Command::new("nasm")
            .arg(format!("./fixtures/exec/{}.asm", $file))
            .status()
            .unwrap();

        let app_output = assert_cmd::Command::cargo_bin("sim8086")
            .unwrap()
            .arg(format!("./fixtures/exec/{}", $file))
            .arg("--exec")
            .output()
            .map(|out| String::from_utf8(out.stdout).unwrap())
            .unwrap();

        insta::with_settings!({ description => $file }, {
            assert_display_snapshot!(app_output);
        })
    };
}

#[test]
fn movs() {
    test_with!("immediate_movs");
    test_with!("register_movs");
}

#[test]
fn add_sub_cmp() {
    test_with!("add_sub_cmp");
}

#[test]
fn ip_tracking() {
    test_with!("ip_register");
}

#[test]
fn conditional_jumps() {
    // test_with!("add_sub_cmp");
}
