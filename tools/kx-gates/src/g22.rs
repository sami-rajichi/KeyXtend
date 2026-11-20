//! G22, voice typing: `g22 keyxtend` dictates each shipped clip through the product's test build into target-window
//! (`g22prod`), `g22 bench` times the default models (`g22bench`) and `g22 staged` checks the staged worker's edges
//! (`g22staged`). The first two pass kx-ipc's sound flag, so only a kx-gates built with `voice` holds them.
#![cfg(windows)]

use kx_module_api::ports::DevicePick;
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
/// The gate worker's features: `gate` plays a sound file in place of the microphone, `card` adds the card part.
pub const WORKER_FEATURES: &str = "gate,card";
/// The variable naming the Vulkan SDK's folder, which the card part's build needs.
pub const CARD_SDK: &str = "VULKAN_SDK";
/// The app's gate build feature, which plays each clip in place of the microphone through the gate worker.
pub const APP_FEATURES: &str = "gate-voice";
/// The flag that picks `g22 keyxtend`'s device for one run, in place of `[g22_product] device`.
pub const DEVICE_FLAG: &str = "--device";

/// How the gate worker is built: its features, and the variable its card part's build needs.
pub fn worker_build() -> String {
    format!("--features {WORKER_FEATURES} with {CARD_SDK} set")
}

/// How the app's gate build is built.
pub fn app_build() -> String {
    format!("--features {APP_FEATURES}")
}

/// Whether `gate` run on `name` takes `DEVICE_FLAG`: only G22 on the product does.
pub fn takes_device(gate: &str, name: &str) -> bool {
    gate == GATE && name == NAME
}

/// The pick `DEVICE_FLAG`'s `value` names, or one sentence saying what it takes.
pub fn device_arg(value: &str) -> Result<DevicePick, String> {
    let (cpu, auto) = (DevicePick::CPU, DevicePick::AUTO);
    DevicePick::parse(value).ok_or_else(|| {
        format!("{DEVICE_FLAG} takes {cpu}, {auto} or a card's id as the engine lists it, as [voice] device does")
    })
}

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

/// Runs G22 on the product, its bench or the staged worker's edges, as `name` says, once each clip's sentence is the
/// credited one, as every run judges by them.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if let Some(hint) = unvoiced(GATE, name) {
        return Err(hint);
    }
    let run: fn(&Ctx) -> Result<Value, String> = match name {
        STAGED => g22staged::run,
        #[cfg(feature = "voice")]
        BENCH => g22bench::run,
        #[cfg(feature = "voice")]
        NAME => g22prod::run,
        _ => {
            return Err(format!(
                "G22 runs on {NAME}, {BENCH} or {STAGED}, not {name}"
            ));
        }
    };
    ctx.cfg.g22_bench.credited(&ctx.cfg.dir)?;
    run(ctx)
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
    fn the_worker_build_line_names_the_workers_gate_and_card_features_and_the_sdk_variable() {
        let manifest: toml::Table = include_str!("../../../apps/keyxtend-worker/Cargo.toml")
            .parse()
            .expect("TOML");
        let features = manifest["features"].as_table().expect("[features]");
        for feature in WORKER_FEATURES.split(',') {
            assert!(features.contains_key(feature), "{feature}");
        }
        let build = worker_build();
        assert!(
            build.contains(&format!("--features {WORKER_FEATURES}")),
            "{build}"
        );
        assert!(build.contains(CARD_SDK), "{build}");
        let p11 = include_str!("../kx-gates-p11.toml");
        assert!(
            p11.contains(&format!("--features {WORKER_FEATURES}`")),
            "the bench's build line"
        );
        assert!(p11.contains(CARD_SDK), "the card build's variable");
    }

    #[test]
    fn the_app_build_line_names_the_apps_gate_voice_feature_as_the_settings_file_does() {
        let manifest: toml::Table = include_str!("../../../apps/keyxtend/Cargo.toml")
            .parse()
            .expect("TOML");
        let features = manifest["features"].as_table().expect("[features]");
        assert!(features.contains_key(APP_FEATURES), "{APP_FEATURES}");
        assert_eq!(app_build(), format!("--features {APP_FEATURES}"));
        let p11 = include_str!("../kx-gates-p11.toml");
        assert!(
            p11.contains(&format!("{}`", app_build())),
            "the app's build line"
        );
    }

    #[test]
    fn only_g22_on_the_product_takes_a_device_pick_and_a_bad_one_is_refused_in_one_line() {
        assert!(takes_device(GATE, NAME));
        assert!(!takes_device(GATE, BENCH) && !takes_device(GATE, STAGED));
        assert!(!takes_device("g19", NAME), "another gate");
        let card = device_arg("Vulkan1").expect("a card");
        assert_eq!(card.name(), "Vulkan1");
        assert!(matches!(card, DevicePick::Card(_)));
        assert_eq!(device_arg(DevicePick::CPU), Ok(DevicePick::Cpu));
        assert_eq!(device_arg(DevicePick::AUTO), Ok(DevicePick::Auto));
        for bad in ["Vulkan 1", "", r"..\card"] {
            let e = device_arg(bad).expect_err(bad);
            assert!(e.contains(DEVICE_FLAG) && !e.contains('\n'), "{e}");
        }
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
