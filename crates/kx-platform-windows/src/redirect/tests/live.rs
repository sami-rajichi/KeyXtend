//! The real probe on the real `%LOCALAPPDATA%\KeyXtend`. It runs only when the build saw `KX_LIVE_HIDDEN=1` (the build
//! script's `HIDDEN_VAR`), as this PC's sessions run inside the Claude app's package and so expect a hidden run; CI's
//! runs are normal, so it stays off there.

use super::*;

/// The probe files of the shipped table in `dir`, or none when it is missing.
fn probes_in(dir: &Path) -> Vec<String> {
    let start = &table().unwrap().probe;
    let all = if dir.is_dir() { names(dir) } else { Vec::new() };
    all.into_iter().filter(|n| n.starts_with(start)).collect()
}

#[test]
#[cfg_attr(
    not(kx_live_hidden),
    ignore = "probes the real app-data folder; opt-in"
)]
fn the_real_probe_sees_this_session_as_hidden_and_leaves_nothing() {
    let data = crate::dirs::local_data().unwrap().join(APP_DIR_NAME);
    let got = WinHidden::new().probe(&data);
    eprintln!("live probe: {got:?}");
    let Ok(Probe::Hidden { package, .. }) = got else {
        panic!("expected a hidden run, got {got:?}");
    };
    assert!(
        probes_in(&data).is_empty(),
        "no probe file in the data folder"
    );
    let root = crate::dirs::local_data()
        .unwrap()
        .join(&table().unwrap().packages);
    let mirror = root.join(&package).join(steps()).join(APP_DIR_NAME);
    assert!(
        probes_in(&mirror).is_empty(),
        "no probe file in the hidden copy"
    );
}
