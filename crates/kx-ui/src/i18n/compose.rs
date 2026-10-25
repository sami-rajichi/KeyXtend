//! Texts built from parts: a notice with its arguments, a list with "and N more", the space bar, the banner's count,
//! a pager's or the shortcuts layer's page, and a chip's name.

use super::I18n;
use super::keys::{
    BANNER_MORE, CHIP_FORGET_NAME, CHIP_NAME, LAYER_PAGE, LIST_COUNT, LIST_JOIN, LIST_MORE, MODULE,
    PAGE_OF, SPACE,
};
use fluent_bundle::{FluentArgs, FluentValue};
use kx_module_api::{Notice, NoticeArg, args, is_hidden};

/// `list.more`'s variable for the named items.
const LIST_VAR: &str = "list";
/// `lang.space`'s variable for the language.
const LANG_VAR: &str = "lang";
/// `lang.space`'s variable for the layout.
const LAYOUT_VAR: &str = "layout";
/// `page.of`'s variable for the page shown.
const AT_VAR: &str = "at";
/// `page.of`'s variable for how many pages there are.
const COUNT_VAR: &str = "count";
/// `key.layer-page`'s variable for the page shown.
const PAGE_VAR: &str = "page";
/// `key.layer-page`'s variable for how many pages there are.
const PAGES_VAR: &str = "pages";
/// `chip.name`'s variable for the chip's text.
const WORD_VAR: &str = "word";
/// The variable that plural forms select on.
const FORM_VAR: &str = "form";
/// Past this, a count takes the plural form of its last two digits.
const HUNDRED: usize = 100;
/// Last two digits below this keep the whole count, whose form past a hundred is the plain plural.
const LOW_TAIL: usize = 3;

impl I18n {
    /// The text of `notice`: a module id shows by its name, a count as a number, a list joined in this language.
    #[must_use]
    pub fn notice(&self, notice: &Notice) -> String {
        let more = notice.args.iter().find_map(|(name, arg)| match arg {
            NoticeArg::Count(n) if *name == args::MORE => Some(*n),
            _ => None,
        });
        let mut values = FluentArgs::new();
        for (name, arg) in &notice.args {
            let value = match arg {
                NoticeArg::Text(id) if *name == args::MODULE => {
                    self.name_of(MODULE, &clean(id)).into()
                }
                NoticeArg::Text(text) => clean(text).into(),
                NoticeArg::Count(n) => count(*n),
                NoticeArg::List(items) => self.list(items, more.unwrap_or(0)).into(),
            };
            values.set(*name, value);
        }
        self.format(notice.key, Some(&values))
    }

    /// `items` joined in this language, ending with "and `more` more" when `more` items are left out.
    /// With nothing to name it says only how many; an item that is all hidden marks is counted, not named.
    #[must_use]
    pub fn list(&self, items: &[String], more: usize) -> String {
        let named: Vec<String> = items
            .iter()
            .map(|i| clean(i))
            .filter(|i| !i.is_empty())
            .collect();
        let more = more.saturating_add(items.len() - named.len());
        let joined = named.join(&self.text(LIST_JOIN));
        if more == 0 {
            return joined;
        }
        let mut values = FluentArgs::new();
        values.set(args::MORE, count(more));
        values.set(FORM_VAR, form(more));
        if named.is_empty() {
            return self.format(LIST_COUNT, Some(&values));
        }
        values.set(LIST_VAR, joined);
        self.format(LIST_MORE, Some(&values))
    }

    /// How many more notices wait behind the one the banner shows; empty when none do.
    #[must_use]
    pub fn more(&self, n: usize) -> String {
        if n == 0 {
            return String::new();
        }
        let mut values = FluentArgs::new();
        values.set(args::MORE, count(n));
        values.set(FORM_VAR, form(n));
        self.format(BANNER_MORE, Some(&values))
    }

    /// A pager's number: page `at` of `pages`, both counted from 1.
    #[must_use]
    pub fn page(&self, at: usize, pages: usize) -> String {
        let mut values = FluentArgs::new();
        values.set(AT_VAR, count(at));
        values.set(COUNT_VAR, count(pages));
        self.format(PAGE_OF, Some(&values))
    }

    /// What readers hear after the Next key's name: the shortcuts layer's page `page` of `pages`, both from 1.
    #[must_use]
    pub fn layer_page(&self, page: usize, pages: usize) -> String {
        let mut values = FluentArgs::new();
        values.set(PAGE_VAR, count(page));
        values.set(PAGES_VAR, count(pages));
        self.format(LAYER_PAGE, Some(&values))
    }

    /// The name readers hear for a chip showing `word`, which loses its hidden marks first; in forget mode it says a
    /// click forgets the word.
    #[must_use]
    pub fn chip(&self, word: &str, forget: bool) -> String {
        let mut values = FluentArgs::new();
        values.set(WORD_VAR, clean(word));
        let key = if forget { CHIP_FORGET_NAME } else { CHIP_NAME };
        self.format(key, Some(&values))
    }

    /// The space bar's text from `parts`, the OS's names: the language, then the layout when known. Each loses its
    /// hidden marks, and a part that was only marks is left out.
    #[must_use]
    pub fn space(&self, parts: &[String]) -> String {
        let mut rest = parts.iter().map(|p| clean(p)).filter(|p| !p.is_empty());
        let first = rest.next().unwrap_or_default();
        rest.fold(first, |lang, layout| {
            let mut values = FluentArgs::new();
            values.set(LANG_VAR, lang);
            values.set(LAYOUT_VAR, layout);
            self.format(SPACE, Some(&values))
        })
    }
}

/// `text` without hidden marks (`is_hidden`), so an argument can neither split a text nor turn it around.
pub(crate) fn clean(text: &str) -> String {
    text.chars().filter(|c| !is_hidden(*c)).collect()
}

/// `n` as a Fluent number, capped so the plural rules never fail on a huge count.
fn count(n: usize) -> FluentValue<'static> {
    FluentValue::from(u32::try_from(n).unwrap_or(u32::MAX))
}

/// The number `n`'s plural form is chosen by: its last two digits past a hundred, when those are 3 or more.
/// CLDR reads Arabic by `n % 100`, which `intl_pluralrules` 7.0.2 drops; English and French forms stay the same.
fn form(n: usize) -> FluentValue<'static> {
    let tail = n % HUNDRED;
    let by_tail = n >= HUNDRED && tail >= LOW_TAIL;
    count(if by_tail { tail } else { n })
}
