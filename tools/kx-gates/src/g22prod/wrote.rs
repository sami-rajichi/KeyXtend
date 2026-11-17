//! Which device wrote a language's text: the voice gate build logs each written dictation's device by id, and the run
//! reads those lines back from the copy's log, never any text.
#![cfg(windows)]

use kx_module_api::ports::DevicePick;
use serde_json::{Value, json};

use crate::hooklog::{read_since, value_of};

/// The field of the build's line that holds the device id.
pub(super) const DEVICE: &str = "device";

/// The devices of `log`'s lines holding `marker` stamped at or after `since_us`, oldest first; an error for such a line
/// with no device id.
pub(super) fn read(log: &str, since_us: i64, marker: &str) -> Result<Vec<DevicePick>, String> {
    read_since(log, since_us, |l| device_of(l, marker))
}

/// The device of `line` when it holds `marker`; an error that never repeats the value when it is no device id.
fn device_of(line: &str, marker: &str) -> Result<Option<DevicePick>, String> {
    if !line.contains(marker) {
        return Ok(None);
    }
    let id = value_of(line, DEVICE).ok_or_else(|| format!("{marker}: no {DEVICE}"))?;
    match DevicePick::parse(id) {
        Some(DevicePick::Auto) | None => Err(format!("{marker}: {DEVICE} is no device id")),
        device => Ok(device),
    }
}

/// A language's device fields against `pick`, and whether they fit it: the last device's id, how many dictations were
/// written, and whether each fits, or why the log could not be read.
pub(super) fn judge(read: &Result<Vec<DevicePick>, String>, pick: &DevicePick) -> (Value, bool) {
    let devices = match read {
        Ok(devices) => devices,
        Err(e) => return (json!({ "device_error": e }), false),
    };
    let fit = !devices.is_empty() && devices.iter().all(|d| fits(pick, d));
    let last = devices.last().map(DevicePick::name);
    let v = json!({ "device": last, "written": devices.len(), "device_match": fit });
    (v, fit)
}

/// A dictation `wrote` on fits `pick`: Automatic takes any device, else only the one picked.
fn fits(pick: &DevicePick, wrote: &DevicePick) -> bool {
    *pick == DevicePick::Auto || pick == wrote
}
