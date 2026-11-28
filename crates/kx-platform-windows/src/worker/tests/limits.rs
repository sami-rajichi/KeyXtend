//! Every job's own job object, read from outside: it runs alone under its own memory cap and is barred from shared UI.

use super::*;
use crate::worker::job::{ALONE, LIMITS, MIB};
use kx_platform::worker::{FetchFrames, NetFrames, VoiceFrames};
use windows::Win32::System::JobObjects::{
    IsProcessInJob, JOB_OBJECT_UILIMIT, JOB_OBJECT_UILIMIT_DESKTOP,
    JOB_OBJECT_UILIMIT_DISPLAYSETTINGS, JOB_OBJECT_UILIMIT_EXITWINDOWS,
    JOB_OBJECT_UILIMIT_GLOBALATOMS, JOB_OBJECT_UILIMIT_HANDLES, JOB_OBJECT_UILIMIT_READCLIPBOARD,
    JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS, JOB_OBJECT_UILIMIT_WRITECLIPBOARD,
    JOBOBJECT_BASIC_UI_RESTRICTIONS, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectBasicUIRestrictions, JobObjectExtendedLimitInformation, QueryInformationJobObject,
};

/// Every job's start, each with its own memory cap: the record and pick jobs' from their callers.
fn every_job() -> Vec<Launch> {
    let word = |job: &str| Launch {
        args: vec![job.to_owned()],
        memory_mb: CFG.memory_mb,
    };
    let more = [VoiceFrames::JOB, FetchFrames::JOB, NetFrames::JOB].map(word);
    let named = [
        Launch::media(&CFG),
        Launch::vault(),
        Launch::record(CFG.memory_mb),
        Launch::pick(CFG.memory_mb),
    ];
    named.into_iter().chain(more).collect()
}

#[test]
fn the_job_caps_each_job_alone_in_it() {
    for launch in every_job() {
        let (_input, _output, child) = spawner().start(&launch).unwrap();
        let mb = usize::try_from(launch.memory_mb).unwrap();
        capped(&child, mb * MIB);
    }
}

/// Checks that `child` runs alone in its job, under `bytes` of memory and the job's limits.
#[allow(unsafe_code, reason = "Reads the limits and members of our own job.")]
fn capped(child: &WinChild, bytes: usize) {
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    let size = size32::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>();
    let mut inside = false.into();
    // SAFETY: our live job and process handles, and locals of the sizes passed.
    unsafe {
        let info = (&raw mut limits).cast();
        let class = JobObjectExtendedLimitInformation;
        QueryInformationJobObject(Some(child.job.handle()), class, info, size, None).unwrap();
        IsProcessInJob(
            raw(&child.process),
            Some(child.job.handle()),
            &raw mut inside,
        )
        .unwrap();
    }
    assert!(inside.as_bool());
    let basic = limits.BasicLimitInformation;
    assert_eq!(basic.LimitFlags.0 & LIMITS.0, LIMITS.0);
    assert_eq!(basic.ActiveProcessLimit, ALONE);
    assert_eq!(limits.ProcessMemoryLimit, bytes);
}

/// The shared UI `child`'s job bars it from.
#[allow(unsafe_code, reason = "Reads the UI limits of our own job.")]
fn barred(child: &WinChild) -> JOB_OBJECT_UILIMIT {
    let mut ui = JOBOBJECT_BASIC_UI_RESTRICTIONS::default();
    let size = size32::<JOBOBJECT_BASIC_UI_RESTRICTIONS>();
    // SAFETY: our live job handle and a local of the size passed.
    unsafe {
        let info = (&raw mut ui).cast();
        let job = Some(child.job.handle());
        QueryInformationJobObject(job, JobObjectBasicUIRestrictions, info, size, None).unwrap();
    }
    ui.UIRestrictionsClass
}

#[test]
fn the_job_bars_every_job_from_shared_ui() {
    let all = [
        JOB_OBJECT_UILIMIT_HANDLES,
        JOB_OBJECT_UILIMIT_READCLIPBOARD,
        JOB_OBJECT_UILIMIT_WRITECLIPBOARD,
        JOB_OBJECT_UILIMIT_GLOBALATOMS,
        JOB_OBJECT_UILIMIT_DESKTOP,
        JOB_OBJECT_UILIMIT_DISPLAYSETTINGS,
        JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS,
        JOB_OBJECT_UILIMIT_EXITWINDOWS,
    ];
    for launch in every_job() {
        let (_input, _output, child) = spawner().start(&launch).unwrap();
        let ui = barred(&child);
        for limit in all {
            assert!(ui.contains(limit), "{:?}: {:#x}", launch.args, limit.0);
        }
    }
}
