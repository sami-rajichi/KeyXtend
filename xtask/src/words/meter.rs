//! The meter (spec §6 "Target"): types the held-back sentences through the module's own tracker, scorer and learner,
//! counts the presses the chips save, and checks the gate languages. It never ships and never prints a word.

mod speed;
mod tally;
#[cfg(test)]
mod tests;
mod typist;
mod verdict;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use kx_mod_predict::{Pack, PredictSettings};
use kx_module_api::ports::{PACK_INDEX, PackIndex, pack_file};
use kx_module_api::{Settings, SettingsError};

use self::tally::Tally;
use self::typist::Typist;
use super::manifest::MeterRules;
use super::read::held_file;
use super::{WordsError, io_error};

/// The two runs per language: the seed alone, which the gate judges, then the seed with a learner.
const RUNS: [(&str, bool); 2] = [("seed", false), ("learned", true)];
/// The day every word is learned on: the meter never purges, so one day serves a whole run.
const DAY: u32 = 0;
/// True in a build without optimisations, whose times are far slower than the product's.
const UNOPTIMISED: bool = cfg!(debug_assertions);
/// The run whose times count, which the speed refusal and `assets/README.md` name.
const RELEASE_RUN: &str = "cargo run --release --locked -p xtask -- words meter";
/// What every time line of an unoptimised build says.
const SLOW: &str = "unoptimised build, times are slow";
/// Why a run that measured no language fails.
const NOTHING: &str = "no language was measured";

/// What the command line asked for.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Opts<'a> {
    /// The packs folder, instead of the manifest's.
    pub(crate) packs: Option<&'a str>,
    /// The held-back sentences' folder, instead of the build's.
    pub(crate) held: Option<&'a str>,
    /// The chips in the lane, instead of the `[predict]` default.
    pub(crate) chips: Option<u8>,
    /// The one language to measure, instead of every shipped one.
    pub(crate) lang: Option<&'a str>,
    /// Time the scorer with a full learner instead of counting presses.
    pub(crate) speed: bool,
}

/// The folders the meter reads.
#[derive(Debug, Clone)]
pub(crate) struct Paths {
    /// The packs and their index.
    pub(crate) packs: PathBuf,
    /// The held-back sentences, one file per language.
    pub(crate) held: PathBuf,
}

/// Runs the meter as `opts` asks, telling `say` each line. An error names each gate language under the target or
/// not measured, each query kind over the time ceiling, or a run that measured nothing.
pub(crate) fn run(
    rules: &MeterRules,
    opts: &Opts<'_>,
    paths: &Paths,
    say: &mut dyn FnMut(&str),
) -> Result<(), WordsError> {
    let settings = settings(opts.chips)?;
    let asked = asked(&paths.packs, opts.lang)?;
    let (measured, mut failed) = if opts.speed {
        speed::run(rules, &settings, paths, &asked, say)?
    } else {
        presses(rules, &settings, paths, (&asked, opts.lang), say)?
    };
    if measured == 0 {
        failed.push(NOTHING.to_owned());
    }
    if !failed.is_empty() {
        return Err(WordsError::Meter(failed.join("; ")));
    }
    say(&if opts.speed {
        speed::passed(rules)
    } else {
        verdict::passed(rules)
    });
    Ok(())
}

/// What a time line ends with: a note in an unoptimised build, else nothing.
fn slow_note() -> String {
    if UNOPTIMISED {
        format!(" ({SLOW})")
    } else {
        String::new()
    }
}

/// The first `max` lines of the held file at `path`, the empty ones left out.
pub(crate) fn held_lines(path: &Path, max: usize) -> Result<Vec<String>, WordsError> {
    let file = File::open(path).map_err(|e| io_error(path, &e))?;
    let mut kept = Vec::new();
    for line in BufReader::new(file).lines().take(max) {
        let line = line.map_err(|e| io_error(path, &e))?;
        if !line.trim().is_empty() {
            kept.push(line);
        }
    }
    Ok(kept)
}

/// Types the held sentences of each language `asked` that has some, in both runs, printing a line for each. Gives
/// how many languages it measured and the gate's failures, judging every gate language unless `lang` was asked for.
fn presses(
    rules: &MeterRules,
    settings: &PredictSettings,
    paths: &Paths,
    (asked, lang): (&[String], Option<&str>),
    say: &mut dyn FnMut(&str),
) -> Result<(usize, Vec<String>), WordsError> {
    let held = |iso: &str| held_file(&paths.held, iso);
    let isos: Vec<String> = (asked.iter())
        .filter(|iso| held(iso).is_file())
        .cloned()
        .collect();
    let mut saved = BTreeMap::new();
    each_pack(paths, &isos, &mut |iso, pack| {
        let lines = held_lines(&held(iso), rules.max_sentences)?;
        for (run, learn) in RUNS {
            let tally = typed(iso, pack, settings, &lines, learn)?;
            say(&tally.line(iso, run, &rules.percentiles));
            if !learn {
                saved.insert(iso.to_owned(), tally.saved_pct());
            }
        }
        Ok(())
    })?;
    Ok((saved.len(), verdict::failures(rules, lang, &saved)))
}

/// The counts after typing `lines` in `iso` over `pack`, with a learner when `learn`.
fn typed(
    iso: &str,
    pack: &Pack<'_>,
    settings: &PredictSettings,
    lines: &[String],
    learn: bool,
) -> Result<Tally, WordsError> {
    let mut typist = Typist::new(iso, pack, settings, learn)?;
    for line in lines {
        typist.sentence(line);
    }
    Ok(typist.into_tally())
}

/// Calls `each` with every language of `isos` and its parsed pack.
fn each_pack(
    paths: &Paths,
    isos: &[String],
    each: &mut dyn FnMut(&str, &Pack<'_>) -> Result<(), WordsError>,
) -> Result<(), WordsError> {
    for iso in isos {
        let name = pack_file(iso).ok_or_else(|| WordsError::Lang(iso.clone()))?;
        let path = paths.packs.join(name);
        let bytes = std::fs::read(&path).map_err(|e| io_error(&path, &e))?;
        let pack = Pack::parse(&bytes).map_err(|e| WordsError::Pack(format!("{iso}: {e}")))?;
        each(iso, &pack)?;
    }
    Ok(())
}

/// The languages to measure: `lang` when the packs' index lists it, else every language it lists.
fn asked(packs: &Path, lang: Option<&str>) -> Result<Vec<String>, WordsError> {
    let path = packs.join(PACK_INDEX);
    let bytes = std::fs::read(&path).map_err(|e| io_error(&path, &e))?;
    let index = PackIndex::read(&bytes);
    let index = index.map_err(|e| WordsError::Pack(format!("{PACK_INDEX}: {e}")))?;
    match lang {
        None => Ok(index.packs().map(str::to_owned).collect()),
        Some(iso) if index.packs().any(|p| p == iso) => Ok(vec![iso.to_owned()]),
        Some(iso) => Err(WordsError::Lang(iso.to_owned())),
    }
}

/// The module's `[predict]` defaults, with `chips` chips when given and within their range.
fn settings(chips: Option<u8>) -> Result<PredictSettings, WordsError> {
    let bad = |e: SettingsError| WordsError::Meter(format!("[predict] {e}"));
    let mut settings = PredictSettings::from_defaults().map_err(bad)?;
    settings.chips = chips.unwrap_or(settings.chips);
    settings.check().map_err(bad)?;
    Ok(settings)
}
