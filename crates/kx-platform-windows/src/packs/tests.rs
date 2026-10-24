//! The pack rules over a fake disk: the code check, the size limit and the folder order.

use std::cell::RefCell;
use std::collections::HashMap;

use proptest::prelude::*;

use super::*;

/// Random codes tried by the property test.
const CASES: u32 = 1_024;
/// The language every folder test looks up.
const ISO: &str = "en";

/// A fake disk: each pack file's answer, and every path it was asked for.
#[derive(Default)]
struct Disk {
    files: HashMap<PathBuf, Result<u8, PackError>>,
    asked: RefCell<Vec<PathBuf>>,
}

impl Disk {
    /// A disk holding `ISO`'s pack in each folder of `with`, each answering its place in that list.
    fn with(with: &[&PathBuf]) -> Self {
        let files = with
            .iter()
            .zip(0..)
            .map(|(dir, n)| (pack(dir), Ok(n)))
            .collect();
        Self {
            files,
            asked: RefCell::default(),
        }
    }
}

impl Os for Disk {
    type View = u8;

    fn map(&self, path: &Path) -> Result<Option<u8>, PackError> {
        self.asked.borrow_mut().push(path.to_path_buf());
        self.files.get(path).copied().transpose()
    }
}

/// An absolute folder on every system.
fn folder(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}

/// `ISO`'s pack file in `dir`.
fn pack(dir: &Path) -> PathBuf {
    dir.join(format!("{ISO}.{PACK_EXT}"))
}

#[test]
fn a_code_of_two_or_three_lowercase_letters_names_its_pack() {
    assert_eq!(file_name("en").as_deref(), Some("en.kxw"));
    assert_eq!(file_name("fil").as_deref(), Some("fil.kxw"));
}

#[test]
fn any_other_code_names_no_pack() {
    let bad = [
        "", "e", "engl", "EN", "En", "../x", "..", "e1", "é", "ع", "e n", "en\0", "en.", "a/b",
        "a\\b", "con", "prn", "aux", "nul", "CON", "com1", "lpt1",
    ];
    for iso in bad {
        assert_eq!(file_name(iso), None, "{iso:?}");
    }
}

#[test]
fn a_bad_code_never_reaches_the_disk() {
    let disk = Disk::default();
    for iso in ["../x", "EN", "e", "engl", "con", "nul"] {
        assert_eq!(find(&disk, &[folder("a")], iso), Ok(None));
    }
    assert!(disk.asked.borrow().is_empty());
}

#[test]
fn the_first_folder_with_the_pack_wins() {
    let (a, b, c) = (folder("a"), folder("b"), folder("c"));
    let disk = Disk::with(&[&b, &c]);
    let dirs = [a.clone(), b.clone(), c];
    assert_eq!(find(&disk, &dirs, ISO), Ok(Some(0)));
    assert_eq!(*disk.asked.borrow(), [pack(&a), pack(&b)]);
}

#[test]
fn a_missing_language_is_none_after_every_folder() {
    let disk = Disk::default();
    let dirs = [folder("a"), folder("b")];
    assert_eq!(find(&disk, &dirs, ISO), Ok(None));
    assert_eq!(*disk.asked.borrow(), [pack(&dirs[0]), pack(&dirs[1])]);
}

#[test]
fn an_unreadable_pack_stands_over_a_later_folder() {
    let (a, b) = (folder("a"), folder("b"));
    let mut disk = Disk::with(&[&b]);
    disk.files.insert(pack(&a), Err(PackError::Unreadable));
    assert_eq!(find(&disk, &[a, b], ISO), Err(PackError::Unreadable));
}

#[test]
fn a_relative_folder_is_never_tried() {
    let abs = folder("a");
    let disk = Disk::with(&[&PathBuf::from("packs"), &abs]);
    assert_eq!(
        find(&disk, &[PathBuf::from("packs"), abs.clone()], ISO),
        Ok(Some(1))
    );
    assert_eq!(*disk.asked.borrow(), [pack(&abs)]);
}

#[test]
fn a_pack_is_one_byte_to_64_mebibytes() {
    assert_eq!(MAX_PACK_BYTES, 67_108_864);
    assert_eq!(view_len(0), Err(PackError::Unreadable));
    assert_eq!(view_len(1), Ok(1));
    assert_eq!(view_len(MAX_PACK_BYTES), Ok(67_108_864));
    assert_eq!(view_len(MAX_PACK_BYTES + 1), Err(PackError::Unreadable));
    assert_eq!(view_len(u64::MAX), Err(PackError::Unreadable));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn only_short_lowercase_codes_that_name_no_device_name_a_pack_in_its_folder(
        iso in prop_oneof![
            "[a-z]{2,3}",
            "[a-zA-Z./\\\\:]{0,5}",
            "\\PC{0,5}",
            prop::sample::select(RESERVED_NAMES).prop_map(str::to_owned),
        ],
    ) {
        let good = (MIN_ISO_LEN..=MAX_ISO_LEN).contains(&iso.len())
            && iso.bytes().all(|b| b.is_ascii_lowercase())
            && !RESERVED_NAMES.contains(&iso.as_str());
        let name = file_name(&iso);
        prop_assert_eq!(name.is_some(), good);
        if let Some(name) = name {
            prop_assert_eq!(Path::new(&name).file_stem(), Some(std::ffi::OsStr::new(&iso)));
            let dir = folder("a");
            let path = dir.join(&name);
            prop_assert_eq!(path.parent(), Some(dir.as_path()));
        }
    }
}
