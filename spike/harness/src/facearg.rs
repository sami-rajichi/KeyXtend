//! The face argument: only the Qt face exists; a gate's own names stay valid.

use spike_core::config::FACE;

use crate::usage::usage;
use crate::{g19, g22, g22bench, g22type};

/// Gates that take the face's name.
const GATES: [&str; 10] = [
    "g2", "g3", "g4", "g7", "g12", "g19", "g22", "g23", "g24", "close",
];

/// Names besides the face that `gate` takes.
fn own_names(gate: &str) -> Vec<&'static str> {
    match gate {
        "g19" => vec![g19::CORE],
        "g22" => vec![g22::BENCH, g22bench::WITH_CLOUD, g22type::TYPING],
        _ => vec![],
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
