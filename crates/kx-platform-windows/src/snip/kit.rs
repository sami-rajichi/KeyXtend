//! What the recorder can use now, read without loading a codec or opening a device (edges 11 and 14): the libraries a
//! video job loads first, in the system folder; the default microphone and speakers, in the system's device list; and
//! the microphone's privacy switches, which `kit.toml` names.

use std::path::Path;
use std::sync::LazyLock;

use kx_module_api::ports::{Gear, RecordFacts};
use kx_platform::snip::Kit;
use serde::Deserialize;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Media::Audio::{
    EDataFlow, IMMDeviceEnumerator, MMDeviceEnumerator, eCapture, eConsole, eRender,
};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::Registry::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW,
};
use windows::core::HSTRING;

use crate::com::Com;
use crate::worker::system_dir;

/// The recorder's own table reader, so its checks hold here too; the facts read only its libraries, never load them.
/// Its crate formats it, so rustfmt skips it here.
#[allow(dead_code, reason = "The facts read only the table's libraries.")]
#[path = "../../../kx-platform-record/src/table.rs"]
#[rustfmt::skip]
mod table;

/// The microphone's privacy switches, compiled in.
const SWITCHES: &str = include_str!("kit.toml");
/// Room for a switch's value with its end, in UTF-16 units; a longer one is never the deny word.
const VALUE_UNITS: usize = 16;

/// The switches' table.
#[derive(Deserialize)]
struct Switches {
    /// The value each key holds.
    value: String,
    /// What it reads when off.
    deny: String,
    mic: Vec<Place>,
}

/// One switch's key.
#[derive(Deserialize)]
struct Place {
    hive: Hive,
    key: String,
}

/// The registry root a key lies under.
#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Hive {
    Machine,
    User,
}

/// The shipped switches; tests prove they parse.
static SHIPPED: LazyLock<Option<Switches>> = LazyLock::new(|| {
    let read = toml::from_str(SWITCHES);
    read.inspect_err(|_| tracing::warn!("the microphone's switch table does not parse"))
        .ok()
});

/// What Windows can record with now; each read asks the system afresh.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinKit;

impl Kit for WinKit {
    fn facts(&self) -> RecordFacts {
        let (mic, sound) = devices();
        RecordFacts {
            video: video(),
            mic: gear(mic, mic_off()),
            sound: gear(sound, false),
        }
    }
}

/// The gear of a device `found` or not, with its switch `off` or not.
const fn gear(found: bool, off: bool) -> Gear {
    match (found, off) {
        (false, _) => Gear::Missing,
        (true, true) => Gear::Refused,
        (true, false) => Gear::Ready,
    }
}

/// True when every library a video job loads first is in the system folder.
fn video() -> bool {
    let (Some(table), Ok(dir)) = (table::table(), system_dir()) else {
        return false;
    };
    video_in(&dir, &table.media_dlls)
}

/// True when each of `dlls` is a file in `dir`; it only looks, never loads one.
fn video_in(dir: &Path, dlls: &[String]) -> bool {
    dlls.iter().all(|dll| dir.join(dll).is_file())
}

/// True when the shipped switches refuse the microphone.
fn mic_off() -> bool {
    refused(SHIPPED.as_ref())
}

/// True when any of `table`'s switches is off, or when there is no table, so the microphone never reads ready by error.
fn refused(table: Option<&Switches>) -> bool {
    let Some(table) = table else {
        return true;
    };
    let values = table.mic.iter().map(|p| read(p.hive, &p.key, &table.value));
    off(values, &table.deny)
}

/// True when any of `values`, a switch's value each or `None` where it cannot be read, is the `deny` word.
fn off(values: impl IntoIterator<Item = Option<String>>, deny: &str) -> bool {
    values.into_iter().any(|v| v.is_some_and(|v| v == deny))
}

/// Whether a default microphone and default speakers are in the system's device list, opening neither.
#[allow(
    unsafe_code,
    reason = "Asks the system's device list for its defaults."
)]
fn devices() -> (bool, bool) {
    let Ok(_com) = Com::start() else {
        return (false, false);
    };
    // SAFETY: COM runs on this thread for the list's whole life, as `_com` drops after it.
    let list = unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER) };
    let Ok(list): windows::core::Result<IMMDeviceEnumerator> = list else {
        return (false, false);
    };
    // SAFETY: a call on the live list, on this thread.
    let there = |flow: EDataFlow| unsafe { list.GetDefaultAudioEndpoint(flow, eConsole) }.is_ok();
    (there(eCapture), there(eRender))
}

/// The string value `name` of `key` under `hive`, when it is there and short.
#[allow(unsafe_code, reason = "Reads one registry string into a local buffer.")]
fn read(hive: Hive, key: &str, name: &str) -> Option<String> {
    let root = match hive {
        Hive::Machine => HKEY_LOCAL_MACHINE,
        Hive::User => HKEY_CURRENT_USER,
    };
    let (key, name) = (HSTRING::from(key), HSTRING::from(name));
    let mut buf = [0_u16; VALUE_UNITS];
    let mut size = u32::try_from(size_of_val(&buf)).ok()?;
    let data = Some(buf.as_mut_ptr().cast());
    // SAFETY: Windows writes at most `size` bytes into `buf`, ends the string, and writes the size back.
    let r = unsafe {
        RegGetValueW(
            root,
            &key,
            &name,
            RRF_RT_REG_SZ,
            None,
            data,
            Some(&raw mut size),
        )
    };
    if r != ERROR_SUCCESS {
        return None;
    }
    let units = buf.get(..usize::try_from(size).ok()? / size_of::<u16>())?;
    let end = units.iter().position(|&u| u == 0).unwrap_or(units.len());
    String::from_utf16(units.get(..end)?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_test_support::tempdir::TempDir;
    use std::fs;

    #[test]
    fn the_shipped_tables_parse_with_every_switch_and_library() {
        let switches = SHIPPED.as_ref().expect("the switches parse");
        assert_eq!(switches.mic.len(), 3);
        assert!(switches.mic.iter().all(|p| !p.key.is_empty()));
        assert!(!switches.deny.is_empty() && !switches.value.is_empty());
        let media = table::table().expect("the recorder's table parses");
        assert!(!media.media_dlls.is_empty());
    }

    #[test]
    fn no_switch_table_refuses_the_microphone() {
        assert!(refused(None));
    }

    #[test]
    fn any_switch_reading_the_deny_word_refuses_and_an_unread_one_does_not() {
        let deny = "Deny";
        let read = |v: &str| Some(v.to_owned());
        assert!(off([read("Allow"), None, read(deny)], deny));
        assert!(!off([read("Allow"), None, read("deny")], deny));
        assert!(!off([None, None], deny));
        assert!(!off(Vec::new(), deny));
    }

    #[test]
    fn video_needs_every_library_as_a_file_in_the_folder() {
        let dir = TempDir::new("kit").unwrap();
        let dlls = ["a.dll".to_owned(), "b.dll".to_owned()];
        fs::write(dir.path().join(&dlls[0]), b"").unwrap();
        assert!(!video_in(dir.path(), &dlls), "one is missing");
        fs::create_dir(dir.path().join(&dlls[1])).unwrap();
        assert!(!video_in(dir.path(), &dlls), "a folder is no library");
        fs::remove_dir(dir.path().join(&dlls[1])).unwrap();
        fs::write(dir.path().join(&dlls[1]), b"").unwrap();
        assert!(video_in(dir.path(), &dlls));
    }

    #[test]
    fn a_device_is_missing_refused_by_its_switch_or_ready() {
        assert_eq!(gear(false, false), Gear::Missing);
        assert_eq!(
            gear(false, true),
            Gear::Missing,
            "no device beats its switch"
        );
        assert_eq!(gear(true, true), Gear::Refused);
        assert_eq!(gear(true, false), Gear::Ready);
    }

    #[test]
    fn a_registry_value_that_is_not_there_reads_as_none() {
        assert_eq!(read(Hive::User, r"Software\KeyXtend\none", "Value"), None);
    }
}
