//! What G8 logs of the machine beside its numbers, as the owner runs WSL and Docker during gates on purpose: free RAM,
//! and the working set of each `[host] vm` process from Windows' performance counters, which need no handle to it.
#![cfg(windows)]

use serde::Serialize;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Performance::{
    PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PdhAddEnglishCounterW,
    PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterValue, PdhOpenQueryW,
};
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::core::{HSTRING, PCWSTR};

use crate::apps::Ctx;
use crate::proc::MB;

/// What the English counter path names around a process: its set, then its counter.
const COUNTER: [&str; 2] = [r"\Process(", r")\Working Set"];

/// One virtual machine process's memory; none when it does not run or cannot be read.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Vm {
    /// Its program name, from `[host] vm`.
    pub name: String,
    /// Its working set, in MB.
    pub mb: Option<f64>,
}

/// The machine's memory at one reading.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Host {
    /// Free physical memory, in MB.
    pub free_mb: Option<f64>,
    /// Each `[host] vm` process, in order.
    pub vm: Vec<Vm>,
}

/// The English performance counter path of process `name`'s working set.
fn counter(name: &str) -> String {
    let [set, value] = COUNTER;
    format!("{set}{name}{value}")
}

/// The machine's memory now.
pub fn read(ctx: &Ctx) -> Host {
    let vm = ctx.cfg.host.vm.iter().map(|name| Vm {
        name: name.clone(),
        mb: working_mb(name),
    });
    Host {
        free_mb: free_mb(),
        vm: vm.collect(),
    }
}

/// Free physical memory now, in MB.
#[allow(unsafe_code, reason = "A system memory read into a local.")]
#[allow(
    clippy::cast_precision_loss,
    reason = "Memory sizes are far below 2^53 bytes."
)]
fn free_mb() -> Option<f64> {
    let mut status = MEMORYSTATUSEX {
        dwLength: u32::try_from(size_of::<MEMORYSTATUSEX>()).ok()?,
        ..Default::default()
    };
    // SAFETY: the call fills a local whose length it was told.
    unsafe { GlobalMemoryStatusEx(&raw mut status) }.ok()?;
    Some(status.ullAvailPhys as f64 / MB)
}

/// Process `name`'s working set in MB from one read of its performance counter; none when it does not run. Only the
/// first process of that name reads, as the counter names the others `name#1` and on.
#[allow(unsafe_code, reason = "Performance counter calls on a query G8 owns.")]
fn working_mb(name: &str) -> Option<f64> {
    let path = HSTRING::from(counter(name));
    let mut query = PDH_HQUERY::default();
    // SAFETY: the calls fill locals and use the query only while it is open; it closes before the return.
    unsafe {
        if PdhOpenQueryW(PCWSTR::null(), 0, &raw mut query) != ERROR_SUCCESS.0 {
            return None;
        }
        let mut c = PDH_HCOUNTER::default();
        let mut value = PDH_FMT_COUNTERVALUE::default();
        let read = PdhAddEnglishCounterW(query, &path, 0, &raw mut c) == ERROR_SUCCESS.0
            && PdhCollectQueryData(query) == ERROR_SUCCESS.0
            && PdhGetFormattedCounterValue(c, PDH_FMT_DOUBLE, None, &raw mut value)
                == ERROR_SUCCESS.0;
        PdhCloseQuery(query);
        read.then(|| value.Anonymous.doubleValue / MB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vm_process_is_read_by_its_working_set_counter() {
        assert_eq!(counter("vmmemWSL"), r"\Process(vmmemWSL)\Working Set");
    }
}
