//! A fake hidden-run port: a scripted probe answer and copies list, a remove that can fail, and a record of each probe
//! and remove.

use crate::lock;
use kx_platform::ui::{Hidden, HiddenCopy, HiddenError, Probe};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// What the fake answers and what it was asked.
struct State {
    /// The answer every probe gives.
    probe: Result<Probe, HiddenError>,
    /// The copies listed, or the list's error.
    copies: Result<Vec<HiddenCopy>, HiddenError>,
    /// The error every remove gives while set.
    remove_error: Option<HiddenError>,
    /// Every folder probed, oldest first.
    probed: Vec<PathBuf>,
    /// Every copy removed, oldest first.
    removed: Vec<HiddenCopy>,
}

/// A normal run with no copies.
impl Default for State {
    fn default() -> Self {
        Self {
            probe: Ok(Probe::Normal),
            copies: Ok(Vec::new()),
            remove_error: None,
            probed: Vec::new(),
            removed: Vec::new(),
        }
    }
}

/// A hidden-run port with scripted answers: a normal run with no copies until a test says otherwise.
#[derive(Default)]
pub struct FakeHidden {
    state: Mutex<State>,
}

/// Shows the probe's answer and counts the calls, never a path, as the real port's types hide theirs.
impl fmt::Debug for FakeHidden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = lock(&self.state);
        f.debug_struct("FakeHidden")
            .field("probe", &state.probe)
            .field("probes", &state.probed.len())
            .field("removes", &state.removed.len())
            .finish_non_exhaustive()
    }
}

impl FakeHidden {
    /// A port for a normal run with no copies.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Answers later probes with `answer`.
    pub fn set_probe(&self, answer: Result<Probe, HiddenError>) {
        lock(&self.state).probe = answer;
    }

    /// Lists `copies` from now on, or fails the list with its error.
    pub fn set_copies(&self, copies: Result<Vec<HiddenCopy>, HiddenError>) {
        lock(&self.state).copies = copies;
    }

    /// Fails later removes with `error`, or takes them again with `None`.
    pub fn set_remove_error(&self, error: Option<HiddenError>) {
        lock(&self.state).remove_error = error;
    }

    /// Every folder probed, oldest first.
    #[must_use]
    pub fn probed(&self) -> Vec<PathBuf> {
        lock(&self.state).probed.clone()
    }

    /// Every copy removed, oldest first.
    #[must_use]
    pub fn removed(&self) -> Vec<HiddenCopy> {
        lock(&self.state).removed.clone()
    }
}

impl Hidden for FakeHidden {
    fn probe(&self, data_dir: &Path) -> Result<Probe, HiddenError> {
        let mut state = lock(&self.state);
        state.probed.push(data_dir.to_path_buf());
        state.probe.clone()
    }

    fn copies(&self) -> Result<Vec<HiddenCopy>, HiddenError> {
        lock(&self.state).copies.clone()
    }

    /// Takes `copy` off the list, as the real port deletes only what it lists now: the list first, then the scripted
    /// error.
    fn remove(&self, copy: &HiddenCopy) -> Result<(), HiddenError> {
        let state = &mut *lock(&self.state);
        let list = state.copies.as_mut().map_err(|e| *e)?;
        let at = list.iter().position(|c| c == copy);
        let at = at.ok_or(HiddenError::Refused)?;
        if let Some(error) = state.remove_error {
            return Err(error);
        }
        state.removed.push(list.remove(at));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A copy in package `pkg`.
    fn copy(pkg: &str) -> HiddenCopy {
        HiddenCopy::new(PathBuf::from(pkg), pkg.into(), pkg.into())
    }

    #[test]
    fn by_default_a_run_is_normal_with_no_copies_and_each_probe_is_kept() {
        let hidden = FakeHidden::new();
        assert_eq!(hidden.probe(Path::new("data")), Ok(Probe::Normal));
        assert_eq!(hidden.copies(), Ok(vec![]));
        assert_eq!(hidden.probed(), [PathBuf::from("data")]);
    }

    #[test]
    fn it_gives_the_scripted_probe_and_copies() {
        let hidden = FakeHidden::new();
        let run = Probe::Hidden {
            package: "Claude_x".into(),
            app: "Claude".into(),
        };
        hidden.set_probe(Ok(run.clone()));
        hidden.set_copies(Ok(vec![copy("a"), copy("b")]));
        assert_eq!(hidden.probe(Path::new("data")), Ok(run));
        assert_eq!(hidden.copies(), Ok(vec![copy("a"), copy("b")]));
        hidden.set_probe(Err(HiddenError::Unreadable));
        assert_eq!(hidden.probe(Path::new("x")), Err(HiddenError::Unreadable));
    }

    #[test]
    fn remove_takes_a_listed_copy_once_and_refuses_any_other() {
        let hidden = FakeHidden::new();
        hidden.set_copies(Ok(vec![copy("a"), copy("b")]));
        assert_eq!(hidden.remove(&copy("a")), Ok(()));
        assert_eq!(hidden.remove(&copy("a")), Err(HiddenError::Refused));
        assert_eq!(hidden.remove(&copy("c")), Err(HiddenError::Refused));
        assert_eq!(hidden.copies(), Ok(vec![copy("b")]));
        assert_eq!(hidden.removed(), [copy("a")]);
    }

    #[test]
    fn a_failing_remove_keeps_the_copy_until_cleared() {
        let hidden = FakeHidden::new();
        hidden.set_copies(Ok(vec![copy("a")]));
        hidden.set_remove_error(Some(HiddenError::Undeleted));
        assert_eq!(hidden.remove(&copy("a")), Err(HiddenError::Undeleted));
        assert_eq!(hidden.copies(), Ok(vec![copy("a")]));
        let unlisted = hidden.remove(&copy("z"));
        assert_eq!(unlisted, Err(HiddenError::Refused), "the list comes first");
        hidden.set_remove_error(None);
        assert_eq!(hidden.remove(&copy("a")), Ok(()));
    }

    #[test]
    fn debug_counts_the_probes_but_prints_no_path() {
        let hidden = FakeHidden::new();
        let _ = hidden.probe(Path::new("Users/someone/data"));
        let text = format!("{hidden:?}");
        assert!(text.starts_with("FakeHidden"), "{text}");
        assert!(text.contains("probes: 1"), "{text}");
        assert!(!text.contains("someone"), "{text}");
    }

    #[test]
    fn a_failed_list_fails_every_remove_with_its_error() {
        let hidden = FakeHidden::new();
        hidden.set_copies(Err(HiddenError::Unreadable));
        assert_eq!(hidden.copies(), Err(HiddenError::Unreadable));
        assert_eq!(hidden.remove(&copy("a")), Err(HiddenError::Unreadable));
        assert!(hidden.removed().is_empty());
    }
}
