use std::process::Command;

use insta::assert_debug_snapshot;
use sim8086::decode;

fn test_with(file: &str) -> Vec<u8> {
    let _ = Command::new("nasm")
        .arg(format!("./{}.asm", file))
        .status()
        .unwrap();

    std::fs::read(format!("./{}", file)).unwrap()
}

#[test]
fn movs() {
    let buffer = test_with("many_register_mov");
    assert_debug_snapshot!(decode::decode(&buffer));

    let buffer = test_with("more_movs");
    assert_debug_snapshot!(decode::decode(&buffer));
}
