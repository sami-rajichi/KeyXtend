//! The gate-only build never ships: a program that holds the gate flag's text anywhere is refused.

use super::*;

/// A made-up program's bytes with `text` in the middle.
fn program(text: &[u8]) -> Vec<u8> {
    [b"MZ\x90\0 code ".as_slice(), text, b"\0 data"].concat()
}

/// The refusal for a staged program that holds the test settings' flag.
fn gate_build() -> StageError {
    StageError::GateFlag {
        exe: shown(PROGRAM),
        flag: settings().gate_flag,
    }
}

#[test]
fn a_program_that_holds_the_gate_flag_is_refused() {
    let flag = settings().gate_flag;
    for bytes in [
        program(flag.as_bytes()),
        flag.as_bytes().to_vec(),
        [flag.as_bytes(), b"\0"].concat(),
        [b"\0", flag.as_bytes()].concat(),
    ] {
        let refused = refuse_gate(&bytes, &flag, Path::new(PROGRAM));
        assert_eq!(refused, Err(gate_build()), "{bytes:?}");
    }
}

#[test]
fn a_program_without_the_gate_flag_passes() {
    let flag = settings().gate_flag;
    let (cut, upper) = (&flag[..flag.len() - 1], flag.to_uppercase());
    for text in ["--show --quit", cut, upper.as_str(), ""] {
        let passed = refuse_gate(&program(text.as_bytes()), &flag, Path::new(PROGRAM));
        assert_eq!(passed, Ok(()), "{text:?}");
    }
}

#[test]
fn an_empty_flag_refuses_every_program() {
    let refused = refuse_gate(&program(b""), "", Path::new(PROGRAM));
    assert!(refused.is_err(), "it fails closed");
}

#[test]
fn the_check_refuses_a_staged_gate_build_and_passes_a_shipped_one() {
    let stage = Fake::new(&[(PROGRAM, &[])]);
    let exe = stage.dir.path().join(PROGRAM);
    std::fs::write(&exe, program(settings().gate_flag.as_bytes())).unwrap();
    assert_eq!(refusal(stage.check()), Some(gate_build()));
    std::fs::write(&exe, program(b"--show")).unwrap();
    assert!(stage.check().is_ok());
}
