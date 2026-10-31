//! The clipboard panel's accessible names, as kx-ui words them in each shipped language: a row's tick number after its
//! title, the count in Insert's label, and the pager's page count. The forms come from kx-ui's texts the config names.
#![cfg(windows)]

use crate::ftl::{self, Piece};
use crate::p8cfg::{G8Clip, G20Product};

/// A row's accessible name read back: the title it shows, and its tick number when ticked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowName {
    /// The row's title.
    pub title: String,
    /// Its tick number, none when unticked.
    pub tick: Option<u16>,
}

/// One language's forms: the text around Insert's count, around a tick's number, and the join before the tick note.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Lang {
    insert: (String, String),
    ticked: (String, String),
    join: String,
}

/// The forms of every shipped language, in kx-ui's file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Names(Vec<Lang>);

/// A count written as plain ASCII digits; none for anything else.
fn number(digits: &str) -> Option<u16> {
    let plain = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
    plain.then(|| digits.parse().ok()).flatten()
}

impl Names {
    /// The forms from `insert`, `ticked` and `join`, one entry per language, in the same order.
    pub fn new(
        insert: Vec<(String, String)>,
        ticked: Vec<(String, String)>,
        join: Vec<String>,
    ) -> Self {
        let langs = insert.into_iter().zip(ticked).zip(join);
        Self(
            langs
                .map(|((insert, ticked), join)| Lang {
                    insert,
                    ticked,
                    join,
                })
                .collect(),
        )
    }

    /// The forms of the texts `g` names, from kx-ui's translation files.
    pub fn load(g: &G20Product) -> Result<Self, String> {
        Ok(Self::new(
            ftl::around(&g.insert_name, &g.count_var)?,
            ftl::around(&g.ticked_name, &g.count_var)?,
            ftl::every(&g.join_name)?,
        ))
    }

    /// The count in Insert's label `name`, in any shipped language; none for a label without one.
    pub fn count(&self, name: &str) -> Option<u16> {
        self.0.iter().find_map(|l| {
            let (before, after) = &l.insert;
            number(
                name.strip_prefix(before.as_str())?
                    .strip_suffix(after.as_str())?,
            )
        })
    }

    /// Row name `name` read back: a trailing tick note in any shipped language gives its number, and the rest is the
    /// title; a name without one is all title.
    pub fn row(&self, name: &str) -> RowName {
        let ticked = |l: &Lang| {
            let (before, after) = &l.ticked;
            let rest = name.strip_suffix(after.as_str())?;
            let head = rest.trim_end_matches(|c: char| c.is_ascii_digit());
            let tick = number(&rest[head.len()..])?;
            let title = head
                .strip_suffix(before.as_str())?
                .strip_suffix(l.join.as_str())?;
            Some(RowName {
                title: title.to_string(),
                tick: Some(tick),
            })
        };
        let plain = || RowName {
            title: name.to_string(),
            tick: None,
        };
        self.0.iter().find_map(ticked).unwrap_or_else(plain)
    }
}

/// The pager's page number as kx-ui words it in each shipped language, and the variable that holds the page count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageNames {
    shapes: Vec<Vec<Piece>>,
    var: String,
}

/// The count of variable `var` in `name` when it reads as `shape`, every variable a run of ASCII digits; none else.
fn page_count(shape: &[Piece], name: &str, var: &str) -> Option<usize> {
    let (mut rest, mut found) = (name, None);
    for piece in shape {
        match piece {
            Piece::Words(w) => rest = rest.strip_prefix(w.as_str())?,
            Piece::Var(v) => {
                let digits =
                    rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
                let n = rest.get(..digits).filter(|d| !d.is_empty())?.parse().ok()?;
                found = (v == var).then_some(n).or(found);
                rest = &rest[digits..];
            }
        }
    }
    rest.is_empty().then_some(found).flatten()
}

impl PageNames {
    /// The forms `shapes`, one per language, whose page count is variable `var`; refused when one lacks it.
    pub fn new(shapes: Vec<Vec<Piece>>, var: &str) -> Result<Self, String> {
        let has = |s: &Vec<Piece>| s.iter().any(|p| matches!(p, Piece::Var(v) if v == var));
        if !shapes.iter().all(has) {
            return Err(format!("the page number lacks ${var} in some language"));
        }
        Ok(Self {
            shapes,
            var: var.to_string(),
        })
    }

    /// The forms of the text `g` names, from kx-ui's translation files.
    pub fn load(g: &G8Clip) -> Result<Self, String> {
        Self::new(ftl::shapes(&g.page_name)?, &g.page_var)
    }

    /// The page count in page-number text `name`, in any shipped language; none for any other text.
    pub fn count(&self, name: &str) -> Option<usize> {
        self.shapes
            .iter()
            .find_map(|s| page_count(s, name, &self.var))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Names {
        let g = crate::config::load().expect("loads").g20_product;
        Names::load(&g).expect("in all three")
    }

    fn row(title: &str, tick: Option<u16>) -> RowName {
        RowName {
            title: title.into(),
            tick,
        }
    }

    #[test]
    fn insert_counts_are_read_in_every_shipped_language() {
        let n = names();
        assert_eq!(n.count("Insert (2)"), Some(2));
        assert_eq!(n.count("Insérer (2)"), Some(2));
        assert_eq!(n.count("إدراج (2)"), Some(2));
        assert_eq!(n.count("Insert (12)"), Some(12));
        assert_eq!(n.count("Insert"), None, "the preview's plain Insert");
        assert_eq!(n.count("Insert ()"), None);
        assert_eq!(n.count("Insert (x)"), None);
        assert_eq!(n.count("Pin"), None);
    }

    #[test]
    fn a_rows_tick_number_comes_after_its_title_in_every_shipped_language() {
        let n = names();
        assert_eq!(n.row("Café crème, Ticked 1"), row("Café crème", Some(1)));
        assert_eq!(n.row("Café crème, Coché 2"), row("Café crème", Some(2)));
        assert_eq!(n.row("قَهْوَة، محدّد رقم 2"), row("قَهْوَة", Some(2)));
        assert_eq!(n.row("kx-g8clip-12"), row("kx-g8clip-12", None));
        assert_eq!(n.row("Ticked 3"), row("Ticked 3", None), "no join, no tick");
        assert_eq!(n.row(""), row("", None));
    }

    #[test]
    fn forms_given_by_hand_read_alike() {
        let n = Names::new(
            vec![("Add [".into(), "]".into())],
            vec![("#".into(), "!".into())],
            vec![" / ".into()],
        );
        assert_eq!(n.count("Add [3]"), Some(3));
        assert_eq!(n.row("tea / #4!"), row("tea", Some(4)));
        assert_eq!(n.row("tea / #!"), row("tea / #!", None), "no number");
    }

    #[test]
    fn the_page_count_is_read_in_every_shipped_language_and_nothing_else_is() {
        let g = crate::config::load().expect("loads").g8_clip;
        let pages = PageNames::load(&g).expect("in all three");
        assert_eq!(pages.count("3 of 200"), Some(200));
        assert_eq!(pages.count("3 sur 200"), Some(200));
        assert_eq!(pages.count("3 من 200"), Some(200));
        assert_eq!(pages.count("Insert (2)"), None);
        assert_eq!(pages.count("200"), None);
        assert_eq!(pages.count("x of 200"), None);
        assert_eq!(pages.count("3 of 200 more"), None);
        let lacking = PageNames::new(vec![vec![Piece::Var("at".into())]], "count");
        assert!(lacking.is_err());
    }
}
