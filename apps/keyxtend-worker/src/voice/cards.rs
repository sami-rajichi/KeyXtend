//! The devices (ADR-0044): the list the host gets, the fit check that refuses a card short of free memory before any
//! load, and the processor's real cores, which `threads` 0 means and nothing passes (16 threads ran 3–8× slower than 8).

use super::library::{Found, Place};
use kx_ipc::{
    Device, DeviceKind, MAX_DEVICES, MAX_NAME_BYTES, MAX_TAG_BYTES, Target, VoiceFault, VoiceNote,
};
use std::num::NonZero;
use std::thread;

/// Bytes in one MB of the frames' memory figures.
const MB: u64 = 1 << 20;
/// The CPUID leaf that tells the processor's topology, level by level.
const TOPOLOGY_LEAF: u32 = 0xB;
/// The CPUID leaf of extended features, and its EDX bit set on a hybrid processor (Intel's P and E cores).
const FEATURE_LEAF: u32 = 7;
const HYBRID_BIT: u32 = 15;
/// The threads per core assumed on a hybrid processor, whose leaf answers for the core it runs on alone.
const HYBRID_SMT: usize = 2;
/// That leaf's level type for the threads that share one core.
const SMT_LEVEL: u32 = 1;
/// Where the level type sits in ECX, and its mask.
const LEVEL_SHIFT: u32 = 8;
const LEVEL_MASK: u32 = 0xFF;
/// The mask of EBX's count of logical processors at that level.
const COUNT_MASK: u32 = 0xFFFF;

/// The processor threads a model runs on: `asked`, or the real cores for 0, and never more than them.
pub fn threads(asked: u8, cores: usize) -> usize {
    let cores = cores.max(1);
    match usize::from(asked) {
        0 => cores,
        n => n.min(cores),
    }
}

/// The processor's real cores: its logical processors less the extra threads each core runs. On a hybrid processor
/// it is half its threads, which never counts a core's second thread; the host's exact count comes in P11 task 14.
pub fn cores() -> usize {
    let logical = thread::available_parallelism().map_or(1, NonZero::get);
    (logical / per_core()).max(1)
}

/// The threads one core runs, from CPUID's topology leaf on x86-64.
#[cfg(target_arch = "x86_64")]
fn per_core() -> usize {
    use std::arch::x86_64::{__cpuid, __cpuid_count};
    let top = __cpuid(0).eax;
    let hybrid = top >= FEATURE_LEAF && (__cpuid_count(FEATURE_LEAF, 0).edx >> HYBRID_BIT) & 1 == 1;
    let smt = if top >= TOPOLOGY_LEAF {
        let level = __cpuid_count(TOPOLOGY_LEAF, 0);
        smt_width(level.ebx, level.ecx)
    } else {
        1
    };
    width(smt, hybrid)
}

/// The threads per core to divide by: the core's own reading, or at least `HYBRID_SMT` on a hybrid processor, where
/// a reading taken on an efficient core would count every thread as a core.
fn width(smt: usize, hybrid: bool) -> usize {
    if hybrid { smt.max(HYBRID_SMT) } else { smt }
}

/// Other processors are counted one thread a core.
#[cfg(not(target_arch = "x86_64"))]
fn per_core() -> usize {
    1
}

/// The threads per core that the topology leaf's first level tells, or 1 when it is no thread level.
fn smt_width(ebx: u32, ecx: u32) -> usize {
    let count = usize::try_from(ebx & COUNT_MASK).unwrap_or(1);
    let smt = (ecx >> LEVEL_SHIFT) & LEVEL_MASK == SMT_LEVEL;
    if smt { count.max(1) } else { 1 }
}

/// Where `target` runs among `found`, refused before any load: a gone card is `NoCard` and a card whose free memory
/// now is under its need is `CardRoom`, and the processor serves either way.
pub fn place(target: &Target<'_>, found: &[Found]) -> (Place, Option<VoiceFault>) {
    let Target::Card { id, need_mb } = target else {
        return (Place::Cpu, None);
    };
    let card = found
        .iter()
        .find(|f| f.kind != DeviceKind::Cpu && f.id == **id);
    match card {
        None => (Place::Cpu, Some(VoiceFault::NoCard)),
        Some(card) if mb(card.free) < *need_mb => (Place::Cpu, Some(VoiceFault::CardRoom)),
        Some(_) => (Place::Card(id.to_string()), None),
    }
}

/// The host's device list: the processor first, then each card, ids that the frames take and names cut to fit.
pub fn list(found: &[Found]) -> Vec<Device> {
    let (cpu, cards): (Vec<&Found>, Vec<&Found>) =
        found.iter().partition(|f| f.kind == DeviceKind::Cpu);
    cpu.into_iter()
        .take(1)
        .chain(cards)
        .filter(|f| tag(&f.id))
        .take(MAX_DEVICES)
        .map(device)
        .collect()
}

/// One device as a frame, shown by its id when the frames refuse its name; the engine reports no driver version.
fn device(f: &Found) -> Device {
    let total_mb = mb(f.total);
    let mut d = Device {
        kind: f.kind,
        id: f.id.clone(),
        name: shown(&f.name),
        total_mb,
        free_mb: mb(f.free).min(total_mb),
        driver: String::new(),
    };
    let list = vec![d.clone()];
    if (VoiceNote::Devices { id: 0, list }).check().is_err() {
        d.name.clone_from(&d.id);
    }
    d
}

/// True for an id or a language code the frames take: visible ASCII, within the tag's length.
pub fn tag(id: &str) -> bool {
    (1..=MAX_TAG_BYTES).contains(&id.len()) && kx_ipc::rules::tag(id).is_ok()
}

/// `name` without control characters, cut at a character boundary to fit a shown name.
fn shown(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars().filter(|c| !c.is_control()) {
        if out.len() + c.len_utf8() > MAX_NAME_BYTES {
            break;
        }
        out.push(c);
    }
    out
}

/// `bytes` in whole MB, at most `u32::MAX`.
fn mb(bytes: u64) -> u32 {
    u32::try_from(bytes / MB).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
