//! The keyboard argument: gates aimed at the product take its name, gates that still drive the frozen spike face take
//! the face's, and `close` takes either; a gate's own names stay valid.
#![cfg(windows)]

use spike_core::config::FACE;

use crate::usage::usage;
use crate::{g8, g8hook, g8lang, g8lit, g8predict, g19, g22, g22bench, g22type, product};

/// Gates aimed at the product, which take its name.
pub(crate) const PRODUCT_GATES: [&str; 12] = [
    "g2", "g3", "g4", "g8", "g9", "g10", "g11", "g12", "g13", "g26", "g27", "close",
];
/// Gates that still drive the frozen spike face until their phase, which take its name.
pub(crate) const FACE_GATES: [&str; 6] = ["g7", "g19", "g22", "g23", "g24", "close"];

/// The keyboards `gate` takes: the product, the frozen face, both or none.
fn keyboards(gate: &str) -> Vec<&'static str> {
    let product = PRODUCT_GATES.contains(&gate).then_some(product::NAME);
    let face = FACE_GATES.contains(&gate).then_some(FACE);
    product.into_iter().chain(face).collect()
}

/// Names besides a keyboard that `gate` takes.
fn own_names(gate: &str) -> &'static [&'static str] {
    match gate {
        "g8" => &[
            g8::WEAK,
            g8lang::ARABIC,
            g8lit::LIT,
            g8lit::OSK,
            g8hook::HOOK,
            g8hook::HOOK_WEAK,
            g8predict::PREDICT,
        ],
        "g19" => &[g19::CORE],
        "g22" => &[g22::BENCH, g22bench::WITH_CLOUD, g22type::TYPING],
        _ => &[],
    }
}

/// Refuses a keyboard gate's name that is neither a keyboard it takes nor one of the gate's own names.
pub fn check(gate: &str, name: &str) -> Result<(), String> {
    let keyboards = keyboards(gate);
    let known = keyboards.contains(&name) || own_names(gate).contains(&name);
    if !keyboards.is_empty() && !known {
        return Err(format!("unknown keyboard {name}\n{}", usage()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Faces the P1 spike had before the Qt stage; they are gone.
    const GONE: [&str; 2] = ["slint", "tauri"];

    #[test]
    fn product_gates_take_the_product_and_refuse_the_frozen_face() {
        for gate in PRODUCT_GATES {
            assert!(check(gate, product::NAME).is_ok(), "{gate}");
        }
        for gate in PRODUCT_GATES.iter().filter(|g| !FACE_GATES.contains(g)) {
            let e = check(gate, FACE).expect_err(gate);
            assert!(e.contains("unknown keyboard"), "{gate}: {e}");
        }
    }

    #[test]
    fn face_gates_take_the_frozen_face_and_refuse_the_product() {
        for gate in FACE_GATES {
            assert!(check(gate, FACE).is_ok(), "{gate} {FACE}");
        }
        for gate in FACE_GATES.iter().filter(|g| !PRODUCT_GATES.contains(g)) {
            assert!(check(gate, product::NAME).is_err(), "{gate}");
        }
    }

    #[test]
    fn close_takes_the_product_or_the_frozen_face() {
        assert!(check("close", product::NAME).is_ok());
        assert!(check("close", FACE).is_ok());
    }

    #[test]
    fn every_keyboard_gate_refuses_a_face_that_is_gone_and_shows_the_usage() {
        for gate in PRODUCT_GATES.iter().chain(&FACE_GATES) {
            for gone in GONE {
                let e = check(gate, gone).expect_err(gone);
                assert!(e.contains("unknown keyboard"), "{gate} {gone}: {e}");
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
        assert!(check("g7", g19::CORE).is_err());
        for name in [
            g8::WEAK,
            g8lang::ARABIC,
            g8lit::LIT,
            g8lit::OSK,
            g8hook::HOOK,
            g8hook::HOOK_WEAK,
            g8predict::PREDICT,
        ] {
            assert!(check("g8", name).is_ok(), "g8 {name}");
            assert!(check("g9", name).is_err(), "{name} is G8's");
        }
    }

    #[test]
    fn g12_takes_the_product_now_and_refuses_the_frozen_face() {
        assert!(check("g12", product::NAME).is_ok());
        assert!(check("g12", FACE).is_err());
    }

    #[test]
    fn a_gate_without_a_keyboard_takes_any_name() {
        assert!(check("g1", "notepad").is_ok());
        assert!(check("g5", "admin").is_ok());
        assert!(check("g25", "notepad").is_ok());
    }
}
