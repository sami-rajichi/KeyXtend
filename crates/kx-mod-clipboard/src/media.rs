//! The helper threads (spec §7 "Security", ADR-0034 decision 5): `kx-clip-media` draws thumbnails and previews, and
//! makes the DIB an inserted PNG goes with, in the media worker through the `Pictures` port; `kx-clip-files` asks what
//! is at copied paths. So the actor never waits on the worker or on a share out of reach; each ends with its sender.

use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

use kx_module_api::Redacted;
use kx_module_api::ports::{
    ClipPart, Clipboard, DibAsk, FileFact, ImageCodec, PictureAsk, PictureError, PictureRun, Pixels,
};

mod queue;

pub use queue::Draws;
use queue::Looks;

/// The drawing thread's name, as a debugger shows it.
const MEDIA_THREAD: &str = "kx-clip-media";
/// The file thread's name.
const FILES_THREAD: &str = "kx-clip-files";

/// The longest sides pictures are drawn at; the module fills them from the settings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sides {
    /// A row's thumbnail (`thumb_px`).
    pub thumb: u32,
    /// The preview page's image (`preview_px`).
    pub preview: u32,
}

/// Which picture of an entry a job draws.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Size {
    /// The row's thumbnail, which the store keeps.
    Thumb,
    /// The preview page's image, kept only while it shows.
    Preview,
}

/// One picture to draw for an entry; its bytes print redacted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    /// The entry.
    pub id: u64,
    /// Which picture.
    pub size: Size,
    /// How the image's bytes are coded.
    pub codec: ImageCodec,
    /// The image's bytes.
    pub bytes: Redacted<Vec<u8>>,
}

/// A drawn picture, or why it was not drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drawn {
    /// The entry.
    pub id: u64,
    /// Which picture.
    pub size: Size,
    /// The pixels, or the worker's error.
    pub pixels: Result<Pixels, PictureError>,
}

/// The DIB to make for an insert's PNG; the PNG prints redacted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DibJob {
    /// The ask's number, so a late answer is known.
    pub n: u32,
    /// The PNG and the DIB's cap.
    pub ask: DibAsk,
}

/// The DIB made for ask `n`, or why it was not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Made {
    /// The ask's number.
    pub n: u32,
    /// The DIB's bytes, or the worker's error.
    pub dib: Result<Redacted<Vec<u8>>, PictureError>,
}

/// Work for the media thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Task {
    /// Draw a picture.
    Draw(Job),
    /// Make a DIB.
    Dib(DibJob),
}

/// What the media thread did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Done {
    /// A picture drawn, or not.
    Drawn(Drawn),
    /// A DIB made, or not.
    Made(Made),
}

/// Why paths are looked at.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// To size file rows and drop entries whose files are all gone.
    Rows,
    /// For insert number `n`, before it pastes.
    Insert(u32),
}

/// The paths of some file entries to look at; they print redacted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Look {
    /// Why.
    pub why: Why,
    /// Each entry and its paths.
    pub items: Vec<(u64, Redacted<Vec<PathBuf>>)>,
}

/// What a look found: each entry's facts, in its paths' order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// Why the look was asked.
    pub why: Why,
    /// Each entry and its facts.
    pub items: Vec<(u64, Vec<FileFact>)>,
}

/// The job drawing `size` of entry `id` from the image part of `parts`, whose bytes move in uncopied; `None` when
/// they hold none.
#[must_use]
pub fn job_of(id: u64, size: Size, parts: Vec<ClipPart>) -> Option<Job> {
    parts.into_iter().find_map(|p| match p {
        ClipPart::Image { codec, bytes } => Some(Job {
            id,
            size,
            codec,
            bytes,
        }),
        _ => None,
    })
}

/// Draws `job` through `run` at its side in `sides`.
pub fn draw(run: &mut dyn PictureRun, job: Job, sides: Sides) -> Drawn {
    let side = match job.size {
        Size::Thumb => sides.thumb,
        Size::Preview => sides.preview,
    };
    let ask = PictureAsk {
        codec: job.codec,
        bytes: job.bytes,
        side,
    };
    Drawn {
        id: job.id,
        size: job.size,
        pixels: run.draw(ask),
    }
}

/// Does `task` through `run`, drawing at its side in `sides`.
pub fn work(run: &mut dyn PictureRun, task: Task, sides: Sides) -> Done {
    match task {
        Task::Draw(job) => Done::Drawn(draw(run, job, sides)),
        Task::Dib(DibJob { n, ask }) => Done::Made(Made {
            n,
            dib: run.dib(ask),
        }),
    }
}

/// Asks `clip` what is at each entry's paths in `look`; an unreachable share can take long.
pub fn look(clip: &dyn Clipboard, look: &Look) -> Facts {
    let items = look.items.iter();
    Facts {
        why: look.why,
        items: items.map(|(id, paths)| (*id, clip.files(paths))).collect(),
    }
}

/// Starts `kx-clip-media`: it does each task from `tasks` through `run` and gives `tell` the result, until `tasks`
/// closes; then the session drops, which stops the worker.
///
/// # Errors
/// The OS's error when the thread does not start.
pub fn spawn_media(
    run: Box<dyn PictureRun>,
    sides: Sides,
    tasks: Receiver<Task>,
    tell: impl Fn(Done) + Send + 'static,
) -> io::Result<JoinHandle<()>> {
    let mut run = run;
    let serve = move || {
        for task in tasks {
            tell(work(&mut *run, task, sides));
        }
    };
    thread::Builder::new()
        .name(MEDIA_THREAD.to_owned())
        .spawn(serve)
}

/// Starts `kx-clip-files`: it answers the looks from `looks` through `tell`, insert checks first and only the newest
/// row look, until `looks` closes.
///
/// # Errors
/// The OS's error when the thread does not start.
pub fn spawn_files(
    clip: Arc<dyn Clipboard>,
    looks: Receiver<Look>,
    tell: impl Fn(Facts) + Send + 'static,
) -> io::Result<JoinHandle<()>> {
    let serve = move || {
        let mut waiting = Looks::default();
        while let Some(asked) = waiting.next(&looks) {
            tell(look(&*clip, &asked));
        }
    };
    thread::Builder::new()
        .name(FILES_THREAD.to_owned())
        .spawn(serve)
}

#[cfg(test)]
mod tests;
