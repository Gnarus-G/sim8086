use std::process::Command;

use insta::assert_debug_snapshot;
use sim8086::decode;

fn test_with(file: &str) {
    let _ = Command::new("nasm")
        .arg(format!("./{}.asm", file))
        .status()
        .unwrap();

    let buffer = std::fs::read(format!("./{}", file)).unwrap();

    assert_debug_snapshot!(decode::decode(&buffer));
}

#[test]
fn movs() {
    test_with("many_register_mov");

    test_with("more_movs");

    test_with("challenge_movs");
}
