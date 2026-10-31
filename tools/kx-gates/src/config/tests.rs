//! Tests of `config`.

use super::*;

#[test]
fn the_shipped_file_loads_and_has_every_app() {
    let cfg = load().expect("kx-gates.toml loads");
    for app in crate::apps::AppKind::ALL {
        assert!(cfg.app(app.name()).is_ok(), "missing app {}", app.name());
    }
    assert_eq!(cfg.g3.probe_keys.len(), 3);
    assert!(cfg.g4.clicks > 0 && cfg.timing.quit_wait_ms > 0);
    assert!(cfg.hand.try_secs > 0 && cfg.g5.rounds > 0);
    for app in ["explorer", "word_file"] {
        assert!(cfg.app(app).is_ok(), "missing app {app}");
    }
    assert!(cfg.probes.move_steps > 0 && cfg.g6.phase_ms > 0);
    assert!(!cfg.g25.panel_classes.is_empty());
}

#[test]
fn g3_probes_only_keys_of_the_product_board() {
    let mut g3 = load().expect("kx-gates.toml loads").g3;
    assert!(g3.check().is_ok());
    g3.probe_keys.push("c-0xFFFF".into());
    let e = g3.check().expect_err("no such key");
    assert!(e.contains("c-0xFFFF"), "{e}");
    g3.probe_keys.clear();
    assert!(g3.check().is_err(), "probing nothing checks nothing");
}

#[test]
fn the_target_window_title_is_no_title_of_the_spike() {
    let target = kx_target_window::config::load().expect("target config loads");
    let spike = spike_core::config::load().expect("spike.toml loads");
    let taken = spike_core::panelcfg::titles(&spike);
    assert!(
        !taken.contains(&&target.title),
        "kx-gates closes windows by title, so the target's must be its own"
    );
}

#[test]
fn program_resolves_only_relative_paths_with_a_folder() {
    let cfg = GatesConfig {
        dir: PathBuf::from("base"),
        ..load().expect("loads")
    };
    assert_eq!(cfg.program("notepad.exe"), PathBuf::from("notepad.exe"));
    assert_eq!(
        cfg.program("../t/x.exe"),
        Path::new("base").join("../t/x.exe")
    );
}
