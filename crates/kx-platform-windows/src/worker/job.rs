//! The worker's job object: it ends the worker when its last handle closes, caps its memory, lets it start no process
//! of its own, bars it from shared UI, and ends it at once on a crash instead of showing an error box. The gate tool's
//! weak mode also caps its processor time to a weak PC's few slow cores, before the worker's first instruction.

use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::thread;

use kx_platform::worker::{
    FetchFrames, Frames, Launch, MediaFrames, NetFrames, PickFrames, RecordFrames, SpawnError,
    VaultFrames, VoiceFrames,
};
use windows::Win32::Foundation::{ERROR_PROCESS_ABORTED, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_CPU_RATE_CONTROL,
    JOB_OBJECT_CPU_RATE_CONTROL_ENABLE, JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP, JOB_OBJECT_LIMIT,
    JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOB_OBJECT_UILIMIT,
    JOB_OBJECT_UILIMIT_DESKTOP, JOB_OBJECT_UILIMIT_DISPLAYSETTINGS, JOB_OBJECT_UILIMIT_EXITWINDOWS,
    JOB_OBJECT_UILIMIT_GLOBALATOMS, JOB_OBJECT_UILIMIT_HANDLES, JOB_OBJECT_UILIMIT_READCLIPBOARD,
    JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS, JOB_OBJECT_UILIMIT_WRITECLIPBOARD,
    JOBOBJECT_BASIC_UI_RESTRICTIONS, JOBOBJECT_CPU_RATE_CONTROL_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOBOBJECTINFOCLASS, JobObjectBasicUIRestrictions,
    JobObjectCpuRateControlInformation, JobObjectExtendedLimitInformation,
    QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
};
use windows::Win32::System::Threading::{ALL_PROCESSOR_GROUPS, GetActiveProcessorCount};
use windows::core::PCWSTR;

use super::{WinChild, raw, refused, size32};

/// Bytes in one MB, as the settings count it.
pub(super) const MIB: usize = 1024 * 1024;
/// The worker runs alone: the job takes no second process.
pub(super) const ALONE: u32 = 1;
/// The exit code of a worker the job ends: "the process terminated unexpectedly".
pub(super) const ENDED: u32 = ERROR_PROCESS_ABORTED.0;
/// The limits every worker's job sets.
pub(super) const LIMITS: JOB_OBJECT_LIMIT = JOB_OBJECT_LIMIT(
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE.0
        | JOB_OBJECT_LIMIT_PROCESS_MEMORY.0
        | JOB_OBJECT_LIMIT_ACTIVE_PROCESS.0
        | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION.0,
);
/// The shared UI the worker may not touch: other programs' windows, the clipboard, global atoms, desktops, display
/// and system settings, and signing out; it draws pixels and needs none of them.
pub(super) const UI: JOB_OBJECT_UILIMIT = JOB_OBJECT_UILIMIT(
    JOB_OBJECT_UILIMIT_HANDLES.0
        | JOB_OBJECT_UILIMIT_READCLIPBOARD.0
        | JOB_OBJECT_UILIMIT_WRITECLIPBOARD.0
        | JOB_OBJECT_UILIMIT_GLOBALATOMS.0
        | JOB_OBJECT_UILIMIT_DESKTOP.0
        | JOB_OBJECT_UILIMIT_DISPLAYSETTINGS.0
        | JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS.0
        | JOB_OBJECT_UILIMIT_EXITWINDOWS.0,
);
/// Each job's shared-UI limits by its word on the command line: every job is barred from all of it, as the record
/// job's capture and the pick job's dialogs proved to need none. Memory comes from each `Launch`, the processor cap
/// from the spawner's weak mode.
pub(super) const JOBS_UI: [(&str, JOB_OBJECT_UILIMIT); 7] = [
    (MediaFrames::JOB, UI),
    (VaultFrames::JOB, UI),
    (VoiceFrames::JOB, UI),
    (FetchFrames::JOB, UI),
    (NetFrames::JOB, UI),
    (RecordFrames::JOB, UI),
    (PickFrames::JOB, UI),
];
/// Windows counts a job's processor rate in hundredths of a percent of the whole PC.
const RATE_PER_PCT: u32 = 100;
/// Weak mode's rate control: on, as a hard cap, so the job never borrows idle time.
const HARD_CAP: JOB_OBJECT_CPU_RATE_CONTROL = JOB_OBJECT_CPU_RATE_CONTROL(
    JOB_OBJECT_CPU_RATE_CONTROL_ENABLE.0 | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP.0,
);

/// A weak PC's processor, as the gate tool's weak mode imitates it: `cores` cores, each at `pct` percent of one here.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CpuCap {
    /// The weak PC's cores.
    pub cores: usize,
    /// Each core's speed, in percent of one of this PC's.
    pub pct: u8,
}

impl CpuCap {
    /// One core at its full speed, in percent; a higher `pct` counts as this.
    pub const FULL_PCT: u8 = 100;
}

/// The rate of `cap` as Windows' share of a PC of `logical` processors. It counts one core at least and none past the
/// PC's, and is never 0, which Windows refuses; the wide sum never overflows.
pub(super) fn share(cap: CpuCap, logical: usize) -> u32 {
    let logical = logical.max(1);
    let wide = |n: usize| u128::try_from(n).unwrap_or(u128::MAX);
    let rate = u128::from(u32::from(cap.pct.min(CpuCap::FULL_PCT)) * RATE_PER_PCT);
    let part = rate.saturating_mul(wide(cap.cores.clamp(1, logical))) / wide(logical);
    u32::try_from(part).map_or(u32::MAX, |p| p.max(1))
}

/// Every active processor of the PC, in every group, which Windows counts a job's rate against; std's count if unread.
#[allow(unsafe_code, reason = "Counts the PC's processors.")]
pub(super) fn logical() -> usize {
    // SAFETY: a plain query that takes a number and returns one.
    let all = unsafe { GetActiveProcessorCount(ALL_PROCESSOR_GROUPS) };
    match usize::try_from(all) {
        Ok(n) if n > 0 => n,
        _ => thread::available_parallelism().map_or(1, usize::from),
    }
}

/// The shared-UI limits of the job `launch` starts; one the table lacks is barred from all of it.
pub(super) fn ui_of(launch: &Launch) -> JOB_OBJECT_UILIMIT {
    let word = launch.args.first().map(String::as_str);
    let listed = JOBS_UI.iter().find(|(job, _)| Some(*job) == word);
    listed.map_or(UI, |&(_, ui)| ui)
}

/// An unnamed job; closing it ends the processes in it.
pub(super) struct Job(OwnedHandle);

impl Job {
    /// A job for one worker of at most `memory_mb` MB, barred from the shared UI `ui` names, its job's `ui_of`.
    #[allow(unsafe_code, reason = "Makes a job of our own.")]
    pub(super) fn new(memory_mb: u32, ui: JOB_OBJECT_UILIMIT) -> Result<Self, SpawnError> {
        // SAFETY: no attributes and no name, so no other program can open it.
        let made =
            unsafe { CreateJobObjectW(None, PCWSTR::null()) }.map_err(refused("make a job"))?;
        // SAFETY: Windows just made this handle, and only the job closes it.
        let job = Self(unsafe { OwnedHandle::from_raw_handle(made.0) });
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = LIMITS;
        limits.BasicLimitInformation.ActiveProcessLimit = ALONE;
        let mb = usize::try_from(memory_mb).unwrap_or(usize::MAX);
        limits.ProcessMemoryLimit = mb.saturating_mul(MIB);
        job.set(JobObjectExtendedLimitInformation, &limits, "cap the job")?;
        let ui = JOBOBJECT_BASIC_UI_RESTRICTIONS {
            UIRestrictionsClass: ui,
        };
        job.set(
            JobObjectBasicUIRestrictions,
            &ui,
            "bar the job from shared UI",
        )?;
        Ok(job)
    }

    /// Sets `value`, the struct that `class` names, on the job.
    #[allow(unsafe_code, reason = "Sets a limit on our own job.")]
    fn set<T>(
        &self,
        class: JOBOBJECTINFOCLASS,
        value: &T,
        step: &'static str,
    ) -> Result<(), SpawnError> {
        let info = std::ptr::from_ref(value).cast();
        // SAFETY: `info` points to a live `T` of the size passed; Windows refuses a size `class` does not take.
        unsafe { SetInformationJobObject(self.handle(), class, info, size32::<T>()) }
            .map_err(refused(step))
    }

    /// Puts `process`, still suspended, in the job.
    #[allow(unsafe_code, reason = "Puts a process of ours in our job.")]
    pub(super) fn assign(&self, process: HANDLE) -> Result<(), SpawnError> {
        // SAFETY: both handles are ours and open for the call.
        unsafe { AssignProcessToJobObject(self.handle(), process) }
            .map_err(refused("put the worker in its job"))
    }

    /// Ends every process in the job with `ENDED`; harmless once they have ended.
    #[allow(unsafe_code, reason = "Ends the processes of our own job.")]
    pub(super) fn end(&self) {
        // SAFETY: our live job handle; a job with no process left is no error worth telling.
        let _ = unsafe { TerminateJobObject(self.handle(), ENDED) };
    }

    /// The most memory its process has taken, in MB rounded up; `None` when Windows does not tell.
    pub(super) fn peak_mb(&self) -> Option<u32> {
        let info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION =
            self.query(JobObjectExtendedLimitInformation)?;
        u32::try_from(info.PeakProcessMemoryUsed.div_ceil(MIB)).ok()
    }

    /// Caps the job's processor time to `cap`'s share of this PC, as a hard cap; the start sets it before the worker runs.
    pub(super) fn weaken(&self, cap: CpuCap) -> Result<(), SpawnError> {
        let mut info = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION {
            ControlFlags: HARD_CAP,
            ..Default::default()
        };
        info.Anonymous.CpuRate = share(cap, logical());
        let class = JobObjectCpuRateControlInformation;
        self.set(class, &info, "cap the job's processor time")
    }

    /// The job's hard processor cap, in hundredths of a percent of the whole PC; `None` without one.
    #[allow(unsafe_code, reason = "Reads a plain number from a union.")]
    fn cpu_cap(&self) -> Option<u32> {
        let cap: JOBOBJECT_CPU_RATE_CONTROL_INFORMATION =
            self.query(JobObjectCpuRateControlInformation)?;
        // SAFETY: every member of the union is plain integers over the same four bytes, so any bits read as a rate.
        let rate = unsafe { cap.Anonymous.CpuRate };
        (cap.ControlFlags.0 & HARD_CAP.0 == HARD_CAP.0).then_some(rate)
    }

    /// The plain struct `class` names, read from the job; `None` when Windows does not tell.
    #[allow(unsafe_code, reason = "Reads a struct of our own job.")]
    fn query<T: Copy + Default>(&self, class: JOBOBJECTINFOCLASS) -> Option<T> {
        let mut info = T::default();
        let (at, size) = ((&raw mut info).cast(), size32::<T>());
        // SAFETY: `at` points to a live `T` of the size passed, a plain Windows struct with no drop, which Windows
        // fills; it refuses a size `class` does not take.
        unsafe { QueryInformationJobObject(Some(self.handle()), class, at, size, None) }.ok()?;
        Some(info)
    }

    /// The job's handle, open while the job lives.
    pub(super) fn handle(&self) -> HANDLE {
        raw(&self.0)
    }
}

impl WinChild {
    /// The worker's processor cap, read back from its job in hundredths of a percent of the whole PC; `None` without
    /// one, as in normal mode.
    #[must_use]
    pub fn cpu_cap(&self) -> Option<u32> {
        self.job.cpu_cap()
    }
}

#[cfg(test)]
mod tests;
