//! G22 on the product's pure parts: the copy's settings here; the bar's look, the foreground, the settling and each
//! language's verdict (`look`); who wrote each text (`wrote`); where its time went (`stages`); the cloud line (`cloud`)
//! and its sealed key never opened (`guard`).

mod cloud;
mod guard;
mod look;
mod stages;
mod wrote;

use kx_kernel::{KERNEL, SPEC as KERNEL_SPEC};
use kx_mod_voice::{SPEC, VoiceSettings};
use kx_settings::VERSION_KEY;

use super::table;
use crate::config::load;

#[test]
fn the_copy_reads_the_bench_models_on_the_runs_device_and_logs_at_the_runs_level() {
    let mut c = load().expect("the settings load");
    c.g22_bench.models_dir = r"E:\models".into();
    let t = table(&c.g22_product, &c.g22_bench).expect("a table");
    let mut section = t[kx_mod_voice::ID.as_str()]
        .as_table()
        .expect("[voice]")
        .clone();
    assert_eq!(
        section.remove(VERSION_KEY),
        Some(i64::from(SPEC.version).into())
    );
    let got: VoiceSettings = toml::Value::Table(section)
        .try_into()
        .expect("the section reads");
    let mut want = VoiceSettings::from_defaults().expect("the defaults");
    (want.models_dir, want.device) = (c.g22_bench.models_dir.clone(), c.g22_product.device.clone());
    assert_eq!(got, want);
    let kernel = t[KERNEL.as_str()].as_table().expect("[kernel]");
    let level = kernel.get("log_level").and_then(toml::Value::as_str);
    assert_eq!(
        level,
        Some(c.g22_product.log_level.as_str()),
        "the run's log level"
    );
    assert_eq!(
        kernel.get(VERSION_KEY),
        Some(&i64::from(KERNEL_SPEC.version).into())
    );
    assert_eq!(t.len(), 2, "only [voice] and [kernel]");
    c.g22_bench.models_dir.clear();
    assert!(
        table(&c.g22_product, &c.g22_bench).is_err(),
        "no models folder"
    );
}
