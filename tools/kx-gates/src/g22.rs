//! G22, voice typing: `g22 keyxtend` dictates each shipped clip through the product's test build into target-window
//! (`g22prod`), `g22 bench` times the default models (`g22bench`) and `g22 staged` checks the staged worker's edges
//! (`g22staged`). The first two pass kx-ipc's sound flag, so only a kx-gates built with `voice` holds them.
#![cfg(windows)]

use serde_json::Value;

use crate::apps::Ctx;
use crate::g22staged;
use crate::product::NAME;
#[cfg(feature = "voice")]
use crate::{g22bench, g22prod};

/// The gate's name on the command line.
const GATE: &str = "g22";
/// Name that runs the bench.
pub const BENCH: &str = "bench";
/// Name that runs the staged worker's edges.
pub const STAGED: &str = "staged";
/// The names that pass the sound flag to a gate build, so they need kx-gates built with `voice`.
const VOICED: [&str; 2] = [NAME, BENCH];
/// The feature that adds them.
const VOICE_FEATURE: &str = "voice";

/// The one-line hint for `gate` run on `name` when this kx-gates lacks the `voice` feature it needs; none otherwise.
pub fn unvoiced(gate: &str, name: &str) -> Option<String> {
    let needs = gate == GATE && VOICED.contains(&name);
    (needs && !cfg!(feature = "voice")).then(|| {
        format!(
            "{GATE} {name} passes the sound flag: build kx-gates with --features {VOICE_FEATURE}"
        )
    })
}

/// kx-ipc's sound flag as an argument. `black_box` keeps its bytes whole in a release exe, where a short copy would
/// be inlined, so the stage and `dev-install` find it and refuse to sign a `voice` build.
#[cfg(feature = "voice")]
pub fn sound_flag() -> String {
    std::hint::black_box(kx_ipc::SOUND_FLAG).to_owned()
}

/// Runs G22 on the product, its bench or the staged worker's edges, as `name` says.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if let Some(hint) = unvoiced(GATE, name) {
        return Err(hint);
    }
    match name {
        STAGED => g22staged::run(ctx),
        #[cfg(feature = "voice")]
        BENCH => g22bench::run(ctx),
        #[cfg(feature = "voice")]
        NAME => g22prod::run(ctx),
        _ => Err(format!(
            "G22 runs on {NAME}, {BENCH} or {STAGED}, not {name}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "voice"))]
    #[test]
    fn without_voice_the_runs_passing_the_sound_flag_give_the_build_hint() {
        for name in VOICED {
            let hint = unvoiced(GATE, name).expect(name);
            assert!(hint.contains("--features voice"), "{hint}");
            assert!(!hint.contains('\n'), "one line");
        }
        assert_eq!(unvoiced(GATE, STAGED), None, "the staged edges pass none");
        assert_eq!(unvoiced("g19", NAME), None, "another gate");
    }

    #[cfg(feature = "voice")]
    #[test]
    fn with_voice_every_run_is_here_and_passes_kx_ipcs_flag() {
        for name in [NAME, BENCH, STAGED] {
            assert_eq!(unvoiced(GATE, name), None, "{name}");
        }
        assert_eq!(sound_flag(), kx_ipc::SOUND_FLAG);
    }

    #[test]
    fn the_hint_names_a_default_off_feature_of_kx_gates() {
        let manifest: toml::Table = include_str!("../Cargo.toml").parse().expect("TOML");
        let features = manifest["features"].as_table().expect("[features]");
        assert!(features.contains_key(VOICE_FEATURE), "{VOICE_FEATURE}");
        let default = features["default"].as_array().expect("default");
        assert!(
            default.iter().all(|f| f.as_str() != Some(VOICE_FEATURE)),
            "off by default"
        );
    }
}
