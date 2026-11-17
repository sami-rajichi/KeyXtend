//! What the engine needs of a speech library, so its rules are tested with a fake: its parts started once, the devices,
//! a file's languages, a model loaded on a place with its threads, and runs that a `Stop` aborts.

use kx_ipc::{DeviceKind, VoiceFault};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;
use zeroize::Zeroizing;

/// Where a model runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// The processor.
    Cpu,
    /// The card the library names with this id.
    Card(String),
}

impl Place {
    /// True for a card.
    pub fn card(&self) -> bool {
        matches!(self, Self::Card(_))
    }
}

/// A device the library found, with its memory in bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// The processor, a card or built-in graphics.
    pub kind: DeviceKind,
    /// The library's id for it, such as `Vulkan0`.
    pub id: String,
    /// Its name as the system gives it.
    pub name: String,
    /// All its memory.
    pub total: u64,
    /// Its free memory now.
    pub free: u64,
}

/// Why a library call failed; the library's own words never leave it, as they may name a path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Failed {
    /// The file is missing.
    NoFile,
    /// The file is no model the library reads.
    BadFile,
    /// Memory ran out.
    Memory,
    /// The device failed.
    Device,
    /// The model does not know the language.
    Language,
    /// The run's `Stop` aborted it.
    Stopped,
    /// Any other failure.
    Other,
}

impl Failed {
    /// The job's fault for this failure on the processor.
    pub fn fault(self) -> VoiceFault {
        match self {
            Self::NoFile => VoiceFault::NoFile,
            Self::BadFile => VoiceFault::BadFile,
            Self::Memory => VoiceFault::Memory,
            Self::Language => VoiceFault::Language,
            Self::Device | Self::Stopped | Self::Other => VoiceFault::Internal,
        }
    }

    /// True for a failure a card may cause and the processor may not share, so the processor takes over.
    pub fn card(self) -> bool {
        matches!(self, Self::Device | Self::Memory | Self::Other)
    }

    /// The fault for this failure on a card: `CardFailed` when the card may be the cause.
    pub fn on_card(self) -> VoiceFault {
        if self.card() {
            VoiceFault::CardFailed
        } else {
            self.fault()
        }
    }

    /// The fault for a file that could not be read for its facts: `BadFile` unless it is missing or memory ran out.
    pub fn read(self) -> VoiceFault {
        match self {
            Self::NoFile => VoiceFault::NoFile,
            Self::Memory => VoiceFault::Memory,
            _ => VoiceFault::BadFile,
        }
    }
}

/// Whole milliseconds since `start`, at most `u32::MAX`.
pub fn ms_since(start: Instant) -> u32 {
    u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX)
}

/// A speech library: transcribe.cpp in the product.
pub trait Library: Send + 'static {
    /// The runner name the models table gives its files.
    const RUNNER: &'static str;
    /// A model loaded with its session.
    type Model: Loaded;

    /// Loads the library's parts once, before any other call; false when they cannot serve, such as with no processor
    /// part.
    fn start(&self) -> bool;

    /// The devices found now, with their free memory.
    fn devices(&self) -> Vec<Found>;

    /// The language codes the model in `path` names.
    ///
    /// # Errors
    /// Why the file could not be read.
    fn langs(&self, path: &Path) -> Result<Vec<String>, Failed>;

    /// Loads the model in `path` on `place`, its processor work on `threads` threads.
    ///
    /// # Errors
    /// Why it could not load there.
    fn load(&self, path: &Path, place: &Place, threads: usize) -> Result<Self::Model, Failed>;
}

/// A loaded model.
pub trait Loaded: Send {
    /// The language codes it knows; empty when it names none.
    fn langs(&self) -> &[String];

    /// The longest sound one run takes, in samples; `None` for no limit.
    fn most(&self) -> Option<usize>;

    /// True when it knows `lang`, or names no language at all.
    fn knows(&self, lang: &str) -> bool {
        let langs = self.langs();
        langs.is_empty() || langs.iter().any(|l| l == lang)
    }

    /// The text of `pcm`, 16 kHz mono floats, in language `lang`; `stop` aborts it.
    ///
    /// # Errors
    /// `Stopped` when `stop` was set, else why it failed.
    fn run(&mut self, pcm: &[f32], lang: &str, stop: &Stop) -> Result<Zeroizing<String>, Failed>;
}

/// What a set `Stop` calls to abort the library's run in flight.
type Hook = Box<dyn Fn() + Send>;

/// Stops one run's writing: set when its writer drops, it also calls the hook the run in flight left.
#[derive(Clone, Default)]
pub struct Stop(Arc<Inner>);

#[derive(Default)]
struct Inner {
    set: AtomicBool,
    hook: Mutex<Option<Hook>>,
}

impl Stop {
    /// Sets it and aborts the run in flight.
    pub fn set(&self) {
        self.0.set.store(true, Ordering::SeqCst);
        if let Some(hook) = self.hook().as_ref() {
            hook();
        }
    }

    /// True once set.
    pub fn is_set(&self) -> bool {
        self.0.set.load(Ordering::SeqCst)
    }

    /// Leaves `hook` for a run starting now; it is called at once when already set.
    pub fn arm(&self, hook: Hook) {
        let mut slot = self.hook();
        if self.is_set() {
            hook();
        }
        *slot = Some(hook);
    }

    /// Drops the hook once the run ended.
    pub fn disarm(&self) {
        *self.hook() = None;
    }

    /// True when `other` is this very stop, so a command is known as its run's.
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    fn hook(&self) -> MutexGuard<'_, Option<Hook>> {
        self.0.hook.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn a_stop_calls_the_armed_hook_once_set_and_never_after_disarming() {
        let (stop, calls) = (Stop::default(), Arc::new(AtomicUsize::new(0)));
        let count = Arc::clone(&calls);
        stop.arm(Box::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
        }));
        assert!(!stop.is_set());
        stop.clone().set();
        assert!(stop.is_set());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        stop.disarm();
        stop.set();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_hook_armed_after_the_stop_runs_at_once_and_clones_are_the_same() {
        let (stop, calls) = (Stop::default(), Arc::new(AtomicUsize::new(0)));
        stop.set();
        let count = Arc::clone(&calls);
        stop.arm(Box::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(stop.same(&stop.clone()));
        assert!(!stop.same(&Stop::default()));
        assert!(Place::Card("Vulkan0".into()).card() && !Place::Cpu.card());
    }

    #[test]
    fn each_failure_has_its_fault_on_the_processor_on_a_card_and_when_read() {
        use Failed::{BadFile, Device, Language, Memory, NoFile, Other, Stopped};
        use VoiceFault::{BadFile as Bad, CardFailed as Card, Internal as Int, Memory as Mem};
        let want = [
            (
                NoFile,
                VoiceFault::NoFile,
                VoiceFault::NoFile,
                VoiceFault::NoFile,
            ),
            (BadFile, Bad, Bad, Bad),
            (Memory, Mem, Card, Mem),
            (Device, Int, Card, Bad),
            (Language, VoiceFault::Language, VoiceFault::Language, Bad),
            (Stopped, Int, Int, Bad),
            (Other, Int, Card, Bad),
        ];
        for (why, cpu, card, read) in want {
            assert_eq!(
                (why.fault(), why.on_card(), why.read()),
                (cpu, card, read),
                "{why:?}"
            );
        }
    }
}
