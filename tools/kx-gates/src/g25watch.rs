//! G25's watch on system windows: the windows of one kind that show after the watch starts, each picked from plain
//! window facts, and the facts of every window shown now.
#![cfg(windows)]

use std::collections::HashSet;

use windows::Win32::Foundation::HWND;

use crate::{apps, win};

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
}
