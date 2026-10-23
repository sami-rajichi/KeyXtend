//! G25's watch on system windows: the windows of one kind that show after the watch starts, each picked from plain
//! window facts, and the facts of every window shown now, with G25's refusal while one has the dialog's class.
#![cfg(windows)]

use std::collections::HashSet;

use windows::Win32::Foundation::HWND;

use crate::{apps, win};

/// What the refusal names as the program of a window whose program does not read.
const UNREAD: &str = "a program that does not read";

/// What G25 reads of a window: its class and program, never its title, which may be private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    /// Its class.
    pub class: String,
    /// Its program's file name, when it can be read.
    pub program: Option<String>,
}

impl Facts {
    /// The facts of window `w`.
    fn of(w: HWND) -> Self {
        Self {
            class: win::class(w),
            program: win::program_name(w),
        }
    }
}

/// True when a window with `facts` has one of `classes` and, when `program` names one, belongs to it.
pub fn picked(facts: &Facts, classes: &[String], program: Option<&str>) -> bool {
    let from = |p: &str| {
        facts
            .program
            .as_deref()
            .is_some_and(|f| apps::same_program(f, p))
    };
    classes.contains(&facts.class) && program.is_none_or(from)
}

/// The facts of every window a person sees now, old or new.
pub fn seen() -> Vec<Facts> {
    win::seen_windows().into_iter().map(Facts::of).collect()
}

/// Why G25 refuses while window `f` shows, by its class and program, never its title.
fn already_open(f: &Facts) -> String {
    let program = f.program.as_deref().unwrap_or(UNREAD);
    format!(
        "a `{}` window of {program} is open, which G25's Alt taps could reach: the owner closes it first",
        f.class
    )
}

/// Ok unless one of the windows `seen` has one of the dialog's `classes`, whatever its program, old or new; else why
/// G25 refuses, naming the first such window.
pub fn none_open(seen: &[Facts], classes: &[String]) -> Result<(), String> {
    match seen.iter().find(|f| picked(f, classes, None)) {
        Some(f) => Err(already_open(f)),
        None => Ok(()),
    }
}

/// The windows of one kind a person sees that they did not see when the watch started.
pub struct Watch<'a> {
    before: HashSet<isize>,
    classes: &'a [String],
    program: Option<&'a str>,
}

impl<'a> Watch<'a> {
    /// Starts watching for windows of `classes`, from `program` when it names one.
    pub fn start(classes: &'a [String], program: Option<&'a str>) -> Self {
        let before = win::seen_windows().into_iter().map(win::key).collect();
        Self {
            before,
            classes,
            program,
        }
    }

    /// The watched windows shown now; a window that was there but cloaked, as the panel's are, counts once shown.
    pub fn shown(&self) -> Vec<HWND> {
        let new = |w: &HWND| !self.before.contains(&win::key(*w));
        let kind = |w: &HWND| picked(&Facts::of(*w), self.classes, self.program);
        win::seen_windows()
            .into_iter()
            .filter(new)
            .filter(kind)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(class: &str, program: Option<&str>) -> Facts {
        Facts {
            class: class.into(),
            program: program.map(Into::into),
        }
    }

    fn list(classes: &[&str]) -> Vec<String> {
        classes.iter().map(|&c| c.to_owned()).collect()
    }

    #[test]
    fn the_switcher_is_picked_by_its_class_alone() {
        let classes = list(&["XamlExplorerHostIslandWindow", "MultitaskingViewFrame"]);
        let w11 = facts("XamlExplorerHostIslandWindow", Some("explorer.exe"));
        assert!(picked(&w11, &classes, None));
        assert!(
            picked(&facts("MultitaskingViewFrame", None), &classes, None),
            "an unread program does not matter"
        );
        assert!(!picked(
            &facts("Notepad", Some("notepad.exe")),
            &classes,
            None
        ));
        assert!(!picked(&w11, &[], None), "no class, no window");
    }

    #[test]
    fn the_power_dialog_needs_its_class_and_the_shells_program() {
        let (class, shell) = (list(&["#32770"]), Some("explorer.exe"));
        assert!(
            picked(&facts("#32770", Some("EXPLORER.EXE")), &class, shell),
            "in any case"
        );
        assert!(
            !picked(&facts("#32770", Some("notepad.exe")), &class, shell),
            "Notepad's own dialog"
        );
        assert!(
            !picked(&facts("#32770", None), &class, shell),
            "an unread program is never the dialog"
        );
        assert!(
            picked(&facts("#32770", None), &class, None),
            "but a watch on the class alone sees it"
        );
        assert!(!picked(
            &facts("Shell_TrayWnd", Some("explorer.exe")),
            &class,
            shell
        ));
    }

    #[test]
    fn g25_refuses_while_any_window_of_the_dialogs_class_shows_and_names_its_class_and_program() {
        let class = list(&["#32770"]);
        assert_eq!(none_open(&[], &class), Ok(()));
        let notepad = facts("Notepad", Some("notepad.exe"));
        assert_eq!(none_open(std::slice::from_ref(&notepad), &class), Ok(()));
        for (open, program) in [
            (facts("#32770", Some("explorer.exe")), "explorer.exe"),
            (facts("#32770", None), UNREAD),
            (facts("#32770", Some("notepad.exe")), "notepad.exe"),
        ] {
            let why = none_open(&[notepad.clone(), open.clone()], &class).unwrap_err();
            assert!(why.contains("`#32770`") && why.contains(program), "{why}");
            assert!(why.contains("Alt taps could reach"), "{why}");
        }
    }
}
