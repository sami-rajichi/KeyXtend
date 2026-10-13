//! The `sounds` subcommand: renders the recipe candidates and imports recordings in the shipping format.

mod finish;
#[cfg(test)]
mod fixture;
mod recipe;
mod resample;
mod synth;
#[cfg(test)]
mod tests;
mod units;
mod wav;

use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use self::recipe::{Book, Jitter, NAME_SEP, Recipe};
use self::units::samples;
use crate::devtools::paths::{is_relative_plain, join_rel, text};
use crate::workspace::{self, LoadError};

/// The `workspace.metadata` section that names the sound tool's files.
const SECTION: &str = "sounds";
/// The extension of every written sound.
const EXT: &str = "wav";
/// An imported recording peaks at the ceiling itself.
const IMPORT_GAIN_DB: f64 = 0.0;

/// Why a `sounds` command failed.
#[derive(Debug)]
pub(crate) enum SoundError {
    /// The workspace could not be found.
    Workspace(String),
    /// A file could not be read or written.
    Io(String),
    /// The recipe file is invalid.
    Recipe(String),
    /// The recording is not a WAV file the importer reads.
    Wav(wav::WavError),
    /// The import window holds no sample.
    Window,
    /// The sound is silent, so it cannot be normalised.
    Silent,
}

impl From<LoadError> for SoundError {
    fn from(err: LoadError) -> Self {
        Self::Workspace(err.to_string())
    }
}

impl From<wav::WavError> for SoundError {
    fn from(err: wav::WavError) -> Self {
        Self::Wav(err)
    }
}

impl fmt::Display for SoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Workspace(why) => write!(f, "workspace: {why}"),
            Self::Io(why) => write!(f, "file error: {why}"),
            Self::Recipe(why) => write!(f, "recipes: {why}"),
            Self::Wav(err) => write!(f, "{err}"),
            Self::Window => write!(f, "the window holds no sample of the recording"),
            Self::Silent => write!(f, "the sound is silent"),
        }
    }
}

/// Which part of a recording `import` keeps; with neither field set, it trims the silence instead.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Cut {
    /// Where the window starts, in ms.
    pub(crate) start_ms: Option<u32>,
    /// The window's length, in ms.
    pub(crate) len_ms: Option<u32>,
}

/// An encoded sound and its length.
#[derive(Debug)]
pub(crate) struct Sound {
    /// The WAV file.
    pub(crate) bytes: Vec<u8>,
    /// Its length, in ms.
    pub(crate) ms: f64,
}

/// Where the sound tool reads and writes, from `[workspace.metadata.sounds]`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SoundsConfig {
    /// The recipe file, under the workspace root.
    recipes: String,
    /// The folder `render` writes to unless given one, under the workspace root.
    out_dir: String,
}

/// Loads and checks `[workspace.metadata.sounds]` from `cargo metadata` JSON, with the workspace root.
pub(crate) fn load(json: &str) -> Result<(PathBuf, SoundsConfig), LoadError> {
    let (root, config): (PathBuf, SoundsConfig) = workspace::root_and_section(json, SECTION)?;
    for rel in [&config.recipes, &config.out_dir] {
        if !is_relative_plain(rel) {
            let why = format!("{SECTION}: {rel:?} is not a relative path of plain names");
            return Err(LoadError::BadConfig(why));
        }
    }
    Ok((root, config))
}

/// Runs `cargo xtask sounds render`: writes every recipe to `<out>/<name>.wav`.
pub(crate) fn render(out: Option<&str>) -> Result<(), SoundError> {
    let (root, config) = load(&workspace::metadata_json()?)?;
    let book = read_book(&root, &config)?;
    let dir = out.map_or_else(|| join_rel(root.clone(), &config.out_dir), PathBuf::from);
    for (path, ms) in render_all(&book, &dir)? {
        println!("wrote {} ({ms:.1} ms)", text(&path));
    }
    Ok(())
}

/// Runs `cargo xtask sounds import`: converts `input` to the shipping format in `output`.
pub(crate) fn import(input: &str, output: &str, cut: Cut) -> Result<(), SoundError> {
    let (root, config) = load(&workspace::metadata_json()?)?;
    let book = read_book(&root, &config)?;
    let bytes = std::fs::read(input).map_err(|err| io_error(Path::new(input), &err))?;
    let sound = import_clip(&book, &bytes, cut)?;
    std::fs::write(output, &sound.bytes).map_err(|err| io_error(Path::new(output), &err))?;
    println!("wrote {output} ({:.1} ms)", sound.ms);
    Ok(())
}

/// Reads and checks the recipe file that `config` names under the workspace `root`.
fn read_book(root: &Path, config: &SoundsConfig) -> Result<Book, SoundError> {
    let path = join_rel(root.to_path_buf(), &config.recipes);
    let source = std::fs::read_to_string(&path).map_err(|err| io_error(&path, &err))?;
    recipe::parse(&source)
}

/// A file error that names the file.
fn io_error(path: &Path, err: &std::io::Error) -> SoundError {
    SoundError::Io(format!("{}: {err}", text(path)))
}

/// Renders every recipe of `book` into `dir`, returning each file written with its length in ms.
fn render_all(book: &Book, dir: &Path) -> Result<Vec<(PathBuf, f64)>, SoundError> {
    std::fs::create_dir_all(dir).map_err(|err| io_error(dir, &err))?;
    let mut written = Vec::new();
    for recipe in &book.recipes {
        for (stem, jitter) in stems(recipe, &book.variants) {
            let sound = render_clip(book, recipe, &stem, jitter)?;
            let path = dir.join(format!("{stem}.{EXT}"));
            std::fs::write(&path, &sound.bytes).map_err(|err| io_error(&path, &err))?;
            written.push((path, sound.ms));
        }
    }
    Ok(written)
}

/// The file stems of `recipe`, each with the jitter it takes: `name`, or `name-1` to `name-n`.
fn stems<'a>(recipe: &Recipe, jitter: &'a Jitter) -> Vec<(String, Option<&'a Jitter>)> {
    let name = &recipe.name;
    match recipe.variants {
        None => vec![(name.clone(), None)],
        Some(n) => (1..=n)
            .map(|i| (format!("{name}{NAME_SEP}{i}"), Some(jitter)))
            .collect(),
    }
}

/// Renders the file `stem` of `recipe` in `book`'s format.
fn render_clip(
    book: &Book,
    recipe: &Recipe,
    stem: &str,
    jitter: Option<&Jitter>,
) -> Result<Sound, SoundError> {
    let mut voice = synth::Voice::new(stem, jitter);
    let mut mix = synth::render(recipe, f64::from(book.format.rate), &mut voice);
    finish::master(&mut mix, &book.format, recipe.gain_db + voice.level_db)
}

/// Converts the WAV file `bytes` to `book`'s format, keeping the part `cut` names.
///
/// The cut and the length cap happen at the recording's own rate, so only the kept part is resampled.
fn import_clip(book: &Book, bytes: &[u8], cut: Cut) -> Result<Sound, SoundError> {
    let audio = wav::read(bytes)?;
    let (from, to) = (audio.rate, book.format.rate);
    let span = if cut.start_ms.is_none() && cut.len_ms.is_none() {
        finish::trim(&audio.samples, book.format.trim_db)
    } else {
        finish::window(&audio.samples, f64::from(from), cut)?
    };
    let cap = |rate: u32| samples(book.format.max_ms, f64::from(rate));
    let span = &span[..span.len().min(cap(from))];
    let mut clip = resample::resample(span, from, to, &book.resample);
    clip.truncate(cap(to));
    finish::master(&mut clip, &book.format, IMPORT_GAIN_DB)
}
