//! The box jobs (ADR-0038 decision 6): every `BoxRun` call blocks up to `vault_ms`, so the jobs run one at a time on
//! the module's own job thread, never on the bus's or the actor's, and the places are read and written there too.
//! Each answer names its job's round, and a password is wiped as soon as its call returns.

mod read;
mod thread;

pub(crate) use thread::Jobs;

use std::path::PathBuf;
use std::sync::Arc;

use kx_module_api::ports::{
    BoxError, BoxPart, BoxRun, Found, KeepassImport, NOTEBOOK_PART, Opened, Padlock, Place,
    PlaceError, PlaceState, Places,
};
use kx_module_api::{Redacted, Secret, SecretBytes};

use super::mark::mark;

/// Hands jobs to the job thread, or to a test that runs them itself.
pub(crate) trait Post: Send {
    /// Queues `ask`; its answer comes back as a `Done`.
    fn post(&mut self, ask: Ask);

    /// Stops taking jobs and waits for the one running.
    fn stop(&mut self);
}

/// A job named by its round.
#[derive(Debug)]
pub(crate) struct Ask {
    pub(crate) round: u32,
    pub(crate) job: Job,
}

/// What the job thread does.
#[derive(Debug)]
pub(crate) enum Job {
    /// Makes a padlock that `password` locks at scrypt work factor `work`.
    Padlock { password: Secret, work: u8 },
    /// Lists the suggested places.
    Suggested,
    /// Lists the folders in `folder`, or the drives, and with `files` the files of that extension in `folder` after them.
    Folders {
        folder: Option<Redacted<PathBuf>>,
        files: Option<&'static str>,
    },
    /// Reads the box in `folder`, if one is there, to tell whether this computer wrote it.
    Look { folder: Redacted<PathBuf>, cap: u64 },
    /// Seals the notebook once and writes it to each folder that can take it.
    Write(Write),
    /// Finds the boxes in the suggested places.
    Find,
    /// Tells whether `folder` can be a place now.
    State { folder: Redacted<PathBuf> },
    /// Reads the box `file`, refused past `cap` bytes, and opens it with `password`.
    Open {
        file: Redacted<PathBuf>,
        cap: u64,
        password: Secret,
    },
    /// Checks `password` against the padlock's locked identity, for a memory check.
    Check { padlock: Padlock, password: Secret },
    /// Reads the `KeePass` `file`, refused past `MAX_KEEPASS_BYTES`, and has the worker open it with `password`.
    Keepass {
        file: Redacted<PathBuf>,
        password: Secret,
    },
}

/// A write: the padlock, the notebook's bytes, the folders, the box's cap and the room each folder is asked for.
#[derive(Debug)]
pub(crate) struct Write {
    pub(crate) padlock: Padlock,
    pub(crate) notebook: SecretBytes,
    pub(crate) folders: Vec<Redacted<PathBuf>>,
    pub(crate) cap: u64,
    pub(crate) need: u64,
}

/// A job's answer, named by its round.
#[derive(Debug)]
pub(crate) struct Done {
    pub(crate) round: u32,
    pub(crate) answer: Answer,
}

/// What a job gave.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Answer {
    Padlock(Result<Padlock, BoxError>),
    Suggested(Vec<Place>),
    Folders(Result<Vec<Place>, PlaceError>),
    Look(Look),
    Wrote(Result<Wrote, BoxError>),
    Found(Vec<Found>),
    State(PlaceState),
    Opened(Result<Unboxed, Shut>),
    Checked(Result<bool, BoxError>),
    Keepass(Result<KeepassImport, Shut>),
}

/// What a picked folder holds.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Look {
    /// No box.
    Empty,
    /// A box: its mark, or `None` past the cap, and when it was last written, when the system tells.
    Box {
        mark: Option<u64>,
        modified_ms: Option<i64>,
    },
    /// The folder could not be read.
    Unread(PlaceError),
}

/// What a write did: the sealed box's mark and length, `None` when no folder could take it, and each folder's outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Wrote {
    pub(crate) sealed: Option<(u64, u64)>,
    pub(crate) reached: Vec<Result<(), PlaceError>>,
}

/// A box opened: what it holds, and the mark and length of its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Unboxed {
    pub(crate) opened: Opened,
    pub(crate) mark: u64,
    pub(crate) len: u64,
    /// The box's own file sits beside this copy, so a write to the folder would replace another box.
    pub(crate) beside: bool,
}

/// Why a box or a `KeePass` file was not opened: its file did not read, or the worker refused it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Shut {
    Read(PlaceError),
    Open(BoxError),
}

impl Job {
    /// The answer a job gives when its run panicked.
    fn failed(&self) -> Answer {
        match self {
            Self::Padlock { .. } => Answer::Padlock(Err(BoxError::Failed)),
            Self::Suggested => Answer::Suggested(Vec::new()),
            Self::Folders { .. } => Answer::Folders(Err(PlaceError::Io)),
            Self::Look { .. } => Answer::Look(Look::Unread(PlaceError::Io)),
            Self::Write(_) => Answer::Wrote(Err(BoxError::Failed)),
            Self::Find => Answer::Found(Vec::new()),
            Self::State { .. } => Answer::State(PlaceState::Missing),
            Self::Open { .. } => Answer::Opened(Err(Shut::Open(BoxError::Failed))),
            Self::Check { .. } => Answer::Checked(Err(BoxError::Failed)),
            Self::Keepass { .. } => Answer::Keepass(Err(Shut::Open(BoxError::Failed))),
        }
    }
}

/// Runs jobs over a vault worker session and the places.
pub(crate) struct Runner {
    run: Box<dyn BoxRun>,
    places: Arc<dyn Places>,
}

impl Runner {
    /// A runner over `run` and `places`.
    pub(crate) fn new(run: Box<dyn BoxRun>, places: Arc<dyn Places>) -> Self {
        Self { run, places }
    }

    /// Runs `job`; a password goes out of scope, and is wiped, before the answer leaves.
    pub(crate) fn answer(&mut self, job: Job) -> Answer {
        match job {
            Job::Padlock { password, work } => Answer::Padlock(self.run.padlock(&password, work)),
            Job::Suggested => Answer::Suggested(self.places.suggested()),
            Job::Folders { folder, files } => Answer::Folders(self.listed(folder.as_ref(), files)),
            Job::Look { folder, cap } => Answer::Look(self.look(folder.expose(), cap)),
            Job::Write(write) => Answer::Wrote(self.write(write)),
            Job::Find => Answer::Found(self.places.find()),
            Job::State { folder } => Answer::State(self.places.state(folder.expose(), 0)),
            Job::Open {
                file,
                cap,
                password,
            } => Answer::Opened(self.open(file.expose(), cap, &password)),
            Job::Check { padlock, password } => {
                Answer::Checked(self.run.check(&padlock, &password))
            }
            Job::Keepass { file, password } => {
                Answer::Keepass(self.keepass(file.expose(), &password))
            }
        }
    }

    /// Seals the notebook once when a folder can take it, then writes it to each such folder.
    fn write(&mut self, write: Write) -> Result<Wrote, BoxError> {
        let ready = |f: &Redacted<PathBuf>| match self.places.state(f.expose(), write.need) {
            PlaceState::Ready => Ok(()),
            PlaceState::Missing => Err(PlaceError::Missing),
            PlaceState::Full => Err(PlaceError::Full),
            PlaceState::Refused => Err(PlaceError::Refused),
        };
        let mut reached: Vec<_> = write.folders.iter().map(ready).collect();
        if reached.iter().all(Result::is_err) {
            return Ok(Wrote {
                sealed: None,
                reached,
            });
        }
        let part = BoxPart {
            name: NOTEBOOK_PART.to_owned(),
            bytes: write.notebook,
        };
        let sealed = self.run.seal(&write.padlock, std::slice::from_ref(&part))?;
        let bytes = sealed.expose();
        let len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if len > write.cap {
            return Err(BoxError::TooLarge);
        }
        for (folder, reach) in write.folders.iter().zip(&mut reached) {
            if reach.is_ok() {
                *reach = self.places.write(folder.expose(), bytes);
            }
        }
        Ok(Wrote {
            sealed: Some((mark(bytes), len)),
            reached,
        })
    }
}

#[cfg(test)]
mod tests;
