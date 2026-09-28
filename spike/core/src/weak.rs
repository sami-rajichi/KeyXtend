//! A Windows job for a child process: it dies with us, and in weak mode it is capped in cores, CPU share and memory.

use std::os::windows::io::AsRawHandle;
use std::process::Child;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_CPU_RATE_CONTROL_ENABLE,
    JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP, JOB_OBJECT_LIMIT_AFFINITY, JOB_OBJECT_LIMIT_JOB_MEMORY,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_CPU_RATE_CONTROL_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOBOBJECTINFOCLASS, JobObjectCpuRateControlInformation,
    JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows::core::PCWSTR;

use crate::config::MIB;
use crate::voicecfg::WeakConfig;

/// The job's CPU rate is in hundredths of a percent.
const RATE_PER_PCT: u32 = 100;
/// Highest CPU share, in percent.
const MAX_PCT: u32 = 100;

/// Weak-mode limits in the units the job takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps {
    /// One bit per allowed core, lowest cores first.
    pub affinity: usize,
    /// CPU share in hundredths of a percent.
    pub cpu_rate: u32,
    /// Memory cap for the whole job.
    pub mem_bytes: usize,
}

impl Caps {
    /// Checks `w` and converts it for a PC with `cpus` logical cores.
    pub fn from(w: &WeakConfig, cpus: usize) -> Result<Caps, String> {
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
                "weak CPU share must be 1 to {MAX_PCT} %, got {}",
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
            cpu_rate: w.rate_pct * RATE_PER_PCT,
            mem_bytes,
        })
    }
}

/// An open job; closing it ends every process in it.
pub struct Job(HANDLE);

// SAFETY: a job handle is a kernel handle that any thread may use or close.
unsafe impl Send for Job {}

impl Job {
    /// A job that kills its processes when closed, capped when `caps` is given.
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

    /// Puts `child` in the job.
    pub fn assign(&self, child: &Child) -> Result<(), String> {
        let process = HANDLE(child.as_raw_handle());
        // SAFETY: both handles are open for the duration of the call; `child` still owns its handle.
        unsafe { AssignProcessToJobObject(self.0, process) }.map_err(|e| format!("job: {e}"))
    }

    fn set<T>(&self, class: JOBOBJECTINFOCLASS, info: &T) -> Result<(), String> {
        let size = u32::try_from(size_of::<T>()).map_err(|e| e.to_string())?;
        // SAFETY: `info` points to a live `T` of `size` bytes that matches `class`.
        unsafe { SetInformationJobObject(self.0, class, (info as *const T).cast(), size) }
            .map_err(|e| format!("job limits: {e}"))
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        // SAFETY: the handle came from CreateJobObjectW and is closed once, here.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voicecfg::WeakConfig;

    fn weak(cores: u32, rate_pct: u32) -> WeakConfig {
        WeakConfig {
            cores,
            rate_pct,
            mem_mb: 4096,
        }
    }

    #[test]
    fn caps_turn_settings_into_job_values() {
        let c = Caps::from(&weak(2, 30), 8).expect("valid");
        assert_eq!(c.affinity, 0b11);
        assert_eq!(c.cpu_rate, 3000, "hundredths of a percent");
        assert_eq!(c.mem_bytes, 4096 * 1024 * 1024);
    }

    #[test]
    fn more_cores_than_the_pc_has_means_all_of_them() {
        assert_eq!(
            Caps::from(&weak(16, 30), 4).expect("valid").affinity,
            0b1111
        );
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
    fn a_job_is_made_with_and_without_caps() {
        let cpus = std::thread::available_parallelism().map_or(1, usize::from);
        let caps = Caps::from(&weak(2, 30), cpus).expect("valid");
        assert!(Job::new(None).is_ok());
        assert!(Job::new(Some(&caps)).is_ok());
    }
}
