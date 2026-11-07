//! The backup box (ADR-0038 decisions 5 and 6): the worker makes padlocks, seals and opens boxes and reads `KeePass`
//! files, so neither format is parsed in our process. It needs `VaultData`, and no password, value or part prints.

use crate::{Capability, Redacted, Secret, SecretBytes, ServiceId, ServiceKey};
use thiserror::Error;

/// The box's facts from the worker's frames crate: its mark, version, name, extension and part names, the worker's
/// caps, the vault job's memory cap, and what a `KeePass` import dropped.
pub use kx_ipc::{
    BOX_EXT, BOX_MARK, BOX_NAME, BOX_PARTS, BOX_VERSION, Dropped, MAX_BOX_BYTES, MAX_BOX_WORK,
    MAX_KEEPASS_BYTES, MAX_PASSWORD_BYTES, MIN_BOX_WORK, NOTEBOOK_PART, VAULT_MEMORY_MB,
};
/// The worker's caps on the notebook, the tops of the `field_chars`, `max_entries` and `max_fields` settings.
pub use kx_ipc::{
    MAX_ENTRIES as MAX_VAULT_ENTRIES, MAX_FIELD_CHARS, MAX_FIELDS as MAX_VAULT_FIELDS,
};

/// The box's file name in each place, `<BOX_NAME>.<BOX_EXT>`.
#[must_use]
pub fn box_file() -> String {
    format!("{BOX_NAME}.{BOX_EXT}")
}

/// How the platform runs the vault job; the module fills it from its settings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BoxConfig {
    /// The longest one call may take, in ms, before the worker is stopped.
    pub deadline_ms: u32,
    /// How long the worker may sit idle before it stops, in seconds.
    pub idle_s: u32,
    /// The wait before a failed worker starts again, in ms; it doubles with each failure in a row.
    pub restart_ms: u32,
    /// The longest wait before a failed worker starts again, in ms.
    pub restart_max_ms: u32,
}

/// The open padlock that locks each new box, and the identity that opens them, locked by the backup password.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Padlock {
    /// The X25519 recipient each copy is sealed to.
    pub recipient: Redacted<String>,
    /// The identity, locked by scrypt under the password.
    pub locked: Redacted<Vec<u8>>,
}

/// One named part of a box, such as the notebook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoxPart {
    /// Its short ASCII name, such as `notebook`.
    pub name: String,
    /// Its plain bytes.
    pub bytes: SecretBytes,
}

/// What an opened box holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opened {
    /// The box's own padlock, which a computer with none keeps.
    pub padlock: Padlock,
    /// Its parts, in the box's order.
    pub parts: Vec<BoxPart>,
}

/// One entry of a `KeePass` file; a field it lacks is empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeepassEntry {
    /// Its title.
    pub title: Secret,
    /// Its user name.
    pub user: Secret,
    /// Its password.
    pub password: Secret,
    /// Its address (URL).
    pub address: Secret,
    /// Its notes, line breaks and all.
    pub notes: Secret,
    /// Its custom strings, A to Z by name, as the file keeps no order for them.
    pub custom: Vec<KeepassField>,
}

/// What a `KeePass` file gave: its entries in the file's order, and what the worker dropped to keep within its caps.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeepassImport {
    /// The entries kept.
    pub entries: Vec<KeepassEntry>,
    /// The entries and fields left out.
    pub dropped: Dropped,
}

impl From<Vec<KeepassEntry>> for KeepassImport {
    fn from(entries: Vec<KeepassEntry>) -> Self {
        Self {
            entries,
            dropped: Dropped::default(),
        }
    }
}

/// One custom string of a `KeePass` entry, which becomes a field under its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeepassField {
    /// Its name.
    pub label: Redacted<String>,
    /// Its value.
    pub value: Secret,
}

/// Why a box or a `KeePass` file was not made, opened or read.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum BoxError {
    /// The password does not open it; a `KeePass` file that also needs a key file says this too.
    #[error("the password does not open it")]
    WrongPassword,
    /// It is damaged, cut short, or not a box or `KeePass` file at all.
    #[error("the box or file is damaged or foreign")]
    Damaged,
    /// A `KeePass` 1 or KDBX 2 file, one whose key derivation passes the worker's caps, or a box from a newer version.
    #[error("the box or file is of a kind this version does not read")]
    Unsupported,
    /// It is over `MAX_BOX_BYTES` or holds more than the worker's caps; a padlock's work factor outside its range,
    /// below as above, says this too, as the worker does.
    #[error("the box or file is too large")]
    TooLarge,
    /// The worker gave no answer within `BoxConfig::deadline_ms`.
    #[error("the vault worker took too long")]
    TimedOut,
    /// The worker could not start, ended or answered badly, or the ask breaks a rule of its frames, such as a repeated
    /// part or a password past `MAX_PASSWORD_BYTES`.
    #[error("the vault worker failed")]
    Failed,
}

/// A session with the vault worker on the platform's side, one call at a time. Dropping it stops the worker.
pub trait BoxRun: Send {
    /// Makes a padlock whose identity `password` locks at scrypt work factor `work` (log N).
    ///
    /// # Errors
    /// `TooLarge` outside `MIN_BOX_WORK` to `MAX_BOX_WORK`, below as above; `TimedOut` past the deadline; `Failed` for
    /// a password past `MAX_PASSWORD_BYTES` or when the worker fails.
    fn padlock(&mut self, password: &Secret, work: u8) -> Result<Padlock, BoxError>;

    /// Seals `parts` to `padlock` into a box's bytes; it needs no password.
    ///
    /// # Errors
    /// `Damaged` for a padlock that will not read, `TooLarge` past `MAX_BOX_BYTES`, `TimedOut` past the deadline, and
    /// `Failed` for no part or an unknown or repeated one, or when the worker fails.
    fn seal(&mut self, padlock: &Padlock, parts: &[BoxPart])
    -> Result<Redacted<Vec<u8>>, BoxError>;

    /// Opens a box's `bytes` with `password`.
    ///
    /// # Errors
    /// `WrongPassword`, `Damaged` for a damaged or foreign box, `Unsupported` for a newer one, `TooLarge`, `TimedOut`
    /// past the deadline, and `Failed` for a password past `MAX_PASSWORD_BYTES` or when the worker fails.
    fn open(&mut self, bytes: &[u8], password: &Secret) -> Result<Opened, BoxError>;

    /// True when `password` opens `padlock`'s locked identity, for a memory check.
    ///
    /// # Errors
    /// `Damaged` when the identity will not read, `TimedOut` past the deadline, `Failed` for a password past
    /// `MAX_PASSWORD_BYTES` or when the worker fails.
    fn check(&mut self, padlock: &Padlock, password: &Secret) -> Result<bool, BoxError>;

    /// The entries of a `KeePass` file's `bytes`, opened with `password`, and what was dropped past the caps.
    ///
    /// # Errors
    /// `WrongPassword`, also for a file that needs a key file, as KDBX cannot show that one is needed; `Damaged` for a
    /// file that is not `KeePass`; `Unsupported` for KDBX 2 or too heavy a key derivation; `TooLarge` past
    /// `MAX_KEEPASS_BYTES`, `TimedOut` past the deadline, and `Failed` for a password past `MAX_PASSWORD_BYTES` or
    /// when the worker fails.
    fn keepass(&mut self, bytes: &[u8], password: &Secret) -> Result<KeepassImport, BoxError>;
}

/// Seals and opens boxes outside our process; the platform provides it.
pub trait Boxes: Send + Sync {
    /// Starts a session as `cfg` says; the worker itself starts at the first call.
    fn start(&self, cfg: BoxConfig) -> Box<dyn BoxRun>;
}

/// The boxes service; it needs `VaultData`.
pub struct BoxesKey;

impl ServiceKey for BoxesKey {
    type Api = dyn Boxes;
    const ID: ServiceId = ServiceId::new("boxes");
    const CAPABILITY: Option<Capability> = Some(Capability::VaultData);
}

#[cfg(test)]
mod tests;
