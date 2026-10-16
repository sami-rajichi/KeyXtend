//! The `sounds` subcommand: renders the recipe candidates, imports recordings in the shipping format and joins two sounds.

mod finish;
#[cfg(test)]
mod fixture;
mod mix;
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

use self::recipe::{Book, Recipe};
use self::units::samples;
use crate::devtools::paths::{is_relative_plain, join_rel, text};
use crate::workspace::{self, LoadError};

/// The `workspace.metadata` section that names the sound tool's files.
const SECTION: &str = "sounds";
/// The extension of every written sound.
const EXT: &str = "wav";
/// An imported or joined sound peaks at the ceiling itself.
const FULL_GAIN_DB: f64 = 0.0;

/// Why a `sounds` command failed.
#[derive(Debug)]
pub(crate) enum SoundError {
    /// The workspace could not be found.
    Workspace(String),
    /// A file could not be read or written.
    Io(String),
    /// The recipe file is invalid.
    Recipe(String),
    /// The sounds file that holds the shipping format is invalid.
    Format(String),
    /// The recording is not a WAV file the importer reads.
    Wav(wav::WavError),
    /// The import window holds no sample.
    Window,
    /// The sound is silent, so it cannot be normalised.
    Silent,
    /// The sound would be longer than the limit, in ms.
    TooLong(f64),
    /// A file to join is not in the shipping format.
    NotShipping(String),
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
            Self::Format(why) => write!(f, "sound format: {why}"),
            Self::Wav(err) => write!(f, "{err}"),
            Self::Window => write!(f, "the window holds no sample of the recording"),
            Self::Silent => write!(f, "the sound is silent"),
            Self::TooLong(max_ms) => write!(f, "the sound would be longer than {max_ms} ms"),
            Self::NotShipping(name) => {
                write!(f, "{name} is not in the shipping format; import it first")
            }
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
    /// The sounds file that holds the shipping format, under the workspace root.
    format: String,
    /// The folder `render` writes to unless given one, under the workspace root.
    out_dir: String,
}

/// Loads and checks `[workspace.metadata.sounds]` from `cargo metadata` JSON, with the workspace root.
pub(crate) fn load(json: &str) -> Result<(PathBuf, SoundsConfig), LoadError> {
    let (root, config): (PathBuf, SoundsConfig) = workspace::root_and_section(json, SECTION)?;
    for rel in [&config.recipes, &config.format, &config.out_dir] {
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
    let sound = import_clip(&book, &read_file(input)?, cut)?;
    write_sound(output, &sound)
}

/// Runs `cargo xtask sounds join`: mixes `second` into `first` from `at_ms` and writes `output`.
pub(crate) fn join(first: &str, second: &str, output: &str, at_ms: u32) -> Result<(), SoundError> {
    let (root, config) = load(&workspace::metadata_json()?)?;
    let book = read_book(&root, &config)?;
    let part = |path: &str| mix::shipping(&book, path, &read_file(path)?);
    let sound = mix::join(&book, &part(first)?, &part(second)?, at_ms)?;
    write_sound(output, &sound)
}

/// The bytes of the file at `path`.
fn read_file(path: &str) -> Result<Vec<u8>, SoundError> {
    std::fs::read(path).map_err(|err| io_error(Path::new(path), &err))
}

/// Writes `sound` to `path` and says so.
fn write_sound(path: &str, sound: &Sound) -> Result<(), SoundError> {
    std::fs::write(path, &sound.bytes).map_err(|err| io_error(Path::new(path), &err))?;
    println!("wrote {path} ({:.1} ms)", sound.ms);
    Ok(())
}

/// Reads and checks the recipe and sounds files that `config` names under the workspace `root`.
fn read_book(root: &Path, config: &SoundsConfig) -> Result<Book, SoundError> {
    let read = |rel: &str| {
        let path = join_rel(root.to_path_buf(), rel);
        std::fs::read_to_string(&path).map_err(|err| io_error(&path, &err))
    };
    recipe::parse(&read(&config.recipes)?, &read(&config.format)?)
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
        for stem in recipe.stems() {
            let sound = render_clip(book, recipe, &stem)?;
            let path = dir.join(format!("{stem}.{EXT}"));
            std::fs::write(&path, &sound.bytes).map_err(|err| io_error(&path, &err))?;
            written.push((path, sound.ms));
        }
    }
    Ok(written)
}

/// Renders the file `stem` of `recipe` in `book`'s format, jittered when the recipe has variants.
fn render_clip(book: &Book, recipe: &Recipe, stem: &str) -> Result<Sound, SoundError> {
    let jitter = recipe.variants.map(|_| &book.variants);
    let mut voice = synth::Voice::new(stem, jitter);
    let mut mix = synth::render(recipe, f64::from(book.format.rate), &mut voice);
    finish::master(&mut mix, &book.format, recipe.gain_db + voice.level_db)
}

/// Converts the WAV file `bytes` to `book`'s format, keeping the part `cut` names.
///
/// A trimmed recording is cut to the length limit, but a longer window is refused; both happen before resampling.
fn import_clip(book: &Book, bytes: &[u8], cut: Cut) -> Result<Sound, SoundError> {
    let audio = wav::read(bytes)?;
    let (from, to) = (audio.rate, book.format.rate);
    let cap = |rate: u32| samples(book.format.max_ms, f64::from(rate));
    let span = if cut.start_ms.is_none() && cut.len_ms.is_none() {
        let loud = finish::trim(&audio.samples, book.format.trim_db);
        &loud[..loud.len().min(cap(from))]
    } else {
        let part = finish::window(&audio.samples, f64::from(from), cut)?;
        if part.len() > cap(from) {
            return Err(SoundError::TooLong(book.format.max_ms));
        }
        part
    };
    let mut clip = resample::resample(span, from, to, &book.resample);
    clip.truncate(book.format.max_frames());
    finish::master(&mut clip, &book.format, FULL_GAIN_DB)
}
