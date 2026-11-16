//! G8's own weak-mode job, copied from the spike's (gone in P11): a Windows job its copy dies with, capped in weak mode
//! in cores, CPU time and memory, from `[g8.weak]`.
#![cfg(windows)]

use serde::Deserialize;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_CPU_RATE_CONTROL_ENABLE,
    JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP, JOB_OBJECT_LIMIT_AFFINITY, JOB_OBJECT_LIMIT_JOB_MEMORY,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_CPU_RATE_CONTROL_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOBOBJECTINFOCLASS, JobObjectCpuRateControlInformation,
    JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows::core::PCWSTR;

/// Bytes in a MiB.
const MIB: usize = 1 << 20;
/// The job's CPU rate is in hundredths of a percent.
const RATE_PER_PCT: u32 = 100;
/// Highest CPU time per allowed core, in percent of one core.
const MAX_PCT: u32 = 100;

/// Weak mode's caps, as `[g8.weak]` gives them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weak {
    /// Cores the copy may use.
    pub cores: u32,
    /// CPU time per allowed core, in percent of one core: the job gets `cores × rate_pct` % of one core in all.
    pub rate_pct: u32,
    /// Memory cap, in MiB.
    pub mem_mb: u64,
}

/// Weak-mode limits in the units the job takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps {
    /// One bit per allowed core, lowest cores first.
    pub affinity: usize,
    /// CPU share of the whole PC, in hundredths of a percent.
    pub cpu_rate: u32,
    /// Memory cap for the whole job.
    pub mem_bytes: usize,
}

/// The job rate on a PC with `cpus` cores that allows `pct` % of one core per allowed core, `cores` of them; at least 1.
fn rate_of(pct: u32, cores: usize, cpus: usize) -> u32 {
    let whole = u128::from(pct * RATE_PER_PCT) * cores as u128 / cpus.max(1) as u128;
    u32::try_from(whole).unwrap_or(u32::MAX).max(1)
}

impl Caps {
    /// The CPU time the job allows on a PC with `cpus` cores, in whole cores.
    #[allow(clippy::cast_precision_loss, reason = "Core counts are tiny.")]
    pub fn cpu_cores(&self, cpus: usize) -> f64 {
        f64::from(self.cpu_rate) * cpus as f64 / f64::from(RATE_PER_PCT * MAX_PCT)
    }

    /// Checks `w` and converts it for a PC with `cpus` logical cores.
    pub fn from(w: &Weak, cpus: usize) -> Result<Caps, String> {
        let bits = usize::BITS as usize;
        let cores = (w.cores as usize).min(cpus).min(bits);
        if cores == 0 {
            return Err(format!(
                "weak mode needs at least one core, got {}",
                w.cores
            ));
        }
        if w.mem_mb == 0 {
            return Err("weak mode needs a memory cap above 0 MiB".to_string());
        }
        if w.rate_pct == 0 || w.rate_pct > MAX_PCT {
            return Err(format!(
                "weak rate_pct must be 1 to {MAX_PCT} % of a core, got {}",
                w.rate_pct
            ));
        }
        let affinity = if cores == bits {
            usize::MAX
        } else {
            (1 << cores) - 1
        };
        let mem_bytes = usize::try_from(w.mem_mb)
            .unwrap_or(usize::MAX)
            .saturating_mul(MIB);
        Ok(Caps {
            affinity,
            cpu_rate: rate_of(w.rate_pct, cores, cpus),
            mem_bytes,
        })
    }
}

/// An open job; closing it ends every process in it.
pub struct Job(HANDLE);

// SAFETY: a job handle is a kernel handle that any thread may use or close.
#[allow(unsafe_code, reason = "A kernel handle any thread may use.")]
unsafe impl Send for Job {}

impl Job {
    /// A job that kills its processes when closed, capped when `caps` is given.
    #[allow(unsafe_code, reason = "Makes a job object of ours.")]
    pub fn new(caps: Option<&Caps>) -> Result<Job, String> {
        // SAFETY: no security attributes and no name; the handle is owned by the returned Job.
        let job =
            Job(unsafe { CreateJobObjectW(None, PCWSTR::null()) }.map_err(|e| e.to_string())?);
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if let Some(c) = caps {
            limits.BasicLimitInformation.LimitFlags |=
                JOB_OBJECT_LIMIT_AFFINITY | JOB_OBJECT_LIMIT_JOB_MEMORY;
            limits.BasicLimitInformation.Affinity = c.affinity;
            limits.JobMemoryLimit = c.mem_bytes;
        }
        job.set(JobObjectExtendedLimitInformation, &limits)?;
        if let Some(c) = caps {
            let mut rate = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION {
                ControlFlags: JOB_OBJECT_CPU_RATE_CONTROL_ENABLE
                    | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
                ..Default::default()
            };
            rate.Anonymous.CpuRate = c.cpu_rate;
            job.set(JobObjectCpuRateControlInformation, &rate)?;
        }
        Ok(job)
    }

    /// Puts the process behind `process` in the job; the handle needs the set-quota and terminate rights.
    #[allow(unsafe_code, reason = "Joins a process of ours to our job.")]
    pub fn assign_handle(&self, process: HANDLE) -> Result<(), String> {
        // SAFETY: the job handle is ours and open; the caller keeps `process` open for the call.
        unsafe { AssignProcessToJobObject(self.0, process) }.map_err(|e| format!("job: {e}"))
    }

    #[allow(unsafe_code, reason = "Sets our own job's limits.")]
    fn set<T>(&self, class: JOBOBJECTINFOCLASS, info: &T) -> Result<(), String> {
        let size = u32::try_from(size_of::<T>()).map_err(|e| e.to_string())?;
        // SAFETY: `info` points to a live `T` of `size` bytes that matches `class`.
        unsafe { SetInformationJobObject(self.0, class, std::ptr::from_ref(info).cast(), size) }
            .map_err(|e| format!("job limits: {e}"))
    }
}

impl Drop for Job {
    #[allow(unsafe_code, reason = "Closes our own job handle.")]
    fn drop(&mut self) {
        // SAFETY: the handle came from CreateJobObjectW and is closed once, here.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[cfg(test)]
mod tests;
