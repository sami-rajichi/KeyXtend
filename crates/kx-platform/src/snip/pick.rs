//! `FileAsk` over the worker's `pick` job (ADR-0056 decision 6): the system's Save and folder dialogs run outside our
//! process, one at a time on the `kx-pick` session thread, its worker granted the foreground before each and held to
//! the ask's `PickLimits`. Each answer is checked again with `PickAsk::check`, and refused in the unsaved part.

mod save;

use crate::snip::unsaved_part;
use crate::worker::{Host, Launch, Lost, PickFrames, Session, Spawner, Times};
use kx_ipc::rules::SEPARATORS;
use kx_ipc::{FileKind as Kind, IpcError, PICK_JOB, PickAsk, PickReply};
use kx_module_api::ports::{
    AskError, FileAsk, FileKind, FolderPath, PickLimits, S_PER_MIN, SaveTo, SnipFile, SnipFolder,
};
use kx_module_api::{Clock, MS_PER_S};
use save::DiskSaveTo;
use std::borrow::Cow;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// The session thread's name.
const THREAD: &str = "kx-pick";

impl Launch {
    /// The pick job capped at `job_mb` (`pick_mb`).
    #[must_use]
    pub fn pick(job_mb: u32) -> Self {
        Self {
            args: vec![PICK_JOB.to_owned()],
            memory_mb: job_mb,
        }
    }
}

/// The pick job's host times from `limits`: a dialog may wait `dialog_min` on the user before its worker is ended.
fn times(limits: &PickLimits) -> Times {
    let s_per_min = u32::try_from(S_PER_MIN).unwrap_or(u32::MAX);
    let ms_per_min = s_per_min.saturating_mul(MS_PER_S);
    Times {
        deadline_ms: u32::from(limits.dialog_min).saturating_mul(ms_per_min),
        idle_s: u32::from(limits.idle_s),
        restart_ms: limits.restart_ms,
        restart_max_ms: limits.restart_max_ms,
    }
}

impl From<Lost> for AskError {
    fn from(_: Lost) -> Self {
        Self::Gone
    }
}

/// The system's dialogs in the worker `S` starts; a Save answer writes beside `folder`'s unsaved part.
pub struct WorkerFileAsk<S: Spawner<PickFrames>> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
    /// Opened by the first ask, as only an ask brings the limits.
    session: OnceLock<Session<S, PickFrames>>,
    folder: Arc<dyn SnipFolder>,
}

impl<S: Spawner<PickFrames>> WorkerFileAsk<S> {
    /// Dialogs over `spawner` on `clock`'s time, whose Save answers copy videos from `folder`'s unsaved part.
    #[must_use]
    pub fn new(spawner: S, clock: Arc<dyn Clock>, folder: Arc<dyn SnipFolder>) -> Self {
        Self {
            spawner: Arc::new(spawner),
            clock,
            session: OnceLock::new(),
            folder,
        }
    }

    /// Shows the dialog `ask` asks for in a job held to `limits`, which a worker started otherwise first leaves, and
    /// answers the path chosen, checked, or `None` on Cancel.
    fn show(
        &self,
        ask: PickAsk<'static>,
        limits: &PickLimits,
    ) -> Result<Option<PathBuf>, AskError> {
        limits.check()?;
        let (times, launch) = (times(limits), Launch::pick(limits.job_mb));
        let session = self.session.get_or_init(|| {
            let (spawner, clock) = (Arc::clone(&self.spawner), Arc::clone(&self.clock));
            let host = Host::new(spawner, clock, times, launch.clone()).fronted();
            Session::open(THREAD, host)
        });
        let shown = session.run(move |host| {
            host.relaunch(times, launch);
            host.pick(&ask)
        });
        shown.unwrap_or(Err(AskError::Gone))
    }
}

impl<S: Spawner<PickFrames>> FileAsk for WorkerFileAsk<S> {
    fn save(
        &self,
        proposed: &SnipFile,
        limits: &PickLimits,
    ) -> Result<Option<Box<dyn SaveTo>>, AskError> {
        let (folder, name) = place(proposed).ok_or(AskError::Failed)?;
        let kind = kind(proposed.kind());
        let ask = PickAsk::Save {
            id: 0,
            folder,
            name,
            kind,
        };
        let Some(chosen) = self.show(ask, limits)? else {
            return Ok(None);
        };
        self.outside(&chosen)?;
        let file = SnipFile::new(chosen, proposed.kind()).ok_or(AskError::BadPath)?;
        Ok(Some(Box::new(DiskSaveTo::new(
            file,
            Arc::clone(&self.folder),
        ))))
    }

    fn folder(&self, at: &FolderPath, limits: &PickLimits) -> Result<Option<FolderPath>, AskError> {
        let at = at.expose().to_str().ok_or(AskError::Failed)?.to_owned();
        let ask = PickAsk::Folder {
            id: 0,
            at: Cow::Owned(at),
        };
        let chosen = self.show(ask, limits)?;
        let Some(chosen) = chosen else {
            return Ok(None);
        };
        self.outside(&chosen)?;
        FolderPath::new(chosen)
            .map(Some)
            .map_err(|_| AskError::BadPath)
    }
}

impl<S: Spawner<PickFrames>> WorkerFileAsk<S> {
    /// Refuses a place that is the unsaved part or lies in it, as each start's sweep deletes what is there.
    fn outside(&self, chosen: &Path) -> Result<(), AskError> {
        let unsaved = unsaved_part(&*self.folder);
        let inside = unsaved.is_some_and(|u| under(chosen, u.expose()));
        if inside {
            Err(AskError::BadPath)
        } else {
            Ok(())
        }
    }
}

impl<S: Spawner<PickFrames>> Host<S, PickFrames> {
    /// Shows `ask`'s dialog with the next id and answers the path chosen once `ask.check` takes it again.
    fn pick(&mut self, ask: &PickAsk<'static>) -> Result<Option<PathBuf>, AskError> {
        let make = |id| fits(numbered(ask.clone(), id));
        self.call(make, |reply| answer(ask, reply))
    }
}

/// The port's answer for the job's `reply` to `ask`, a failure's code logged; `None` for a path that fails the job's
/// own check, which ends the worker.
fn answer(
    ask: &PickAsk<'static>,
    reply: PickReply<'static>,
) -> Option<Result<Option<PathBuf>, AskError>> {
    Some(match reply {
        PickReply::Picked { path, .. } => {
            ask.check(&path).ok()?;
            Ok(Some(PathBuf::from(path.into_owned())))
        }
        PickReply::Cancelled { .. } => Ok(None),
        PickReply::Refused { .. } => Err(AskError::BadPath),
        PickReply::Failed { code, .. } => {
            tracing::warn!(code = format_args!("{code:#x}"), "a dialog did not show");
            Err(AskError::Failed)
        }
    })
}

/// True when `path` is `folder` or lies under it, its parts compared without case, as Windows reads names.
fn under(path: &Path, folder: &Path) -> bool {
    let parts = |p: &Path| -> Vec<String> {
        let text = p.to_string_lossy();
        let sep = |c: char| u8::try_from(c).is_ok_and(|b| SEPARATORS.contains(&b));
        let named = text.split(sep).filter(|part| !part.is_empty());
        named.map(str::to_lowercase).collect()
    };
    parts(path).starts_with(&parts(folder))
}

/// The folder and the name of `file`, as the frames carry them.
fn place(file: &SnipFile) -> Option<(Cow<'static, str>, Cow<'static, str>)> {
    let path = file.expose();
    let folder = path.parent()?.to_str()?.to_owned();
    let name = path.file_name()?.to_str()?.to_owned();
    Some((Cow::Owned(folder), Cow::Owned(name)))
}

/// `ask` carrying `id`.
fn numbered(mut ask: PickAsk<'static>, id: u32) -> PickAsk<'static> {
    if let PickAsk::Save { id: own, .. } | PickAsk::Folder { id: own, .. } = &mut ask {
        *own = id;
    }
    ask
}

/// Refuses an ask the frames would, such as a name Windows refuses, before any worker starts.
fn fits(ask: PickAsk<'static>) -> Result<PickAsk<'static>, AskError> {
    let written: Result<(), IpcError> = ask.write(&mut io::sink());
    written.map_err(|_| AskError::Failed)?;
    Ok(ask)
}

/// The frames' kind for the port's.
const fn kind(kind: FileKind) -> Kind {
    match kind {
        FileKind::Png => Kind::Png,
        FileKind::Mp4 => Kind::Mp4,
    }
}

#[cfg(test)]
mod tests;
