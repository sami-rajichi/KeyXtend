//! The face argument: only the Qt face exists; a gate's own names stay valid.
#![cfg(windows)]

use spike_core::config::FACE;

use crate::usage::usage;
use crate::{g19, g22, g22bench, g22type};

/// Gates that take the face's name.
pub(crate) const GATES: [&str; 10] = [
    "g2", "g3", "g4", "g7", "g12", "g19", "g22", "g23", "g24", "close",
];

/// Names besides the face that `gate` takes.
fn own_names(gate: &str) -> &'static [&'static str] {
    match gate {
        "g19" => &[g19::CORE],
        "g22" => &[g22::BENCH, g22bench::WITH_CLOUD, g22type::TYPING],
        _ => &[],
    }
}

/// Refuses a face gate's name that is neither the face nor one of the gate's own names.
pub fn check(gate: &str, name: &str) -> Result<(), String> {
    let known = name == FACE || own_names(gate).contains(&name);
    if GATES.contains(&gate) && !known {
        return Err(format!("unknown face {name}\n{}", usage()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Faces the P1 spike had before the Qt stage; they are gone.
    const GONE: [&str; 2] = ["slint", "tauri"];

    #[test]
    fn every_face_gate_takes_the_qt_face() {
        for gate in GATES {
            assert!(check(gate, FACE).is_ok(), "{gate} {FACE}");
        }
    }

    #[test]
    fn every_face_gate_refuses_a_face_that_is_gone_and_shows_the_usage() {
        for gate in GATES {
            for gone in GONE {
                let e = check(gate, gone).expect_err(gone);
                assert!(e.contains("unknown face"), "{gate} {gone}: {e}");
                assert!(e.contains(&usage()), "{gate} {gone}: {e}");
            }
        }
    }

    #[test]
    fn a_gates_own_names_stay_valid_for_that_gate_only() {
        assert!(check("g19", g19::CORE).is_ok());
        for name in [g22::BENCH, g22bench::WITH_CLOUD, g22type::TYPING] {
            assert!(check("g22", name).is_ok(), "g22 {name}");
            assert!(check("g2", name).is_err(), "g2 {name}");
        }
        assert!(check("g2", g19::CORE).is_err());
    }

    #[test]
    fn a_gate_without_a_face_takes_any_name() {
        assert!(check("g1", "notepad").is_ok());
        assert!(check("g5", "admin").is_ok());
    }
}
