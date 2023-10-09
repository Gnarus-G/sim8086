use std::{io::Write, process::Command};

use insta::assert_display_snapshot;

macro_rules! test_with {
    ($file:literal) => {
        let _ = Command::new("nasm")
            .arg(format!("./fixtures/decode/{}.asm", $file))
            .status()
            .unwrap();

        let app_output = assert_cmd::Command::cargo_bin("sim8086")
            .unwrap()
            .arg(format!("./fixtures/decode/{}", $file))
            .output()
            .map(|out| String::from_utf8(out.stdout).unwrap())
            .unwrap();

        let test_file_path = format!("./fixtures/decode/{}_test.asm", $file);

        let mut test_file = std::fs::File::create(&test_file_path).unwrap();

        write!(test_file, "{}", app_output).unwrap();

        let _ = Command::new("nasm").arg(&test_file_path).status().unwrap();

        assert_cmd::Command::new("diff")
            .arg(format!("./fixtures/decode/{}", $file))
            .arg(format!("./fixtures/decode/{}_test", $file))
            .assert()
            .success();

        std::fs::remove_file(test_file_path).unwrap();
        std::fs::remove_file(format!("./fixtures/decode/{}", $file)).unwrap();
        std::fs::remove_file(format!("./fixtures/decode/{}_test", $file)).unwrap();

        insta::with_settings!({ description => $file }, {
            assert_display_snapshot!(app_output);
        })
    };
}

#[test]
fn movs() {
    test_with!("many_register_mov");

    test_with!("more_movs");

    test_with!("challenge_movs");
}

#[test]
fn add_sub_cmp() {
    test_with!("add_sub_cmp");
}

#[test]
fn jumps() {
    let file = "jnz";

    let _ = Command::new("nasm")
        .arg(format!("./fixtures/decode/{}.asm", file))
        .status()
        .unwrap();

    let app_output = assert_cmd::Command::cargo_bin("sim8086")
        .unwrap()
        .arg(format!("./fixtures/decode/{}", file))
        .output()
        .map(|out| String::from_utf8(out.stdout).unwrap())
        .unwrap();

    std::fs::remove_file(format!("./fixtures/decode/{}", file)).unwrap();

    insta::with_settings!({ description => file }, {
        assert_display_snapshot!(app_output);
    })
}
