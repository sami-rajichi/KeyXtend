//! The folder reveal's rule over a recording OS: the drive first, then each folder from the root down, the folder held
//! while the shell explores it, and nothing shown for a link, a file, nothing or another drive.
// Windows paths only: elsewhere a backslash is no separator, so the folders above would not be found.
#![cfg(windows)]

use std::cell::RefCell;
use std::rc::Rc;

use super::*;

/// A code the fake shell refuses with.
const CODE: i32 = -5;
/// The voice module's models folder in these tests, and the folder above it.
const MODELS: &str = r"D:\KeyXtend\models";
const ABOVE: &str = r"D:\KeyXtend";

/// Every call the fake heard, in order, each with the path it was given.
type Log = Rc<RefCell<Vec<String>>>;

/// A recording OS: whether the drive is a network one, the parts that are not plain folders, whether the shell is
/// there and explores, and every call.
#[derive(Default)]
struct Fake {
    remote: bool,
    odd: Vec<(&'static str, Found)>,
    gone: bool,
    refuses: bool,
    log: Log,
}

/// A held folder, which notes when it is let go.
struct Held(Log);

impl Drop for Held {
    fn drop(&mut self) {
        self.0.borrow_mut().push("let go".into());
    }
}

impl Fake {
    /// Notes a call with its path.
    fn note(&self, call: &str, folder: &Path) {
        self.log
            .borrow_mut()
            .push(format!("{call} {}", folder.display()));
    }

    /// What `folder` is: a plain folder unless the test made it odd.
    fn at(&self, folder: &Path) -> Found {
        let odd = self.odd.iter().find(|(p, _)| Path::new(p) == folder);
        odd.map_or(Found::Folder, |(_, f)| *f)
    }

    /// Every call so far.
    fn calls(&self) -> Vec<String> {
        self.log.borrow().clone()
    }
}

impl Plain for Fake {
    type Shell = ();
    type Hold = Held;

    fn local(&self, folder: &Path) -> bool {
        self.note("local", folder);
        !self.remote
    }

    fn found(&self, folder: &Path) -> Found {
        self.note("found", folder);
        self.at(folder)
    }

    fn hold(&self, folder: &Path) -> Result<Held, Found> {
        self.note("hold", folder);
        match self.at(folder) {
            Found::Folder => Ok(Held(Rc::clone(&self.log))),
            odd => Err(odd),
        }
    }

    fn shell(&self) -> Result<(), i32> {
        self.log.borrow_mut().push("shell".into());
        if self.gone { Err(CODE) } else { Ok(()) }
    }

    fn explore(&self, (): &(), folder: &Path) -> Result<(), i32> {
        self.note("explore", folder);
        if self.refuses { Err(CODE) } else { Ok(()) }
    }
}

/// `path` as the port checks it.
fn folder(path: &str) -> FolderPath {
    FolderPath::new(PathBuf::from(path)).unwrap()
}

#[test]
fn a_plain_folder_is_explored_once_while_held_after_each_folder_above_it() {
    let fake = Fake::default();
    assert_eq!(show_plain(&fake, &folder(MODELS)), Ok(()));
    let want = [
        format!("local {MODELS}"),
        r"found D:\".to_owned(),
        format!("found {ABOVE}"),
        format!("hold {MODELS}"),
        "shell".to_owned(),
        format!("explore {MODELS}"),
        "let go".to_owned(),
    ];
    assert_eq!(fake.calls(), want);
}

#[test]
fn forward_slashes_reach_every_check_and_the_shell_as_backslashes() {
    let fake = Fake::default();
    assert_eq!(show_plain(&fake, &folder("D:/KeyXtend/models")), Ok(()));
    let calls = fake.calls();
    assert!(calls.contains(&format!("explore {MODELS}")), "{calls:?}");
    assert!(calls.iter().all(|c| !c.contains('/')), "{calls:?}");
}

#[test]
fn a_folder_off_a_fixed_local_drive_is_a_bad_path_and_nothing_on_it_is_read() {
    let fake = Fake {
        remote: true,
        ..Fake::default()
    };
    assert_eq!(
        show_plain(&fake, &folder(MODELS)),
        Err(FolderError::BadPath)
    );
    assert_eq!(fake.calls(), [format!("local {MODELS}")]);
}

#[test]
fn a_link_or_nothing_above_the_folder_stops_before_it_is_held() {
    for (odd, error) in [
        (Found::Linked, FolderError::Linked),
        (Found::Other, FolderError::Missing),
    ] {
        let fake = Fake {
            odd: vec![(ABOVE, odd)],
            ..Fake::default()
        };
        assert_eq!(show_plain(&fake, &folder(MODELS)), Err(error));
        let calls = fake.calls();
        assert_eq!(calls.last(), Some(&format!("found {ABOVE}")), "{calls:?}");
    }
}

#[test]
fn a_link_a_file_or_nothing_as_the_folder_never_reaches_the_shell() {
    for (odd, error) in [
        (Found::Linked, FolderError::Linked),
        (Found::Other, FolderError::Missing),
    ] {
        let fake = Fake {
            odd: vec![(MODELS, odd)],
            ..Fake::default()
        };
        assert_eq!(show_plain(&fake, &folder(MODELS)), Err(error));
        let calls = fake.calls();
        assert_eq!(calls.last(), Some(&format!("hold {MODELS}")), "{calls:?}");
    }
}

#[test]
fn a_missing_or_refusing_shell_gives_its_code_and_the_folder_is_let_go() {
    let gone = Fake {
        gone: true,
        ..Fake::default()
    };
    let refuses = Fake {
        refuses: true,
        ..Fake::default()
    };
    for fake in [gone, refuses] {
        let shown = show_plain(&fake, &folder(MODELS));
        assert_eq!(shown, Err(FolderError::Refused(CODE)));
        assert_eq!(fake.calls().last().map(String::as_str), Some("let go"));
    }
}

#[test]
fn a_reparse_point_is_a_link_even_as_a_folder_and_only_a_folder_is_one() {
    assert_eq!(found(true, false), Found::Folder);
    assert_eq!(found(true, true), Found::Linked, "a junction");
    assert_eq!(found(false, true), Found::Linked, "a link to a folder");
    assert_eq!(found(false, false), Found::Other, "a file");
}
