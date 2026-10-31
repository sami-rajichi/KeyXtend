//! G20's worker check (spec §7, ADR-0034): every `keyxtend-worker` a uiAccess product started must run without uiAccess
//! and at most at Medium integrity. A product without uiAccess, such as the test build, skips it with the reason.
#![cfg(windows)]

use std::ffi::c_void;
use std::path::Path;

use kx_platform_windows::front::path_of;
use kx_platform_windows::worker::WORKER_EXE;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, IsValidSid,
    SECURITY_MAX_SID_SIZE, TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TokenIntegrityLevel,
};
use windows::Win32::System::ProcessStatus::EnumProcesses;
use windows::Win32::System::SystemServices::SECURITY_MANDATORY_MEDIUM_RID;
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::apps::{Ctx, same_name};
use crate::g3::ui_access;
use crate::product::{self, NAME};
use crate::win::size32;

/// The name that runs the worker check alone, on the running product.
pub const WORKER: &str = "worker";
/// The check's key for the product's own rights.
pub const PRODUCT: &str = "product";
/// The integrity level's last SID part a worker may run at, at most: Medium, S-1-16-8192.
pub const MEDIUM: u32 = SECURITY_MANDATORY_MEDIUM_RID.cast_unsigned();
/// The process ids the first look reads; it doubles until every running process fits.
const FIRST_PIDS: usize = 1024;
/// A SID buffer of the largest size, in the 32-bit parts a SID is aligned to.
const SID_WORDS: usize = SECURITY_MAX_SID_SIZE as usize / size_of::<u32>();

/// What a process's token allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rights {
    /// It has uiAccess.
    pub ui_access: bool,
    /// Its integrity level's last SID part.
    pub level: u32,
}

/// The check's outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The product has no uiAccess, so there is nothing a worker could inherit.
    Skipped,
    /// The product has uiAccess, but no worker runs beside it.
    NoWorker,
    /// Every worker runs without uiAccess, at most at Medium.
    Reduced,
    /// A worker keeps uiAccess or runs above Medium.
    Broad,
}

impl Verdict {
    /// True only when every worker was checked and passed.
    pub fn ok(self) -> bool {
        self == Self::Reduced
    }

    /// True only when a worker was found with too many rights.
    pub fn fails(self) -> bool {
        self == Self::Broad
    }

    /// Why, for the result line.
    pub fn why(self) -> &'static str {
        match self {
            Self::Skipped => {
                "the product runs without uiAccess, so its worker cannot inherit it: check the dev install"
            }
            Self::NoWorker => {
                "no keyxtend-worker runs beside the product: open the panel on a picture so one starts"
            }
            Self::Reduced => "every worker runs without uiAccess, at most at Medium integrity",
            Self::Broad => "a worker keeps uiAccess or runs above Medium integrity",
        }
    }
}

/// The verdict on the `workers` of a product whose token gives `product`: a worker passes without uiAccess at Medium
/// integrity or lower, and fails with uiAccess or above Medium, such as High or System; less privilege is safe.
pub fn verdict(product: Rights, workers: &[Rights]) -> Verdict {
    let broad = |w: &Rights| w.ui_access || w.level > MEDIUM;
    match (product.ui_access, workers) {
        (false, _) => Verdict::Skipped,
        (true, []) => Verdict::NoWorker,
        (true, w) if w.iter().any(broad) => Verdict::Broad,
        (true, _) => Verdict::Reduced,
    }
}

/// The processes of `found`, by id and program path, that run the worker program in folder `dir`.
pub fn beside(found: &[(u32, String)], dir: &Path) -> Vec<u32> {
    let ours = |path: &str| {
        let p = Path::new(path);
        let folder = p
            .parent()
            .is_some_and(|f| same_name(&f.to_string_lossy(), &dir.to_string_lossy()));
        let file = p
            .file_name()
            .is_some_and(|n| same_name(&n.to_string_lossy(), WORKER_EXE));
        folder && file
    };
    found
        .iter()
        .filter(|(_, path)| ours(path))
        .map(|(pid, _)| *pid)
        .collect()
}

/// An integrity label with room for its SID behind it, as `TokenIntegrityLevel` writes it.
#[repr(C)]
#[derive(Default)]
struct Label {
    label: TOKEN_MANDATORY_LABEL,
    sid: [u32; SID_WORDS],
}

/// The integrity level's last SID part of process `pid`.
#[allow(
    unsafe_code,
    reason = "Both handles are ours and closed after use; Windows fills our own buffer."
)]
fn level(pid: u32) -> Result<u32, String> {
    let (mut label, mut len, mut token) = (Label::default(), 0u32, HANDLE::default());
    // SAFETY: both handles are ours and closed before returning; `label` holds the bytes the call may write and lives
    // past the SID reads, which point into it; the SID is checked before its parts are read, the last within its count.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|e| format!("OpenProcess {pid}: {e}"))?;
        let opened = OpenProcessToken(process, TOKEN_QUERY, &raw mut token);
        let _ = CloseHandle(process);
        opened.map_err(|e| format!("OpenProcessToken: {e}"))?;
        let buf = Some(std::ptr::from_mut(&mut label).cast::<c_void>());
        let read = GetTokenInformation(
            token,
            TokenIntegrityLevel,
            buf,
            size32::<Label>(),
            &raw mut len,
        );
        let _ = CloseHandle(token);
        read.map_err(|e| format!("GetTokenInformation: {e}"))?;
        let sid = label.label.Label.Sid;
        let bad = || format!("process {pid} has no readable integrity label");
        if !IsValidSid(sid).as_bool() {
            return Err(bad());
        }
        let last = u32::from(*GetSidSubAuthorityCount(sid))
            .checked_sub(1)
            .ok_or_else(bad)?;
        Ok(*GetSidSubAuthority(sid, last))
    }
}

/// What the token of process `pid` allows.
fn rights(pid: u32) -> Result<Rights, String> {
    Ok(Rights {
        ui_access: ui_access(pid)?,
        level: level(pid)?,
    })
}

/// The rights of each worker in `pids` as `read` gives them. One `read` fails on is skipped once `runs` says it ended,
/// as an idle worker stops; while it still runs, its error fails the check.
fn read_each(
    pids: Vec<u32>,
    read: impl Fn(u32) -> Result<Rights, String>,
    runs: impl Fn(u32) -> bool,
) -> Result<Vec<Rights>, String> {
    let mut found = Vec::new();
    for pid in pids {
        match read(pid) {
            Ok(r) => found.push(r),
            Err(_) if !runs(pid) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(found)
}

/// Every running process's id and program path, as far as Windows names it.
#[allow(unsafe_code, reason = "Windows fills our own buffer of process ids.")]
fn processes() -> Vec<(u32, String)> {
    let mut pids = vec![0u32; FIRST_PIDS];
    loop {
        let (cap, mut used) = (
            size32::<u32>().saturating_mul(u32::try_from(pids.len()).unwrap_or(u32::MAX)),
            0u32,
        );
        // SAFETY: `pids` holds `cap` bytes, which the call writes no further than.
        if unsafe { EnumProcesses(pids.as_mut_ptr(), cap, &raw mut used) }.is_err() {
            return Vec::new();
        }
        if used < cap {
            pids.truncate(used as usize / size_of::<u32>());
            return pids
                .into_iter()
                .filter_map(|p| path_of(p).map(|path| (p, path)))
                .collect();
        }
        pids.resize(pids.len() * 2, 0);
    }
}

/// The worker check on the product whose process is `pid` and program `exe`, as a result part.
pub fn check(pid: u32, exe: &Path) -> Value {
    let read = || -> Result<(Rights, Vec<Rights>), String> {
        let dir = exe.parent().ok_or("the product's program has no folder")?;
        let product = rights(pid)?;
        let runs = |w: u32| beside(&processes(), dir).contains(&w);
        Ok((product, read_each(beside(&processes(), dir), rights, runs)?))
    };
    match read() {
        Ok((product, workers)) => {
            let v = verdict(product, &workers);
            json!({ PRODUCT: product, "workers": workers, "ok": v.ok(),
                "fails": v.fails(), "why": v.why() })
        }
        Err(e) => json!({ "error": e, "ok": false, "fails": true }),
    }
}

/// The product's rights as the check's `part` read them; none when it could not read them.
pub fn product_of(part: &Value) -> Option<Rights> {
    Rights::deserialize(&part[PRODUCT]).ok()
}

/// Runs `g20 worker` on the running product, which only reads tokens: it passes once every worker checked passed.
pub fn run(_ctx: &Ctx) -> Result<Value, String> {
    let (pid, exe) =
        product::running().ok_or_else(|| format!("no {NAME} runs: start the dev install first"))?;
    let part = check(pid, Path::new(&exe));
    Ok(json!({ "gate": "G20", "name": WORKER, "worker": part, "pass": part["ok"] == true }))
}

#[cfg(test)]
mod tests;
