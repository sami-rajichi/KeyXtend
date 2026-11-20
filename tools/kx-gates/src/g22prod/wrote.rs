//! Who wrote a language's text: the voice gate build logs each written dictation's device by id, or a cloud
//! dictation's service by its row's id, and the run reads those lines back from the copy's log, never any text.

use kx_ipc::gate::{DEVICE, FAULT, SERVICE};
use kx_mod_voice::keys::KEY_UNREAD;
use kx_module_api::ports::{BlobName, CloudFault, DevicePick};
use serde_json::{Value, json};

use crate::hooklog::{read_since, value_of};

/// Who wrote a dictation, or who must: a device by id (Automatic takes any), or a cloud service by its row's id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Writer {
    /// A model on this device.
    Device(DevicePick),
    /// The cloud service of this row.
    Service(String),
}

impl Writer {
    /// Its id as a line prints it.
    fn id(&self) -> &str {
        match self {
            Self::Device(d) => d.name(),
            Self::Service(id) => id,
        }
    }

    /// The fields its line's values go under: its id's and its match's.
    fn keys(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Device(_) => (DEVICE, "device_match", "device_error"),
            Self::Service(_) => (SERVICE, "service_match", "service_error"),
        }
    }

    /// A dictation `wrote` fits this one: Automatic takes any device, else only the same device or service.
    fn fits(&self, wrote: &Self) -> bool {
        let auto = *self == Self::Device(DevicePick::Auto);
        (auto && matches!(wrote, Self::Device(_))) || self == wrote
    }
}

/// The writers of `log`'s lines holding `marker` stamped at or after `since_us`, oldest first; an error for such a line
/// with no device or service id, or with both.
pub(super) fn read(log: &str, since_us: i64, marker: &str) -> Result<Vec<Writer>, String> {
    read_since(log, since_us, |l| writer_of(l, marker))
}

/// The writer of `line` when it holds `marker`; an error that never repeats the value when it has no single id.
fn writer_of(line: &str, marker: &str) -> Result<Option<Writer>, String> {
    if !line.contains(marker) {
        return Ok(None);
    }
    let none = || format!("{marker}: no single {DEVICE} or {SERVICE} id");
    match (value_of(line, DEVICE), value_of(line, SERVICE)) {
        (Some(id), None) => match DevicePick::parse(id) {
            Some(DevicePick::Auto) | None => Err(format!("{marker}: {DEVICE} is no device id")),
            Some(device) => Ok(Some(Writer::Device(device))),
        },
        (None, Some(id)) if BlobName::new(id).is_ok() => Ok(Some(Writer::Service(id.to_owned()))),
        _ => Err(none()),
    }
}

/// A language's writer fields against `want`, and whether they fit it: the last writer's id, how many dictations
/// were written, and whether each fits, or why the log could not be read.
pub(super) fn judge(read: &Result<Vec<Writer>, String>, want: &Writer) -> (Value, bool) {
    let (id, matched, error) = want.keys();
    let writers = match read {
        Ok(writers) => writers,
        Err(e) => return (json!({ error: e }), false),
    };
    let fit = !writers.is_empty() && writers.iter().all(|w| want.fits(w));
    let last = writers.last().map(Writer::id);
    let v = json!({ id: last, "written": writers.len(), matched: fit });
    (v, fit)
}

/// The fault kinds of `log`'s lines stamped at or after `since_us`, oldest first: those holding `marker`, and the voice
/// module's key-read warning as a key fault. A value that is no plain kind name is left out, so nothing else prints.
pub(super) fn faults(log: &str, since_us: i64, marker: &str) -> Vec<String> {
    let plain = |k: &&str| !k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric());
    let kind = |l: &str| {
        if l.contains(KEY_UNREAD) {
            return Ok(Some(CloudFault::Key.kind().to_owned()));
        }
        let value = l.contains(marker).then(|| value_of(l, FAULT)).flatten();
        Ok(value.filter(plain).map(str::to_owned))
    };
    read_since(log, since_us, kind).unwrap_or_default()
}
