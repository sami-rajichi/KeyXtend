//! One row of the models table: its checks, and what it hands the ports (the model a job loads, the card it asks, the
//! download it fetches).

use std::collections::BTreeMap;
use std::path::Path;

use kx_module_api::Redacted;
use kx_module_api::ports::{
    CardId, CardUse, FetchAsk, Fingerprint, JobLimits, LINK_SCHEME, Link, SpeechModel, TAG_LEN,
    fits_iso, host_ok, link_named,
};
use serde::Deserialize;
use thiserror::Error;

use super::{Rules, file_ok};
use crate::plain::one_line;

/// Why a row was refused, in the order the rules are checked.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum RowError {
    /// The shown name or the short name is empty or holds a hidden mark.
    #[error("its name or short name is empty or holds a hidden mark")]
    Name,
    /// The runner is not one the table lists, or not a tag.
    #[error("its runner is not listed")]
    Runner,
    /// The file is no plain model file name.
    #[error("its file is no plain model file name")]
    File,
    /// The size is zero.
    #[error("its size is zero")]
    Bytes,
    /// The SHA-256 is not 64 lower-case hex digits.
    #[error("its SHA-256 is not 64 lower-case hex digits")]
    Print,
    /// An address is not HTTPS at a fixed revision on a host listed by name, ending in the file, or there is none; or
    /// a listed host is none `host_ok` takes.
    #[error("an address is not https at a fixed revision on a listed host")]
    Link,
    /// The licence is not one the table lists.
    #[error("its licence is not listed")]
    Licence,
    /// The credit is empty, or the original's page is no https link.
    #[error("its credit or original page is missing")]
    Credit,
    /// The memory on the processor or the card is outside the ports' ranges.
    #[error("its memory is outside its range")]
    Memory,
    /// It names no language, a language that is no ISO 639 code, or a code that is no tag.
    #[error("its languages are missing or malformed")]
    Langs,
}

/// A row as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawRow {
    /// The id `picks` stores.
    pub(super) id: String,
    /// The shown name.
    name: String,
    /// The bar's name for it.
    short: String,
    /// The runner that reads it.
    runner: String,
    /// The file's name.
    file: String,
    /// The file's size.
    bytes: u64,
    /// The file's SHA-256 in lower-case hex.
    sha256: String,
    /// The addresses, tried in order.
    links: Vec<String>,
    /// The hosts a link or a redirect may reach: a link names one exactly; `.` and a site's domain take those under it.
    hosts: Vec<String>,
    /// The licence.
    licence: String,
    /// Who to credit.
    credit: String,
    /// The original model's page.
    origin: String,
    /// The system memory a job needs, in MB.
    cpu_mb: u32,
    /// The card memory it takes, in MB.
    card_mb: u32,
    /// It can write while hearing.
    live: bool,
    /// It may head an order.
    lead: bool,
    /// Each keyboard language and the model's own code for it.
    langs: BTreeMap<String, String>,
}

/// A checked row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The id `picks` stores.
    pub id: String,
    /// The shown name.
    pub name: String,
    /// The bar's name for it, which no other row has.
    pub short: String,
    /// The runner that reads it, as the speech port names it.
    pub runner: String,
    /// The file's name in the models folder.
    pub file: String,
    /// The file's size, checked before it takes its name.
    pub bytes: u64,
    /// The file's SHA-256, which is what is trusted.
    pub print: Fingerprint,
    /// HTTPS addresses at a fixed revision, tried in order.
    pub links: Vec<String>,
    /// The hosts a link or a redirect may reach: a link names one exactly; `.` and a site's domain take those under it.
    pub hosts: Vec<String>,
    /// The licence the user receives it under.
    pub licence: String,
    /// Who the credit line names.
    pub credit: String,
    /// The original model's page.
    pub origin: String,
    /// The system memory a job needs for it, in MB, beside `job_mb`.
    pub cpu_mb: u32,
    /// The card memory it takes, in MB, beside `card_margin_mb`.
    pub card_mb: u32,
    /// It can write while hearing (recorded, unused).
    pub live: bool,
    /// It may head an order; false keeps it from ever being a default.
    pub lead: bool,
    /// Each keyboard language (ISO 639) and the model's own code for it.
    pub langs: BTreeMap<String, String>,
}

impl Row {
    /// Checks `raw` against `rules` and keeps it.
    pub(super) fn checked(raw: RawRow, rules: &Rules<'_>) -> Result<Self, RowError> {
        if let Some(why) = raw.fault(rules) {
            return Err(why);
        }
        let print = print_of(&raw.sha256).ok_or(RowError::Print)?;
        Ok(Self {
            id: raw.id,
            name: raw.name,
            short: raw.short,
            runner: raw.runner,
            file: raw.file,
            bytes: raw.bytes,
            print,
            links: raw.links,
            hosts: raw.hosts,
            licence: raw.licence,
            credit: raw.credit,
            origin: raw.origin,
            cpu_mb: raw.cpu_mb,
            card_mb: raw.card_mb,
            live: raw.live,
            lead: raw.lead,
            langs: raw.langs,
        })
    }

    /// The model's own code for keyboard language `lang`, if it knows it.
    #[must_use]
    pub fn code(&self, lang: &str) -> Option<&str> {
        self.langs.get(lang).map(String::as_str)
    }

    /// The model a job loads from `folder` for `lang`; `None` when it does not know the language.
    #[must_use]
    pub fn model(&self, folder: &Path, lang: &str) -> Option<SpeechModel> {
        Some(SpeechModel {
            file: Redacted::new(folder.join(&self.file)),
            lang: self.code(lang)?.to_owned(),
            runner: self.runner.clone(),
            cpu_mb: self.cpu_mb,
        })
    }

    /// What card `id` must hold for it, with `margin_mb` kept free beside it.
    #[must_use]
    pub fn card(&self, id: CardId, margin_mb: u32) -> CardUse {
        CardUse {
            id,
            need_mb: self.card_mb,
            margin_mb,
        }
    }

    /// Its download into `folder`, within `job`, keeping `min_free_mb` free and failing after `stall_s` without a byte.
    #[must_use]
    pub fn fetch(
        &self,
        folder: &Path,
        job: JobLimits,
        min_free_mb: u32,
        stall_s: u16,
    ) -> Option<FetchAsk> {
        let links = (self.links.iter())
            .map(|l| Link::new(l.clone()).ok())
            .collect::<Option<Vec<_>>>()?;
        Some(FetchAsk {
            job,
            links,
            hosts: self.hosts.clone(),
            file: Redacted::new(folder.join(&self.file)),
            bytes: self.bytes,
            print: self.print,
            min_free_mb,
            stall_s,
        })
    }
}

impl RawRow {
    /// The first rule the row breaks, if any.
    fn fault(&self, rules: &Rules<'_>) -> Option<RowError> {
        let memory =
            SpeechModel::CPU_MB.contains(&self.cpu_mb) && CardUse::NEED_MB.contains(&self.card_mb);
        let credit = one_line(&self.credit) && Link::new(self.origin.clone()).is_ok();
        let checks = [
            (
                one_line(&self.name) && one_line(&self.short),
                RowError::Name,
            ),
            (
                rules.runners.contains(&self.runner) && tag_ok(&self.runner),
                RowError::Runner,
            ),
            (file_ok(&self.file, rules.exts), RowError::File),
            (self.bytes > 0, RowError::Bytes),
            (print_of(&self.sha256).is_some(), RowError::Print),
            (self.links_ok(rules.revision), RowError::Link),
            (rules.licences.contains(&self.licence), RowError::Licence),
            (credit, RowError::Credit),
            (memory, RowError::Memory),
            (self.langs_ok(), RowError::Langs),
        ];
        checks
            .into_iter()
            .find_map(|(ok, why)| (!ok).then_some(why))
    }

    /// True when there is an address and each is good, and each listed host is one a download may list.
    fn links_ok(&self, revision: usize) -> bool {
        let hosts = !self.hosts.is_empty() && self.hosts.iter().all(|h| host_ok(h));
        hosts && !self.links.is_empty() && self.links.iter().all(|l| self.link_ok(l, revision))
    }

    /// True for an https link to a host listed by name (`link_named`, the port's own rule), at a commit of `revision`
    /// hex digits and ending in the file.
    fn link_ok(&self, link: &str, revision: usize) -> bool {
        let Some((_, path)) = link
            .strip_prefix(LINK_SCHEME)
            .and_then(|r| r.split_once('/'))
        else {
            return false;
        };
        let hex = |b: u8| b.is_ascii_digit() || (b'a'..=b'f').contains(&b);
        let fixed = path
            .split('/')
            .any(|p| p.len() == revision && p.bytes().all(hex));
        let named = path.rsplit('/').next() == Some(self.file.as_str());
        let listed = link_named(&self.hosts, link);
        Link::new(link.to_owned()).is_ok() && listed && fixed && named
    }

    /// True when it names a language and each is an ISO 639 code with a tag for the model's code.
    fn langs_ok(&self) -> bool {
        let pair = |(lang, code): (&String, &String)| fits_iso(lang) && tag_ok(code);
        !self.langs.is_empty() && self.langs.iter().all(pair)
    }
}

/// The fingerprint `sha256` spells, only in the lower-case hex the table writes.
fn print_of(sha256: &str) -> Option<Fingerprint> {
    Fingerprint::from_hex(sha256).filter(|p| p.hex() == sha256)
}

/// True for a tag the frames carry: within `TAG_LEN`, visible ASCII only.
fn tag_ok(tag: &str) -> bool {
    TAG_LEN.contains(&tag.len()) && tag.bytes().all(|b| b.is_ascii_graphic())
}
