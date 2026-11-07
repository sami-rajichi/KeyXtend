//! The kinds and labels tables (design "The notebook"): each kind's icon token, filter and starter labels, and each
//! label's hidden start and `@@` feed. Both are compiled in; their ids name the texts, never English in logic.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use kx_module_api::ports::{EntryKind, FillFilter};
use serde::Deserialize;
use serde::de::IntoDeserializer;
use serde::de::value::Error as ValueError;
use thiserror::Error;

/// The kinds table, compiled in so it is never read from disk.
const KINDS: &str = include_str!("../kinds.toml");
/// The labels table, compiled in so it is never read from disk.
const LABELS: &str = include_str!("../labels.toml");
/// What joins the words of an id.
const DASH: char = '-';
/// What joins the words of an icon token.
const UNDERSCORE: char = '_';

/// The tables, parsed once; a broken table would hold no kind or label, and a test keeps it sound.
static TABLES: LazyLock<Tables> =
    LazyLock::new(|| Tables::parse(KINDS, LABELS).unwrap_or_default());

/// The shipped tables.
#[must_use]
pub fn tables() -> &'static Tables {
    &TABLES
}

/// Why a table is unusable; it names table ids only.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum TableError {
    /// The table is not TOML of the expected shape.
    #[error("the table does not parse: {0}")]
    Parse(String),
    /// A kind is missing, listed twice or unknown.
    #[error("the kinds table must list every kind once")]
    Kinds,
    /// A label id is listed twice or is not lower-case words joined by single dashes.
    #[error("label {0:?} is listed twice or badly named")]
    Label(String),
    /// A kind's starter label is unknown or listed twice.
    #[error("kind {0:?} starts with an unknown or repeated label")]
    Starter(String),
    /// A kind's icon token is not lower-case words joined by underscores.
    #[error("kind {0:?} names a bad icon token")]
    Icon(String),
    /// A kind's pair names an unknown label, or one label twice.
    #[error("kind {0:?} pairs an unknown or repeated label")]
    Pair(String),
}

/// A kind as the file names it; `EntryKind` takes each.
#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Kind {
    Person,
    Login,
    Wifi,
    Card,
    Note,
    Other,
}

impl From<Kind> for EntryKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Person => Self::Person,
            Kind::Login => Self::Login,
            Kind::Wifi => Self::Wifi,
            Kind::Card => Self::Card,
            Kind::Note => Self::Note,
            Kind::Other => Self::Other,
        }
    }
}

/// The filter chip that lists a kind, as the file names it; `all` is All alone.
#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Chip {
    All,
    People,
    Logins,
    Wifi,
    Cards,
    Notes,
}

impl From<Chip> for FillFilter {
    fn from(chip: Chip) -> Self {
        match chip {
            Chip::All => Self::All,
            Chip::People => Self::People,
            Chip::Logins => Self::Logins,
            Chip::Wifi => Self::Wifi,
            Chip::Cards => Self::Cards,
            Chip::Notes => Self::Notes,
        }
    }
}

/// The kinds file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KindFile {
    kind: Vec<KindLine>,
}

/// One kind as the file writes it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KindLine {
    id: String,
    icon: String,
    filter: Chip,
    fields: Vec<String>,
    #[serde(default)]
    pair: Option<[String; 2]>,
}

/// The labels file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LabelFile {
    label: Vec<LabelRow>,
}

/// One kind: its id, icon token, filter and starter labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindRow {
    /// The kind.
    pub kind: EntryKind,
    /// Its id, which names its text and the kind in the saved notebook.
    pub id: String,
    /// Its icon token in `icons.toml [fill]`.
    pub icon: String,
    /// The filter that lists it beside All.
    pub filter: FillFilter,
    /// Its starter fields' label ids, in order.
    pub fields: Vec<String>,
    /// The labels its User and Password row buttons type, if it has them; Both types the first, a Tab, then the second.
    pub pair: Option<[String; 2]>,
}

/// One label of the table.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelRow {
    /// Its id, which names its text and the label in the saved notebook.
    pub id: String,
    /// Its field starts hidden.
    #[serde(default)]
    pub hidden: bool,
    /// Its values show in the `@@` list.
    #[serde(default)]
    pub emails: bool,
    /// Its field offers Make one on the New and Edit pages.
    #[serde(default)]
    pub make: bool,
}

/// The kinds, in `EntryKind::ALL`'s order, and the labels, in Add a field's order.
#[derive(Debug, Default)]
pub struct Tables {
    kinds: Vec<KindRow>,
    labels: Vec<LabelRow>,
}

impl Tables {
    /// Reads and checks the kinds table `kinds` and the labels table `labels`.
    ///
    /// # Errors
    /// The first `TableError` found.
    pub fn parse(kinds: &str, labels: &str) -> Result<Self, TableError> {
        let parse = |e: toml::de::Error| TableError::Parse(e.to_string());
        let labels = toml::from_str::<LabelFile>(labels).map_err(parse)?.label;
        check_labels(&labels)?;
        let lines = toml::from_str::<KindFile>(kinds).map_err(parse)?.kind;
        let mut rows = Vec::with_capacity(lines.len());
        for line in lines {
            rows.push(kind_row(line, &labels)?);
        }
        let mut kinds = Vec::with_capacity(EntryKind::ALL.len());
        for kind in EntryKind::ALL {
            let mut of = rows.iter().filter(|r| r.kind == kind);
            let row = of.next().ok_or(TableError::Kinds)?;
            kinds.push(row.clone());
        }
        if rows.len() != kinds.len() {
            return Err(TableError::Kinds);
        }
        Ok(Self { kinds, labels })
    }

    /// Every kind, in `EntryKind::ALL`'s order.
    #[must_use]
    pub fn kinds(&self) -> &[KindRow] {
        &self.kinds
    }

    /// Kind `kind`'s row.
    #[must_use]
    pub fn kind(&self, kind: EntryKind) -> Option<&KindRow> {
        self.kinds.iter().find(|r| r.kind == kind)
    }

    /// The kind whose id is `id`.
    #[must_use]
    pub fn kind_by_id(&self, id: &str) -> Option<&KindRow> {
        self.kinds.iter().find(|r| r.id == id)
    }

    /// Every label, in Add a field's order.
    #[must_use]
    pub fn labels(&self) -> &[LabelRow] {
        &self.labels
    }

    /// The label whose id is `id`.
    #[must_use]
    pub fn label(&self, id: &str) -> Option<&LabelRow> {
        self.labels.iter().find(|l| l.id == id)
    }
}

/// `line` as a row: a known kind, a good icon token, and known starter labels, each once.
fn kind_row(line: KindLine, labels: &[LabelRow]) -> Result<KindRow, TableError> {
    let id: &str = &line.id;
    let kind = Kind::deserialize(IntoDeserializer::<ValueError>::into_deserializer(id))
        .map_err(|_| TableError::Kinds)?;
    if !is_words(&line.icon, UNDERSCORE) {
        return Err(TableError::Icon(line.id));
    }
    let mut seen = BTreeSet::new();
    let known = |f: &String| labels.iter().any(|l| &l.id == f);
    if !line.fields.iter().all(|f| known(f) && seen.insert(f)) {
        return Err(TableError::Starter(line.id));
    }
    let paired = |[a, b]: &[String; 2]| known(a) && known(b) && a != b;
    if !line.pair.as_ref().is_none_or(paired) {
        return Err(TableError::Pair(line.id));
    }
    Ok(KindRow {
        kind: kind.into(),
        id: line.id,
        icon: line.icon,
        filter: line.filter.into(),
        fields: line.fields,
        pair: line.pair,
    })
}

/// Refuses a label id listed twice or not made of words joined by dashes.
fn check_labels(labels: &[LabelRow]) -> Result<(), TableError> {
    let mut seen = BTreeSet::new();
    let bad = labels
        .iter()
        .find(|l| !is_words(&l.id, DASH) || !seen.insert(&l.id));
    bad.map_or(Ok(()), |l| Err(TableError::Label(l.id.clone())))
}

/// True when `text` is lower-case ASCII words of letters and digits, each joined to the next by one `joint`.
fn is_words(text: &str, joint: char) -> bool {
    let word = |w: &str| {
        !w.is_empty()
            && w.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    };
    text.split(joint).all(word)
}

#[cfg(test)]
mod tests;
