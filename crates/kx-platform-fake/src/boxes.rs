//! A fake boxes service with no real crypto: plainly fake padlocks, boxes that only a `FakeBoxes` opens and only with
//! the padlock's password, one failure on cue per call, `KeePass` entries and drops set per file and password, and
//! calls counted by kind.

use crate::lock;
use kx_module_api::ports::{
    BoxConfig, BoxError, BoxPart, BoxRun, Boxes, KeepassEntry, KeepassImport, MAX_BOX_WORK,
    MAX_KEEPASS_BYTES, MAX_PASSWORD_BYTES, MIN_BOX_WORK, Opened, Padlock,
};
use kx_module_api::{Redacted, Secret, Zeroizing};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

mod format;

/// One kind of box call, to fail or count.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BoxCall {
    /// `padlock`.
    Padlock,
    /// `seal`.
    Seal,
    /// `open`.
    Open,
    /// `check`.
    Check,
    /// `keepass`.
    Keepass,
}

impl BoxCall {
    /// Whether the port lets this call return `error`.
    fn returns(self, error: BoxError) -> bool {
        use BoxError::{Damaged, Failed, TimedOut, TooLarge};
        match self {
            Self::Padlock => matches!(error, TooLarge | TimedOut | Failed),
            Self::Seal => matches!(error, Damaged | TooLarge | TimedOut | Failed),
            Self::Check => matches!(error, Damaged | TimedOut | Failed),
            Self::Open | Self::Keepass => true,
        }
    }
}

/// A `KeePass` file the test set: its bytes, the password that opens it, if any, and what an import of it gives.
struct KeepassFile {
    bytes: Vec<u8>,
    password: Option<Zeroizing<String>>,
    import: KeepassImport,
}

/// What the sessions did, and what the test set for them.
#[derive(Default)]
struct Log {
    fails: HashMap<BoxCall, BoxError>,
    calls: HashMap<BoxCall, usize>,
    files: Vec<KeepassFile>,
    config: Option<BoxConfig>,
    work: Option<u8>,
    starts: usize,
    stops: usize,
}

impl Log {
    /// Counts a `call`, then takes the failure scripted for it.
    fn enter(&mut self, call: BoxCall) -> Result<(), BoxError> {
        *self.calls.entry(call).or_default() += 1;
        self.fails.remove(&call).map_or(Ok(()), Err)
    }

    /// The import of the file set as `bytes`, opened with `password`: `Damaged` for bytes never set, as for a file
    /// that is not `KeePass`.
    fn import(&self, bytes: &[u8], password: &str) -> Result<KeepassImport, BoxError> {
        let file = self.files.iter().find(|f| f.bytes == bytes);
        let file = file.ok_or(BoxError::Damaged)?;
        let opens = file
            .password
            .as_ref()
            .is_some_and(|p| p.as_str() == password);
        opens
            .then(|| file.import.clone())
            .ok_or(BoxError::WrongPassword)
    }

    /// Keeps `file`, in place of one set with the same bytes.
    fn keep(&mut self, file: KeepassFile) {
        self.files.retain(|f| f.bytes != file.bytes);
        self.files.push(file);
    }
}

/// Boxes shared by their clones: each session's calls count here and take the failures scripted here.
///
/// A box holds its padlock and parts in plain bytes behind the box's mark, with a checksum, so one fake opens what
/// another made; a padlock keeps only a hash of its password.
#[derive(Clone, Default)]
pub struct FakeBoxes {
    log: Arc<Mutex<Log>>,
}

impl FakeBoxes {
    /// Boxes with no session and nothing scripted.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Refuses the next `call` with `error`, once, replacing one set for that call and not yet used; a debug build
    /// stops on an error the port never returns there.
    pub fn fail(&self, call: BoxCall, error: BoxError) {
        debug_assert!(
            call.returns(error),
            "the port never returns {error:?} from {call:?}"
        );
        lock(&self.log).fails.insert(call, error);
    }

    /// Hands back `entries`, with nothing dropped, for the `KeePass` file `file` opened with `password`, in place of
    /// what was set for those bytes; another password is wrong.
    pub fn set_keepass(&self, file: &[u8], password: &str, entries: Vec<KeepassEntry>) {
        self.set_import(file, password, entries.into());
    }

    /// Hands back `import`, its entries and what the worker dropped, for the `KeePass` file `file` opened with
    /// `password`, in place of what was set for those bytes; another password is wrong.
    pub fn set_import(&self, file: &[u8], password: &str, import: KeepassImport) {
        let password = Some(Zeroizing::new(password.to_owned()));
        let bytes = file.to_vec();
        lock(&self.log).keep(KeepassFile {
            bytes,
            password,
            import,
        });
    }

    /// Makes `file` a `KeePass` file that also needs a key file, which every password fails as `WrongPassword`, as the
    /// worker answers, since KDBX cannot show that a key file is missing.
    pub fn set_key_file(&self, file: &[u8]) {
        lock(&self.log).keep(KeepassFile {
            bytes: file.to_vec(),
            password: None,
            import: KeepassImport::default(),
        });
    }

    /// How many calls of kind `call` were asked, refused ones too.
    #[must_use]
    pub fn calls(&self, call: BoxCall) -> usize {
        lock(&self.log).calls.get(&call).copied().unwrap_or(0)
    }

    /// How many sessions started.
    #[must_use]
    pub fn starts(&self) -> usize {
        lock(&self.log).starts
    }

    /// Whether a session is held, from its start until the module drops it.
    #[must_use]
    pub fn running(&self) -> bool {
        let log = lock(&self.log);
        log.starts > log.stops
    }

    /// The config the latest session took, kept after it stops.
    #[must_use]
    pub fn config(&self) -> Option<BoxConfig> {
        lock(&self.log).config
    }

    /// The work factor the latest padlock call asked, refused or not.
    #[must_use]
    pub fn work(&self) -> Option<u8> {
        lock(&self.log).work
    }
}

/// Shows the counts, never a password, a value or a part.
impl fmt::Debug for FakeBoxes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.log);
        f.debug_struct("FakeBoxes")
            .field("starts", &log.starts)
            .field("calls", &log.calls)
            .field("keepass_files", &log.files.len())
            .finish_non_exhaustive()
    }
}

impl Boxes for FakeBoxes {
    fn start(&self, cfg: BoxConfig) -> Box<dyn BoxRun> {
        let mut log = lock(&self.log);
        log.config = Some(cfg);
        log.starts += 1;
        let log = Arc::clone(&self.log);
        Box::new(FakeBoxRun { log })
    }
}

/// A live fake session; dropping it stops the worker.
struct FakeBoxRun {
    log: Arc<Mutex<Log>>,
}

impl BoxRun for FakeBoxRun {
    fn padlock(&mut self, password: &Secret, work: u8) -> Result<Padlock, BoxError> {
        let mut log = lock(&self.log);
        log.work = Some(work);
        log.enter(BoxCall::Padlock)?;
        carried(password)?;
        // A work factor below the range is `TooLarge` as well, as the worker answers.
        let within = (MIN_BOX_WORK..=MAX_BOX_WORK).contains(&work);
        within
            .then(|| format::padlock(password))
            .ok_or(BoxError::TooLarge)
    }

    fn seal(
        &mut self,
        padlock: &Padlock,
        parts: &[BoxPart],
    ) -> Result<Redacted<Vec<u8>>, BoxError> {
        lock(&self.log).enter(BoxCall::Seal)?;
        format::seal(padlock, parts).map(Redacted::new)
    }

    fn open(&mut self, bytes: &[u8], password: &Secret) -> Result<Opened, BoxError> {
        lock(&self.log).enter(BoxCall::Open)?;
        carried(password)?;
        format::open(bytes, password)
    }

    fn check(&mut self, padlock: &Padlock, password: &Secret) -> Result<bool, BoxError> {
        lock(&self.log).enter(BoxCall::Check)?;
        carried(password)?;
        format::check(padlock, password)
    }

    fn keepass(&mut self, bytes: &[u8], password: &Secret) -> Result<KeepassImport, BoxError> {
        let mut log = lock(&self.log);
        log.enter(BoxCall::Keepass)?;
        carried(password)?;
        if bytes.len() > MAX_KEEPASS_BYTES {
            return Err(BoxError::TooLarge);
        }
        log.import(bytes, password.expose())
    }
}

/// `Failed` for a password past `MAX_PASSWORD_BYTES`, which the worker's frames never carry, as the platform answers.
fn carried(password: &Secret) -> Result<(), BoxError> {
    let fits = password.expose().len() <= MAX_PASSWORD_BYTES;
    fits.then_some(()).ok_or(BoxError::Failed)
}

impl Drop for FakeBoxRun {
    fn drop(&mut self) {
        lock(&self.log).stops += 1;
    }
}

#[cfg(test)]
mod tests;
