//! Weak mode's caps turn into the job's values on any PC, and a job takes a child by its handle.

use std::os::windows::io::AsRawHandle;
use std::process::{Command, Stdio};

use super::*;

fn weak(cores: u32, rate_pct: u32) -> Weak {
    Weak {
        cores,
        rate_pct,
        mem_mb: 4096,
    }
}

#[test]
fn caps_turn_settings_into_job_values() {
    let c = Caps::from(&weak(2, 30), 16).expect("valid");
    assert_eq!(c.affinity, 0b11);
    assert_eq!(
        c.cpu_rate, 375,
        "2 of 16 CPUs at 30 %, in hundredths of a percent"
    );
    assert_eq!(c.mem_bytes, 4096 * 1024 * 1024);
}

#[test]
fn each_allowed_core_runs_at_the_share_on_any_pc() {
    for cpus in [2, 8, 16] {
        let c = Caps::from(&weak(2, 30), cpus).expect("valid");
        assert!((c.cpu_cores(cpus) - 0.6).abs() < 1e-9, "{cpus} CPUs");
    }
    let tiny = Caps::from(&weak(1, 1), 256).expect("valid");
    assert_eq!(tiny.cpu_rate, 1, "never 0, which Windows refuses");
}

#[test]
fn more_cores_than_the_pc_has_means_all_of_them() {
    let c = Caps::from(&weak(16, 30), 4).expect("valid");
    assert_eq!(c.affinity, 0b1111);
}

#[test]
fn every_core_or_a_huge_memory_cap_saturates() {
    let all = Caps::from(&weak(u32::MAX, 30), usize::MAX).expect("valid");
    assert_eq!(all.affinity, usize::MAX);
    let mut w = weak(2, 30);
    w.mem_mb = u64::MAX;
    assert_eq!(Caps::from(&w, 8).expect("valid").mem_bytes, usize::MAX);
}

#[test]
fn bad_caps_are_reported_not_ignored() {
    assert!(Caps::from(&weak(2, 30), 0).is_err(), "no cores known");
    let mut w = weak(2, 30);
    w.mem_mb = 0;
    assert!(Caps::from(&w, 8).is_err(), "no memory");
    assert!(Caps::from(&weak(0, 30), 8).is_err());
    assert!(Caps::from(&weak(2, 0), 8).is_err());
    assert!(Caps::from(&weak(2, 101), 8).is_err());
}

#[test]
fn a_child_joins_a_job_by_its_handle() {
    let job = Job::new(None).expect("a job");
    // `findstr` waits for its input until we close it, so it cannot end before it joins.
    let mut child = Command::new("findstr")
        .arg("x")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("starts");
    let joined = job.assign_handle(HANDLE(child.as_raw_handle()));
    drop(child.stdin.take());
    let _ = child.wait();
    assert_eq!(joined, Ok(()));
}

#[test]
fn a_job_is_made_with_and_without_caps() {
    let cpus = std::thread::available_parallelism().map_or(1, usize::from);
    let caps = Caps::from(&weak(2, 30), cpus).expect("valid");
    assert!(Job::new(None).is_ok());
    assert!(Job::new(Some(&caps)).is_ok());
}

#[test]
fn the_shipped_weak_mode_is_the_roadmaps_weak_pc() {
    let w = crate::config::load().expect("the settings load").g8.weak;
    assert_eq!(
        Caps::from(&w, w.cores as usize).map(|c| c.affinity.count_ones()),
        Ok(w.cores)
    );
}
