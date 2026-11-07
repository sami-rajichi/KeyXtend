//! Setup and Add place (design "The backup box"): a place picked from the lists (`browse`), then a look at the box
//! already there, as one this computer did not write is never overwritten unasked; then, with no padlock yet, a new
//! password, typed once and shown while typed, and a padlock at `box_work`. The password page also sends a memory
//! check, a box to open, and a `KeePass` file's password exactly as typed.

use std::path::PathBuf;

use kx_module_api::ports::{BoxError, FillNote, MAX_PASSWORD_BYTES, Padlock, Place, PlaceError};
use kx_module_api::{Redacted, Secret, Zeroizing};

use super::Site;
use super::flow::{Flow, Goal, Then, when};
use super::jobs::{Job, Look};
use super::keepass::Kdbx;
use super::state::{At, Backup, Flowed, Purpose};
use super::write::place_note;
use crate::entry::{Line, Stamp, text};

/// The fewest characters of a memory check's or a box's password: any, as what it opens judges it.
const ANY: usize = 1;

impl Backup {
    /// Setup's first page: the suggested places.
    pub(super) fn setup(&mut self) -> Flowed {
        self.places(Then::Setup)
    }

    /// Picks `place` for `then`: a place kept already goes to the Backup page, and any other is looked at first.
    pub(super) fn pick(&mut self, place: Place, then: Then) -> Flowed {
        if self.kept.sites.iter().any(|s| s.place.path == place.path) {
            self.home();
            return Flowed::Show;
        }
        // The blob keeps a path as text, so a path that is not valid Unicode cannot be a place: ReadOnly says pick
        // another.
        if place.path.expose().to_str().is_none() {
            self.note = Some(FillNote::ReadOnly);
            return Flowed::Show;
        }
        let folder: Redacted<PathBuf> = place.path.clone();
        let round = self.post(Job::Look {
            folder,
            cap: self.cfg.cap,
        });
        self.asked = Some((round, Purpose::Look(place, then)));
        Flowed::Show
    }

    /// Takes a look's answer: no box or this computer's own goes on, another computer's box asks, the data folder goes
    /// back to the places with its note, and another unread folder says so.
    pub(super) fn looked(&mut self, round: u32, look: Look, at: &At<'_>) -> Flowed {
        let Some(Purpose::Look(place, then)) = self.answered(round) else {
            return Flowed::Stay;
        };
        match look {
            Look::Empty => self.go_on(place, at),
            Look::Box {
                mark: Some(mark), ..
            } if self.kept.wrote(mark) => self.go_on(place, at),
            Look::Box { modified_ms, .. } => {
                let written = modified_ms.map(|ms| when(Stamp::from_unix_ms(ms), &at.wall));
                self.foreign(place, written, then)
            }
            Look::Unread(PlaceError::Refused) => {
                self.places(then);
                self.note = Some(FillNote::DataFolder);
                Flowed::Show
            }
            Look::Unread(error) => {
                tracing::warn!(%error, "a picked folder did not read");
                self.note = Some(place_note(error).unwrap_or(FillNote::ReadOnly));
                Flowed::Show
            }
        }
    }

    /// Goes on with a place whose box may be written: with no padlock the password comes next, else it is added.
    fn go_on(&mut self, place: Place, at: &At<'_>) -> Flowed {
        if self.kept.padlock.is_none() {
            let line = Line::new(self.cfg.field_chars);
            let (place, then) = (Some(place), Then::Setup);
            let goal = Goal::New { place, then };
            self.flow = Some(Flow::Password { line, goal });
            return Flowed::Show;
        }
        self.add_site(place);
        self.home();
        self.write(at.book, false, at.now);
        Flowed::Show
    }

    /// Done on the password page: a password within the rules goes to the worker, and the line is wiped at once.
    pub(super) fn done_typing(&mut self) -> Flowed {
        let max = self.cfg.field_chars;
        let Some(Flow::Password { line, goal }) = &mut self.flow else {
            return Flowed::Stay;
        };
        let typed = line.text();
        let made = match goal {
            Goal::New { .. } => password(typed, self.cfg.min_chars, max),
            Goal::Check | Goal::Open(_) => password(typed, ANY, max),
            Goal::Keepass(_) => as_typed(typed),
        };
        let made = made.map(|password| {
            *line = Line::new(max);
            (password, goal.clone())
        });
        match made {
            Ok((password, goal)) => self.go(goal, password),
            Err(note) => self.note = Some(note),
        }
        Flowed::Show
    }

    /// Sends `password`, typed for `goal`, to the worker: a new padlock, a memory check, a box or a `KeePass` file to
    /// open.
    fn go(&mut self, goal: Goal, password: Secret) {
        let (job, purpose) = match goal {
            Goal::New { place, then } => {
                let work = self.cfg.work;
                (
                    Job::Padlock { password, work },
                    Purpose::Padlock(place, then),
                )
            }
            Goal::Check => {
                let Some(padlock) = self.kept.padlock.clone() else {
                    return;
                };
                (Job::Check { padlock, password }, Purpose::Check)
            }
            Goal::Open(source) => {
                let (file, cap) = (source.file.clone(), self.cfg.cap);
                (
                    Job::Open {
                        file,
                        cap,
                        password,
                    },
                    Purpose::Open(source),
                )
            }
            Goal::Keepass(Kdbx { file, .. }) => (Job::Keepass { file, password }, Purpose::Keepass),
        };
        let round = self.post(job);
        self.asked = Some((round, purpose));
    }

    /// Takes a padlock's answer: setup adds its place, a change makes every box wait for its rewrite, and both write.
    pub(super) fn padlocked(
        &mut self,
        round: u32,
        made: Result<Padlock, BoxError>,
        at: &At<'_>,
    ) -> Flowed {
        let Some(Purpose::Padlock(place, then)) = self.answered(round) else {
            return Flowed::Stay;
        };
        let padlock = match made {
            Ok(padlock) => padlock,
            Err(error) => {
                tracing::warn!(%error, "no padlock was made");
                self.note = Some(FillNote::BoxFailed);
                return Flowed::Show;
            }
        };
        self.kept.padlock = Some(padlock);
        self.begin_checks(Stamp::of(&at.wall));
        if then == Then::Change {
            self.kept.sites.iter_mut().for_each(|s| s.waiting = true);
        }
        match place {
            Some(place) => self.add_site(place),
            None => self.save(),
        }
        self.home();
        self.write(at.book, false, at.now);
        Flowed::Show
    }

    /// Adds `place`, whose box the next write makes, and saves.
    fn add_site(&mut self, place: Place) {
        self.kept.sites.push(Site {
            place,
            written: None,
            mark: None,
            len: 0,
            waiting: false,
        });
        self.save();
    }

    /// Shows the Backup page from its top.
    pub(super) fn home(&mut self) {
        self.flow = Some(Flow::Home {
            at: 0,
            asking: None,
        });
    }
}

/// A new backup password from `typed`: the field rules of a shown text, at least `min` characters, and within what the
/// worker's frames carry.
///
/// # Errors
/// `Refused` for a hidden mark, more than `max` characters or more than `MAX_PASSWORD_BYTES` bytes, `TooShort` under
/// `min`.
pub(super) fn password(typed: &str, min: usize, max: usize) -> Result<Secret, FillNote> {
    let mut held = Zeroizing::new(String::with_capacity(typed.len()));
    held.push_str(typed);
    text::fit(&mut held, false, max).map_err(|_| FillNote::Refused)?;
    if held.chars().count() < min {
        return Err(FillNote::TooShort);
    }
    if held.len() > MAX_PASSWORD_BYTES {
        return Err(FillNote::Refused);
    }
    Ok(Redacted::new(held))
}

/// A `KeePass` file's password from `typed`, unchanged and maybe empty, as our rules did not set it.
///
/// # Errors
/// `Refused` past `MAX_PASSWORD_BYTES`, which the worker's frames do not carry.
fn as_typed(typed: &str) -> Result<Secret, FillNote> {
    if typed.len() > MAX_PASSWORD_BYTES {
        return Err(FillNote::Refused);
    }
    let mut held = Zeroizing::new(String::with_capacity(typed.len()));
    held.push_str(typed);
    Ok(Redacted::new(held))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::MAX_FIELD_CHARS;

    /// A character of four bytes in UTF-8.
    const WIDE: char = '\u{1e900}';

    #[test]
    fn a_password_takes_the_field_rules_and_the_least_length() {
        assert_eq!(password("short", 8, 1000).err(), Some(FillNote::TooShort));
        let marked = "\u{200f}abcdefg\u{200e}";
        assert_eq!(
            password(marked, 8, 1000).err(),
            Some(FillNote::TooShort),
            "the marks go first"
        );
        assert_eq!(
            password("abc\u{202e}defgh", 8, 1000).err(),
            Some(FillNote::Refused)
        );
        let kept = password("\u{200f}كلمة سر ممتازة", 8, 1000).unwrap();
        assert_eq!(kept.expose().as_str(), "كلمة سر ممتازة");
    }

    #[test]
    fn a_password_past_the_workers_frame_is_refused_before_it_goes() {
        let most = MAX_PASSWORD_BYTES / WIDE.len_utf8();
        let fits: String = std::iter::repeat_n(WIDE, most).collect();
        assert!(password(&fits, 8, MAX_FIELD_CHARS).is_ok());
        let over: String = std::iter::repeat_n(WIDE, most + 1).collect();
        assert_eq!(password(&over, 8, most + 1).err(), Some(FillNote::Refused));
    }

    #[test]
    fn a_keepass_password_keeps_its_marks_may_be_empty_and_stops_at_the_frame() {
        let marked = "\u{200f}abc\u{200e}";
        assert_eq!(as_typed(marked).unwrap().expose().as_str(), marked);
        assert_eq!(as_typed("").unwrap().expose().as_str(), "");
        let fits = "x".repeat(MAX_PASSWORD_BYTES);
        assert!(as_typed(&fits).is_ok());
        let over = "x".repeat(MAX_PASSWORD_BYTES + 1);
        assert_eq!(as_typed(&over).err(), Some(FillNote::Refused));
    }
}
