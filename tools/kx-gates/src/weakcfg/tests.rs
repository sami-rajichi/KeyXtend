//! The weak PC's one section: the shipped one passes, each bad key is refused, and it sits in `kx-gates.toml` alone,
//! with no old copy in G8's or the bench's section.

use std::path::Path;

use super::*;
use crate::cfgfile::FILE;

fn shipped() -> Weak {
    crate::config::load().expect("the settings load").weak
}

#[test]
fn the_shipped_weak_pc_passes_its_check() {
    assert_eq!(shipped().check(), Ok(()));
}

#[test]
fn a_bad_value_is_refused() {
    let bad: [fn(&mut Weak); 5] = [
        |w| w.cores = 0,
        |w| w.cores = u8::MAX,
        |w| w.core_pct = 0,
        |w| w.core_pct = CpuCap::FULL_PCT + 1,
        |w| w.mem_mb = 0,
    ];
    for (i, spoil) in bad.iter().enumerate() {
        let mut w = shipped();
        spoil(&mut w);
        assert!(w.check().is_err(), "case {i} passed");
    }
}

#[test]
fn the_memory_must_hold_more_than_a_voice_job_alone() {
    let job_mb = VoiceSettings::from_defaults().expect("the defaults").job_mb;
    let mut w = shipped();
    w.mem_mb = job_mb;
    assert!(w.check().is_err(), "at job_mb");
    w.mem_mb = job_mb + 1;
    assert_eq!(w.check(), Ok(()));
}

#[test]
fn the_section_sits_in_the_main_file_and_no_gate_keeps_its_own_copy() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let main: toml::Table = std::fs::read_to_string(dir.join(FILE))
        .expect("the main file reads")
        .parse()
        .expect("it parses");
    assert!(main.contains_key(SECTION), "[{SECTION}] in {FILE}");
    let all = crate::cfgload::read(dir).expect("the settings read");
    assert!(all["g8"].get(SECTION).is_none(), "no [g8.weak]");
    for old in ["weak_cores", "weak_core_pct", "weak_mb"] {
        assert!(all["g22_bench"].get(old).is_none(), "g22_bench.{old}");
    }
}
