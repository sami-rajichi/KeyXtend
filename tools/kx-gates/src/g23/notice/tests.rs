//! Tests of `g23::notice`.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use super::*;

/// How the vault's source names the offer's key.
const OFFER_KEY: &str = "keys::VAULT_OFFER";
/// How a `Notice` literal in the vault's source starts its question.
const ASK: &str = "ask: Some(&NoticeAsk::";

fn words(w: &str) -> Piece {
    Piece::Words(w.to_string())
}

fn var(v: &str) -> Piece {
    Piece::Var(v.to_string())
}

#[test]
fn words_must_match_in_order_and_a_variable_takes_some_text() {
    let shape = [
        words("Started inside "),
        var("app"),
        words(", so nothing saves."),
    ];
    assert!(fits(&shape, "Started inside Claude, so nothing saves."));
    assert!(fits(&shape, "Started inside A, B, so nothing saves."));
    assert!(
        !fits(&shape, "Started inside , so nothing saves."),
        "an empty app"
    );
    assert!(!fits(
        &shape,
        "Started inside Claude, so nothing saves. More"
    ));
    assert!(!fits(&shape, "Begun inside Claude, so nothing saves."));
    let ends = [words("From "), var("app")];
    assert!(fits(&ends, "From Claude") && !fits(&ends, "From "));
    assert!(fits(&[words("Plain")], "Plain") && !fits(&[words("Plain")], "Plainer"));
}

/// The English banner with `app` in its variable's place.
fn banner(shapes: &[Vec<Piece>], app: &str) -> String {
    let fill = |p: &Piece| match p {
        Piece::Words(w) => w.clone(),
        Piece::Var(_) => app.to_string(),
    };
    shapes[0].iter().map(fill).collect()
}

/// The hidden-run banner's shapes in every shipped language.
fn shapes() -> Vec<Vec<Piece>> {
    ftl::shapes(keys::HIDDEN_RUN).expect("kx-ui has it")
}

#[test]
fn the_hidden_run_banner_in_any_language_ends_the_run_with_its_error() {
    let shapes = shapes();
    let shown = vec!["Esc".to_string(), banner(&shapes, "Claude")];
    assert_eq!(normal(&shown, &shapes), Err(HIDDEN_RUN.to_string()));
    for (i, shape) in shapes.iter().enumerate() {
        let one = banner(std::slice::from_ref(shape), "Claude");
        assert!(normal(&[one], &shapes).is_err(), "language {i}");
    }
    let plain = vec!["Esc".to_string(), "Quick-fill".to_string()];
    assert_eq!(normal(&plain, &shapes), Ok(()));
    assert_eq!(normal(&[], &shapes), Ok(()));
}

/// Runs `wade` on a banner that shows `queue`'s front beside a key, each answer taking it off: the result and the
/// notices left.
fn waded(queue: &[String], most: usize) -> (Result<(), String>, usize) {
    let q = RefCell::new(queue.iter().cloned().collect::<VecDeque<_>>());
    let names = || {
        let front = q.borrow().front().cloned();
        Ok(front.into_iter().chain(["Esc".to_string()]).collect())
    };
    let next = || Ok(q.borrow_mut().pop_front().is_some());
    let got = wade(&shapes(), most, names, next);
    (got, q.borrow().len())
}

#[test]
fn a_hidden_run_banner_queued_behind_another_notice_still_ends_the_run() {
    let queue = ["Words missing".to_string(), banner(&shapes(), "Claude")];
    assert_eq!(waded(&queue, 5), (Err(HIDDEN_RUN.to_string()), 1));
}

#[test]
fn other_notices_are_answered_until_the_banner_shows_none() {
    let queue = ["Words missing".to_string(), "Settings reset".to_string()];
    assert_eq!(waded(&queue, 2), (Ok(()), 0));
    assert_eq!(waded(&[], 2), (Ok(()), 0));
}

#[test]
fn more_notices_than_keys_or_a_failed_read_or_answer_end_the_run() {
    let queue = ["A".to_string(), "B".to_string(), "C".to_string()];
    let (got, _) = waded(&queue, 2);
    assert!(got.expect_err("too many").contains("notices"));
    let unread = wade(&shapes(), 2, || Err("no read".into()), || Ok(false));
    assert_eq!(unread, Err("no read".to_string()));
    let unanswered = wade(
        &shapes(),
        2,
        || Ok(vec!["A".into()]),
        || Err("no click".into()),
    );
    assert_eq!(unanswered, Err("no click".to_string()));
}

#[test]
fn an_answered_banner_moved_on_once_it_hides_or_reads_otherwise_and_a_failed_read_is_an_error() {
    let before = vec!["Words missing".to_string()];
    let same = || Ok(Some(before.clone()));
    assert_eq!(moved_on(false, same, &before), Ok(true), "it hid");
    assert_eq!(
        moved_on(true, same, &before),
        Ok(false),
        "it still reads so"
    );
    let next = || Ok(Some(vec!["Settings reset".to_string()]));
    assert_eq!(moved_on(true, next, &before), Ok(true), "the next notice");
    assert_eq!(moved_on(true, || Ok(None), &before), Ok(true), "it went");
    let failed = || Err("no read".to_string());
    assert_eq!(moved_on(true, failed, &before), Err("no read".to_string()));
}

#[test]
fn the_banner_is_known_by_the_title_kx_ui_gives_its_window() {
    let qml = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/kx-ui/qml/Notice.qml");
    let qml = std::fs::read_to_string(qml).expect("kx-ui has the banner");
    let title = crate::config::load().expect("loads").g23.notice_title;
    assert!(
        qml.contains(&format!("title: res.tr(\"{title}\")")),
        "{title}"
    );
}

#[test]
fn a_click_on_the_banner_waits_past_the_double_click_time() {
    assert!(past_double(500, 50) > 500);
    assert!(past_double(u32::MAX, u64::MAX) > u64::from(u32::MAX));
}

#[test]
fn the_banner_is_only_the_window_with_its_title() {
    let notice = "KeyXtend notice".to_string();
    let shown = [
        (1, "KeyXtend".to_string()),
        (2, notice.clone()),
        (3, "Quick-fill".to_string()),
    ];
    assert_eq!(banner_of(&shown, std::slice::from_ref(&notice)), Some(2));
    assert_eq!(banner_of(&shown[..1], std::slice::from_ref(&notice)), None);
}

/// The texts of the vault's source files, without its tests.
fn vault_source() -> Vec<String> {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/kx-mod-vault/src");
    let (mut files, mut dirs) = (Vec::new(), vec![root]);
    while let Some(dir) = dirs.pop() {
        for e in std::fs::read_dir(&dir)
            .expect("the vault's source")
            .flatten()
        {
            let (path, name) = (e.path(), e.file_name());
            if name == "tests" || name == "tests.rs" {
                continue;
            }
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                files.push(std::fs::read_to_string(&path).expect("reads"));
            }
        }
    }
    files
}

/// The `NoticeAsk` names in `text` that ask in a `Notice` literal naming `key`.
fn asks_for(text: &str, key: &str) -> Vec<String> {
    let named = |(at, _): (usize, &str)| {
        let rest = &text[at + ASK.len()..];
        let name = rest
            .chars()
            .take_while(|c| c.is_ascii_uppercase() || *c == '_');
        let literal = &rest[..rest.find('}')?];
        literal.contains(key).then(|| name.collect())
    };
    text.match_indices(ASK).filter_map(named).collect()
}

#[test]
fn the_offer_is_answered_from_the_buttons_the_vault_raises_it_with() {
    let found: Vec<String> = vault_source()
        .iter()
        .flat_map(|text| asks_for(text, OFFER_KEY))
        .collect();
    assert_eq!(found.len(), 1, "one place raises the offer: {found:?}");
    let known = [
        ("YES_NO", NoticeAsk::YES_NO),
        ("CHECK_LATER", NoticeAsk::CHECK_LATER),
        ("YES_LATER", NoticeAsk::YES_LATER),
    ];
    let raised = known.iter().find(|(n, _)| *n == found[0]).map(|(_, a)| *a);
    assert_eq!(raised, Some(OFFER_ASK), "the vault asks with {}", found[0]);
}
