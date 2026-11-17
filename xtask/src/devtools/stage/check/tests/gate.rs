//! The gate-only builds never ship: a program that holds any gate flag's text anywhere is refused, the app's own and
//! the sound flag the voice gate build passes its worker.

use super::*;

/// A made-up program's bytes with `text` in the middle.
fn program(text: &[u8]) -> Vec<u8> {
    [b"MZ\x90\0 code ".as_slice(), text, b"\0 data"].concat()
}

/// The refusal for a staged program that holds `flag`.
fn gate_build(flag: &str) -> StageError {
    StageError::GateFlag {
        exe: shown(PROGRAM),
        flag: flag.to_string(),
    }
}

#[test]
fn a_program_that_holds_a_gate_flag_is_refused() {
    let config = settings();
    let flags = config.gate_flags();
    for flag in flags {
        for bytes in [
            program(flag.as_bytes()),
            flag.as_bytes().to_vec(),
            [flag.as_bytes(), b"\0"].concat(),
            [b"\0", flag.as_bytes()].concat(),
        ] {
            let refused = refuse_gate(&bytes, &flags, Path::new(PROGRAM));
            assert_eq!(refused, Err(gate_build(flag)), "{bytes:?}");
        }
    }
}

#[test]
fn the_gate_flags_are_the_apps_own_and_the_sound_flag() {
    let config = settings();
    assert_eq!(
        config.gate_flags(),
        [config.gate_flag.as_str(), kx_ipc::SOUND_FLAG]
    );
}

#[test]
fn a_program_without_a_gate_flag_passes() {
    let config = settings();
    let flags = config.gate_flags();
    for flag in flags {
        let (cut, upper) = (&flag[..flag.len() - 1], flag.to_uppercase());
        for text in ["--show --quit", cut, upper.as_str(), ""] {
            let passed = refuse_gate(&program(text.as_bytes()), &flags, Path::new(PROGRAM));
            assert_eq!(passed, Ok(()), "{text:?}");
        }
    }
}

#[test]
fn an_empty_flag_refuses_every_program() {
    let refused = refuse_gate(&program(b""), &[""], Path::new(PROGRAM));
    assert!(refused.is_err(), "it fails closed");
}

#[test]
fn the_check_refuses_a_staged_gate_build_and_passes_a_shipped_one() {
    let stage = Fake::new(&[(PROGRAM, &[])]);
    let exe = stage.dir.path().join(PROGRAM);
    for flag in settings().gate_flags() {
        std::fs::write(&exe, program(flag.as_bytes())).unwrap();
        assert_eq!(refusal(stage.check()), Some(gate_build(flag)), "{flag}");
    }
    std::fs::write(&exe, program(b"--show")).unwrap();
    assert!(stage.check().is_ok());
}
