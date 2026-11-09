//! The gate's own documents: the Notepad text with its lines in a fixed order, what Copy, Cut and Paste leave of it,
//! and the Chrome page whose script counts Copy's key-downs in its title.
#![cfg(windows)]

use serde_json::{Value, json};

use crate::launch::html_text;
use crate::p10cfg::G19Product;

/// The Notepad document's lines, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    /// Copied.
    Copy,
    /// Cut.
    Cut,
    /// Pasted over.
    Paste,
    /// The Arabic line.
    Arabic,
    /// The empty line for the caret's Paste.
    Caret,
}

impl Line {
    /// Every line, in the document's order.
    pub const ALL: [Self; 5] = [
        Self::Copy,
        Self::Cut,
        Self::Paste,
        Self::Arabic,
        Self::Caret,
    ];

    /// Its place, 0 the top.
    pub fn at(self) -> usize {
        Self::ALL
            .iter()
            .position(|l| *l == self)
            .unwrap_or_default()
    }

    /// Its text in `g`'s document.
    pub fn text(self, g: &G19Product) -> &str {
        match self {
            Self::Copy => &g.copy_line,
            Self::Cut => &g.cut_line,
            Self::Paste => &g.paste_line,
            Self::Arabic => &g.arabic,
            Self::Caret => "",
        }
    }
}

/// What joins Notepad's lines as the gate writes them.
pub const BREAK: &str = "\n";

/// The Notepad document.
pub fn doc(g: &G19Product) -> String {
    let lines: Vec<&str> = Line::ALL.iter().map(|l| l.text(g)).collect();
    lines.join(BREAK)
}

/// The document after Copy of the copy line, Cut of the cut line and Paste over the paste line, line by line.
pub fn edited(g: &G19Product) -> Vec<String> {
    let after = |l: Line| match l {
        Line::Cut => String::new(),
        Line::Paste => g.cut_line.clone(),
        other => other.text(g).to_string(),
    };
    Line::ALL.into_iter().map(after).collect()
}

/// Text `t` split into lines, whichever line ends the app writes.
pub fn lines_of(t: &str) -> Vec<String> {
    let t = t.replace("\r\n", BREAK).replace('\r', BREAK);
    t.split(BREAK).map(String::from).collect()
}

/// How `got` differs from `want`, in numbers only, never text: both line counts, the first line that differs, and
/// both lengths there in characters.
pub fn differ(got: &[String], want: &[String]) -> Value {
    let most = got.len().max(want.len());
    let first = (0..most).find(|&i| got.get(i) != want.get(i));
    let chars = |v: &[String]| first.and_then(|i| v.get(i)).map(|l| l.chars().count());
    json!({
        "lines": got.len(), "want_lines": want.len(), "first": first, "chars": chars(got), "want_chars": chars(want),
    })
}

/// The page script: each Ctrl+C key-down counted in the title after the copy mark, each with Shift after the other.
fn counter(g: &G19Product) -> String {
    let (code, copy, shift) = (&g.copy_code, &g.copy_mark, &g.shift_mark);
    format!(
        "<script>const t=document.title;let c=0,s=0;addEventListener('keydown',e=>{{if(e.ctrlKey&&e.code==='{code}')\
         {{e.shiftKey?s++:c++;document.title=t+'{copy}'+c+'{shift}'+s;}}}},true);</script>"
    )
}

/// The page body: the paragraph, the text box, the password box, the unknown box and the key-down counter.
pub fn page(g: &G19Product) -> String {
    let (h, name) = (html_text, |n: &str| {
        format!("aria-label=\"{}\"", html_text(n))
    });
    format!(
        "<div style=\"{}\"><p>{}</p>\n<p><input {} value=\"{}\" size=\"{}\"></p>\n<p><input type=\"password\" {} \
         value=\"{}\"></p>\n<div role=\"{}\" {} tabindex=\"0\" contenteditable=\"true\">{}</div></div>\n{}",
        h(&g.css),
        h(&g.page_text),
        name(&g.box_name),
        h(&g.box_text),
        g.box_size,
        name(&g.pass_name),
        h(&g.pass_value),
        h(&g.unknown_role),
        name(&g.unknown_name),
        h(&g.box_text),
        counter(g)
    )
}

/// The number at the start of `s`, and what follows it.
fn number(s: &str) -> Option<(usize, &str)> {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    Some((s[..end].parse().ok()?, &s[end..]))
}

/// The Ctrl+C and the Ctrl+Shift+C key-downs in window title `title`, read from the last copy mark on.
pub fn counts(title: &str, g: &G19Product) -> Option<(usize, usize)> {
    let at = title.rfind(g.copy_mark.as_str())?;
    let (copies, rest) = number(&title[at + g.copy_mark.len()..])?;
    let (shifted, _) = number(rest.strip_prefix(g.shift_mark.as_str())?)?;
    Some((copies, shifted))
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use proptest::prelude::*;

    use super::*;
    use crate::config::load;

    #[test]
    fn the_document_holds_each_line_at_its_place_and_ends_on_the_empty_line() {
        let g = load().expect("loads").g19_product;
        let text = doc(&g);
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(lines.len(), Line::ALL.len());
        for line in Line::ALL {
            assert_eq!(lines[line.at()], line.text(&g), "{line:?}");
        }
        assert_eq!(Line::Caret.text(&g), "");
        assert_eq!(Line::Copy.text(&g), g.copy_line);
    }

    #[test]
    fn copy_keeps_cut_empties_and_paste_takes_the_cut_line() {
        let g = load().expect("loads").g19_product;
        let after = edited(&g);
        assert_eq!(after[Line::Copy.at()], g.copy_line);
        assert_eq!(after[Line::Cut.at()], "");
        assert_eq!(after[Line::Paste.at()], g.cut_line);
        assert_eq!(after[Line::Arabic.at()], g.arabic);
    }

    #[test]
    fn a_document_that_differs_is_told_in_numbers_only() {
        let s = |v: &[&str]| v.iter().map(ToString::to_string).collect::<Vec<_>>();
        let want = s(&["Copy", "", "Coupez", "حدد", ""]);
        let extra = differ(&s(&["Copy", "", "Coupez", "حدد", "", ""]), &want);
        assert_eq!(
            extra,
            json!({ "lines": 6, "want_lines": 5, "first": 5, "chars": 0, "want_chars": null })
        );
        let short = differ(&s(&["Copy", "", "Paste", "حدد", ""]), &want);
        assert_eq!(
            (&short["first"], &short["chars"], &short["want_chars"]),
            (&json!(2), &json!(5), &json!(6))
        );
        assert_eq!(differ(&want, &want)["first"], json!(null), "the same");
        assert!(!extra.to_string().contains("Copy"), "no text");
    }

    #[test]
    fn the_page_names_each_box_and_counts_copy_in_its_title() {
        let g = load().expect("loads").g19_product;
        let body = page(&g);
        for name in [&g.box_name, &g.pass_name, &g.unknown_name] {
            assert!(body.contains(&format!("aria-label=\"{name}\"")), "{name}");
        }
        assert!(body.contains("type=\"password\"") && body.contains(&g.page_text));
        assert!(body.contains(&format!("role=\"{}\"", g.unknown_role)));
        assert!(body.contains(&format!("size=\"{}\"", g.box_size)));
        assert!(body.contains(&g.copy_code) && body.contains(&g.shift_mark));
        let title = format!(
            "KeyXtend probe 1{}3{}0 - Google Chrome",
            g.copy_mark, g.shift_mark
        );
        assert_eq!(counts(&title, &g), Some((3, 0)));
        assert_eq!(counts("KeyXtend probe 1", &g), None);
        let late = format!(
            "x{}1{}2{}4{}0",
            g.copy_mark, g.shift_mark, g.copy_mark, g.shift_mark
        );
        assert_eq!(counts(&late, &g), Some((4, 0)), "the last mark counts");
    }

    /// The shipped settings, read once for the property test.
    static G: LazyLock<G19Product> = LazyLock::new(|| load().expect("loads").g19_product);

    proptest! {
        #[test]
        fn the_last_marks_counts_read_back_and_any_title_reads_without_a_panic(
            copies in 0usize..100_000,
            shifted in 0usize..100_000,
            lead in "\\PC{0,20}",
            tail in "[A-Za-z -]{0,20}",
            junk in "\\PC{0,40}",
        ) {
            let g = &*G;
            let title = format!("{lead}{}{copies}{}{shifted}{tail}", g.copy_mark, g.shift_mark);
            prop_assert_eq!(counts(&title, g), Some((copies, shifted)));
            let _ = counts(&junk, g);
        }
    }
}
