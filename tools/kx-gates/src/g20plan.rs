//! G20's pure parts: the rows the gate's copies show, what a search finds, the rows its ticks pick, the text Notepad
//! holds after Insert, and the rows check, which logs no text but the gate's own.
#![cfg(windows)]

use kx_mod_clipboard::insert::JoinConfig;
use kx_module_api::text::fold_any;
use serde_json::{Value, json};

use crate::p8cfg::G20Product;

/// Windows' line end, which the product's clipboard writes for each break and Notepad saves.
const CRLF: &str = "\r\n";
/// The neutral line break the join holds.
const LF: &str = "\n";

/// The rows the kept copies `texts`, oldest first, show: newest first.
pub fn shown(texts: &[&str]) -> Vec<String> {
    texts.iter().rev().map(|t| (*t).to_string()).collect()
}

/// The rows of `rows` a search for `typed` finds, in their order: both folded as the product folds them, the query's
/// blanks around it gone.
pub fn found(rows: &[String], typed: &str) -> Vec<String> {
    let query = fold_any(typed);
    let query = query.trim();
    let finds = |r: &&String| fold_any(r).contains(query);
    rows.iter().filter(finds).cloned().collect()
}

/// The rows G20's ticks pick, in tick order: each ticked search's first row among the copies' rows.
pub fn ticked(g: &G20Product) -> Vec<String> {
    let rows = shown(&g.texts());
    let ticks = g.searches.iter().filter(|s| s.tick);
    ticks
        .filter_map(|s| found(&rows, &s.typed).into_iter().next())
        .collect()
}

/// What an insert of `texts` pastes: the texts in order, `join`'s text between two.
pub fn pasted(texts: &[String], join: &JoinConfig) -> String {
    texts.join(&join.text())
}

/// True when Notepad's `saved` text is `want` exactly, its Windows line ends read as the join's breaks.
pub fn saved_as(saved: &str, want: &str) -> bool {
    saved.replace(CRLF, LF) == want
}

/// The texts of `hidden` that one of `rows` shows.
pub fn leaks<'a>(rows: &[String], hidden: &[&'a str]) -> Vec<&'a str> {
    let shows = |h: &&str| rows.iter().any(|r| r == h);
    hidden.iter().copied().filter(shows).collect()
}

/// The rows check on titles `got` against `want`: whether they match, which `hidden` texts leaked, and how many rows
/// hold none of `ours`, so no text but the gate's own is logged.
pub fn rows_part(got: &[String], want: &[String], hidden: &[&str], ours: &[&str]) -> Value {
    let unknown = got.iter().filter(|r| !ours.contains(&r.as_str())).count();
    let leaked = leaks(got, hidden);
    let ok = got == want && leaked.is_empty();
    json!({ "rows": got.len(), "rows_ok": got == want, "leaks": leaked, "unknown": unknown, "ok": ok })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: &str = "Morning coffee notes";
    const FR: &str = "Café crème à midi";
    const AR: &str = "قَهْوَةُ الصَّبَاحِ";

    fn join(breaks: u8, rule: &str) -> JoinConfig {
        JoinConfig {
            breaks,
            rule: rule.into(),
        }
    }

    #[test]
    fn the_rows_show_the_kept_copies_newest_first() {
        assert_eq!(shown(&[EN, FR, AR]), [AR, FR, EN]);
        assert!(shown(&[]).is_empty());
    }

    #[test]
    fn a_search_finds_rows_ignoring_case_accents_and_harakat_in_their_order() {
        let rows = shown(&[EN, FR, AR]);
        assert_eq!(found(&rows, "cafe"), [FR]);
        assert_eq!(found(&rows, "CAFÉ"), [FR], "case and accent fold");
        assert_eq!(found(&rows, "قهوة"), [AR], "the copy's harakat fold");
        assert_eq!(found(&rows, "قَهْوَة"), [AR], "typed harakat fold too");
        assert_eq!(found(&rows, "e"), [FR, EN], "newest first, as the rows");
        assert_eq!(found(&rows, "  cafe "), [FR], "the blanks around go");
        assert!(found(&rows, "tea").is_empty());
    }

    #[test]
    fn the_ticks_pick_each_ticked_searchs_first_row_in_order() {
        let g = crate::config::load().expect("loads").g20_product;
        assert_eq!(ticked(&g), [FR, AR]);
        let mut one = g.clone();
        one.searches
            .iter_mut()
            .for_each(|s| s.tick = s.lang == "ar");
        assert_eq!(ticked(&one), [AR]);
    }

    #[test]
    fn the_pasted_text_joins_two_ticks_with_the_breaks_and_the_rule() {
        let texts = [FR.to_string(), AR.to_string()];
        assert_eq!(pasted(&texts, &join(1, "")), format!("{FR}\n{AR}"));
        assert_eq!(pasted(&texts, &join(2, "")), format!("{FR}\n\n{AR}"));
        assert_eq!(pasted(&texts, &join(3, "")), format!("{FR}\n\n\n{AR}"));
        assert_eq!(
            pasted(&texts, &join(2, "---")),
            format!("{FR}\n\n---\n\n{AR}")
        );
        assert_eq!(pasted(&texts[..1], &join(2, "")), FR, "one text, no join");
    }

    #[test]
    fn the_saved_file_matches_with_windows_line_ends_and_nothing_more() {
        let want = format!("{FR}\n\n{AR}");
        assert!(saved_as(&format!("{FR}\r\n\r\n{AR}"), &want));
        assert!(saved_as(&want, &want));
        assert!(!saved_as(&format!("{FR}\r\n{AR}"), &want), "a break short");
        assert!(
            !saved_as(&format!("{FR}\r\n\r\n{AR}\r\n"), &want),
            "a break after"
        );
        assert!(!saved_as(&format!("{AR}\r\n\r\n{FR}"), &want), "tick order");
    }

    #[test]
    fn a_row_holding_a_hidden_text_is_a_leak() {
        let hidden = ["Hidden copy", "Cleared copy"];
        let rows = [AR.to_string(), "Hidden copy".to_string()];
        assert_eq!(leaks(&rows, &hidden), ["Hidden copy"]);
        assert!(leaks(&[AR.into(), FR.into()], &hidden).is_empty());
    }

    #[test]
    fn the_rows_part_logs_only_the_gates_own_texts() {
        let ours = [EN, FR, AR, "Hidden copy"];
        let want = shown(&[EN, FR, AR]);
        let part = rows_part(&want, &want, &["Hidden copy"], &ours);
        assert_eq!(part["ok"], true);
        assert_eq!(
            (part["rows"].as_u64(), part["unknown"].as_u64()),
            (Some(3), Some(0))
        );
        let mut got = want.clone();
        got.insert(0, "the owner's copy".to_string());
        got.push("Hidden copy".to_string());
        let part = rows_part(&got, &want, &["Hidden copy"], &ours);
        assert_eq!(part["ok"], false);
        assert_eq!(part["unknown"], 1);
        assert_eq!(part["leaks"], serde_json::json!(["Hidden copy"]));
        assert!(!part.to_string().contains("owner"), "no other text");
        let order = rows_part(&shown(&[AR, FR, EN]), &want, &[], &ours);
        assert_eq!(order["ok"], false, "newest first only");
    }
}
