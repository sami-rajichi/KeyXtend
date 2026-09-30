//! The UI's texts: Fluent files compiled in, the language choice, and lookups that fall back to English (ADR-0017).
//! A key `group.name` in code is message `group`, attribute `name`, since Fluent ids cannot hold dots.

mod compose;
pub mod keys;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use unic_langid::LanguageIdentifier;

/// One language the UI speaks, with its Fluent file.
#[derive(Debug)]
pub struct Lang {
    /// Its BCP-47 tag, the value the `[ui] language` setting names.
    pub tag: &'static str,
    /// It is written right to left, so the UI mirrors.
    pub rtl: bool,
    /// Its Fluent file, compiled in.
    pub ftl: &'static str,
}

/// Every language, the fallback first.
pub static LANGS: [Lang; 3] = [
    Lang {
        tag: "en",
        rtl: false,
        ftl: include_str!("../i18n/en.ftl"),
    },
    Lang {
        tag: "fr",
        rtl: false,
        ftl: include_str!("../i18n/fr.ftl"),
    },
    Lang {
        tag: "ar",
        rtl: true,
        ftl: include_str!("../i18n/ar.ftl"),
    },
];

/// The `[ui] language` value that follows the Windows display language.
pub const AUTO: &str = "auto";

/// Where a key's group ends and its name starts.
const DOT: char = '.';

/// What separates a language tag's parts: BCP-47's hyphen, and the underscore some systems write.
const TAG_SEPARATORS: [char; 2] = ['-', '_'];

/// A language's texts, safe to share between threads.
type Bundle = FluentBundle<FluentResource>;

/// The language for the `[ui] language` value `setting` when Windows shows `os_tag`.
/// A supported tag wins; any other value, `auto` included, follows `os_tag`, else English.
#[must_use]
pub fn pick(setting: &str, os_tag: &str) -> &'static Lang {
    find(setting).unwrap_or_else(|| negotiate(os_tag))
}

/// The language with tag `tag`, if the UI has it.
#[must_use]
pub fn find(tag: &str) -> Option<&'static Lang> {
    LANGS.iter().find(|l| l.tag == tag)
}

/// The language closest to `os_tag`, such as French for `fr-CA`, else the fallback.
fn negotiate(os_tag: &str) -> &'static Lang {
    let wanted: Vec<LanguageIdentifier> = langid(os_tag).into_iter().collect();
    let have: Vec<LanguageIdentifier> = LANGS
        .iter()
        .map(|l| langid(l.tag).unwrap_or_default())
        .collect();
    let best = negotiate_languages(&wanted, &have, None, NegotiationStrategy::Lookup);
    let at = best
        .first()
        .and_then(|id| have.iter().position(|h| h == *id));
    &LANGS[at.unwrap_or(0)]
}

/// `tag` as a language id; a tag with extensions, such as `fr-FR-u-ca-gregory`, is read by its first part.
fn langid(tag: &str) -> Option<LanguageIdentifier> {
    tag.parse()
        .ok()
        .or_else(|| tag.split(TAG_SEPARATORS).next()?.parse().ok())
}

/// The message and the attribute `key` names; a key without a dot names a message's own text.
fn split(key: &str) -> (&str, Option<&str>) {
    match key.split_once(DOT) {
        Some((group, name)) => (group, Some(name)),
        None => (key, None),
    }
}

/// The key of `id` in `group`.
fn join(group: &str, id: &str) -> String {
    format!("{group}{DOT}{id}")
}

/// `lang`'s texts; a broken entry is left out and the rest load, so a bad line never stops the UI.
fn bundle(lang: &Lang) -> Bundle {
    let res = FluentResource::try_new(lang.ftl.to_owned()).unwrap_or_else(|(res, _)| res);
    let mut b = Bundle::new_concurrent(vec![langid(lang.tag).unwrap_or_default()]);
    // Each argument sits between bidi isolation marks, so Latin words keep their order inside Arabic text.
    b.set_use_isolating(true);
    // A key written twice keeps its first text; the file tests refuse that.
    let _ = b.add_resource(res);
    b
}

/// The first text of `key` among `bundles`, with `args`; Fluent marks a missing argument in the text itself.
fn find_in(bundles: &[Bundle], key: &str, args: Option<&FluentArgs>) -> Option<String> {
    let (id, attr) = split(key);
    bundles.iter().find_map(|b| {
        let message = b.get_message(id)?;
        let pattern = match attr {
            Some(name) => message.get_attribute(name)?.value(),
            None => message.value()?,
        };
        let mut errors = Vec::new();
        Some(b.format_pattern(pattern, args, &mut errors).into_owned())
    })
}

/// The texts of one language, with English behind it for any key it lacks.
pub struct I18n {
    /// The language shown.
    lang: &'static Lang,
    /// Its texts first, then the fallback's.
    bundles: Vec<Bundle>,
}

impl I18n {
    /// The texts of `lang`, falling back to English.
    #[must_use]
    pub fn new(lang: &'static Lang) -> Self {
        Self::of(lang, &LANGS[0])
    }

    /// The texts of `lang`, falling back to `fallback`.
    fn of(lang: &'static Lang, fallback: &'static Lang) -> Self {
        let mut langs = vec![lang];
        if fallback.tag != lang.tag {
            langs.push(fallback);
        }
        Self {
            lang,
            bundles: langs.into_iter().map(bundle).collect(),
        }
    }

    /// The language shown.
    #[must_use]
    pub fn lang(&self) -> &'static Lang {
        self.lang
    }

    /// True when the language itself, without the fallback, has a text for `key`.
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        find_in(self.bundles.get(..1).unwrap_or_default(), key, None).is_some()
    }

    /// The text of `key`; a key no language has shows as itself.
    #[must_use]
    pub fn text(&self, key: &str) -> String {
        self.format(key, None)
    }

    /// The text of `id` in `group`, such as a module or theme name, or `id` itself when no language has it.
    #[must_use]
    pub fn name_of(&self, group: &str, id: &str) -> String {
        find_in(&self.bundles, &join(group, id), None).unwrap_or_else(|| id.to_owned())
    }

    /// The text of `key` with `args`, or the key itself.
    fn format(&self, key: &str, args: Option<&FluentArgs>) -> String {
        find_in(&self.bundles, key, args).unwrap_or_else(|| key.to_owned())
    }
}

#[cfg(test)]
mod tests;
