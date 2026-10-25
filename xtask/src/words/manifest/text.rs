//! The manifest's text tables: how Tatoeba and Wikipedia are read and counted into word chains, each language's
//! text sources, and the credits written beside the packs.

use serde::Deserialize;

/// A share per mille is out of this many.
pub(crate) const PER_MILLE: u64 = 1_000;
/// A share in percent is out of this many.
pub(crate) const PERCENT: u32 = 100;

/// How text is read and counted: the `[text]` table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TextRules {
    /// A sentence ends at each of these characters when a space or the text's end follows.
    pub(crate) sentence_ends: Vec<char>,
    /// The shortest chain counted, in words.
    pub(crate) min_chain: u8,
    /// The longest chain counted, in words.
    pub(crate) max_chain: u8,
    /// How many times a Tatoeba count weighs a Wikipedia one.
    pub(crate) tatoeba_weight: u32,
    /// How many Tatoeba sentences per thousand are held back for the meter.
    pub(crate) held_back_per_mille: u64,
    /// The most chains counted at once before the rarest go.
    pub(crate) max_live_chains: usize,
    /// The most sentence keys remembered to skip repeats; past it new sentences are no longer remembered.
    pub(crate) max_seen_sentences: usize,
    /// A word over this many times as common in Tatoeba as in wordfreq has the Tatoeba chains it ends scaled to it.
    pub(crate) skew_ratio: f64,
    /// How many of the most seen one-word contexts the report shows.
    pub(crate) report_contexts: usize,
    /// How many followers it shows for each.
    pub(crate) report_followers: usize,
    /// The next words kept per context.
    pub(crate) followers: u8,
    /// What Tatoeba writes for an empty field.
    pub(crate) null: String,
    /// The folder, under cargo's target folder, that the held-back sentences go to.
    pub(crate) held: String,
    /// The folder, under the workspace root, that the packs go to unless `--out` names another.
    pub(crate) out: String,
}

/// The packs together: the `[packs]` table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Packs {
    /// The most KB that every file written may take together once deflated, as a download carries them.
    pub(crate) download_kb: u64,
    /// The deflate level the download size is measured at.
    pub(crate) level: u32,
}

/// The meter's rules: the `[meter]` table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MeterRules {
    /// The languages that must save `min_saved_pct` of the presses with the seed alone.
    pub(crate) gate: Vec<String>,
    /// The least share of presses, in percent, a gate language must save.
    pub(crate) min_saved_pct: u32,
    /// The most held-back sentences typed per language.
    pub(crate) max_sentences: usize,
    /// The percentiles of the time per guess that are printed, lowest first.
    pub(crate) percentiles: Vec<u32>,
    /// The longest the highest percentile of a `--speed` query kind may take, in microseconds.
    pub(crate) ceiling_us: u64,
    /// The words `--speed` learns to fill its learner.
    pub(crate) speed_learns: usize,
    /// The queries `--speed` times of each kind.
    pub(crate) speed_samples: usize,
}

/// One language's text: `[lang.<iso>.text]`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LangText {
    /// The most bytes, in KB, its chain sections may take.
    pub(crate) chain_budget_kb: u64,
    /// Its Tatoeba sentences, if any.
    pub(crate) tatoeba: Option<Tatoeba>,
    /// Its Wikipedia dump files, if any.
    pub(crate) wiki: Option<Wiki>,
}

/// One language's Tatoeba file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Tatoeba {
    /// The source file.
    pub(crate) file: String,
    /// The most lines read from it.
    pub(crate) max_sentences: u64,
}

/// One language's Wikipedia dump files.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Wiki {
    /// The source files, read in order.
    pub(crate) files: Vec<String>,
    /// The most bytes of article text read from them together.
    pub(crate) max_bytes: u64,
}

/// The files written beside the packs: the `[credits]` table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Credits {
    /// The credits file's name.
    pub(crate) file: String,
    /// The Tatoeba authors file's name.
    pub(crate) authors: String,
    /// The licence file's name.
    pub(crate) licence: String,
    /// The licence file's lines.
    pub(crate) licence_text: Vec<String>,
    /// wordfreq's licence section.
    pub(crate) wordfreq: Section,
    /// The notice of `SCOWLv2`, the English dictionary.
    pub(crate) scowl: Section,
    /// The works wordfreq is built on, whose SUBTLEX authors its licence says to credit.
    pub(crate) citations: Section,
}

/// A part of a source file copied word for word: from the line that starts with `from` to the first later line that
/// starts with `to`, that line left out, or to the file's end.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Section {
    /// The source file.
    pub(crate) file: String,
    /// How its first line starts.
    pub(crate) from: String,
    /// How the line after its last starts; without it the part runs to the file's end.
    pub(crate) to: Option<String>,
}

impl MeterRules {
    /// The highest percentile printed, which the speed ceiling judges.
    pub(crate) fn top(&self) -> u32 {
        self.percentiles.last().copied().unwrap_or_default()
    }
}

impl LangText {
    /// Every source file this language's text reads.
    pub(crate) fn files(&self) -> impl Iterator<Item = &str> {
        let tatoeba = self.tatoeba.iter().map(|t| t.file.as_str());
        let wiki = self
            .wiki
            .iter()
            .flat_map(|w| w.files.iter().map(String::as_str));
        tatoeba.chain(wiki)
    }
}
