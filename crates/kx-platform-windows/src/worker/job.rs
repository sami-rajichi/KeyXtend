//! The worker's job object: it ends the worker when its last handle closes, caps its memory, lets it start no process
//! of its own, and ends it at once on a crash instead of showing an error box.

use std::os::windows::io::{FromRawHandle, OwnedHandle};

use kx_platform::pictures::SpawnError;
use windows::Win32::Foundation::{ERROR_PROCESS_ABORTED, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
    JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectExtendedLimitInformation, SetInformationJobObject, TerminateJobObject,
};
use windows::core::PCWSTR;

use super::{raw, refused, size32};

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

/// An unnamed job; closing it ends the processes in it.
pub(super) struct Job(OwnedHandle);

impl Job {
    /// A job for one worker of at most `memory_mb` MB.
    #[allow(unsafe_code, reason = "Makes and caps a job of our own.")]
    pub(super) fn new(memory_mb: u32) -> Result<Self, SpawnError> {
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
        let size = size32::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>();
        let info = (&raw const limits).cast();
        // SAFETY: `info` points to a live struct of `size` bytes that matches the class.
        unsafe {
            SetInformationJobObject(job.handle(), JobObjectExtendedLimitInformation, info, size)
        }
        .map_err(refused("cap the job"))?;
        Ok(job)
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

    /// The job's handle, open while the job lives.
    pub(super) fn handle(&self) -> HANDLE {
        raw(&self.0)
    }
}
