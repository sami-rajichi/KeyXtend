use super::*;
use crate::LANGS;
use kx_module_api::NoticeAsk;
use proptest::prelude::*;
use std::collections::BTreeSet;

const KEYBOARD: ModuleId = ModuleId::new("keyboard");

fn failed() -> Notice {
    Notice::about(keys::MODULE_FAILED, KEYBOARD)
}

/// A notice about no module that asks Yes or No.
fn asking() -> Notice {
    Notice {
        ask: Some(&NoticeAsk::YES_NO),
        ..Notice::plain(keys::KEYS_NOT_SENT)
    }
}

#[test]
fn notices_show_once_in_order_with_their_text() {
    let mut n = Notices::default();
    assert!(n.push(failed()));
    assert!(n.push(Notice::plain(keys::KEYS_NOT_SENT)));
    let shown = n.take(&I18n::new(&LANGS[0]));
    assert_eq!(shown.len(), 2);
    assert!(shown[0].text.contains("Keyboard") && shown[0].retry);
    assert!(!shown[1].retry);
    assert_ne!(shown[0].id, shown[1].id);
    assert!(n.take(&I18n::new(&LANGS[0])).is_empty(), "taken once");
}

#[test]
fn the_same_notice_waits_or_shows_only_once() {
    let mut n = Notices::default();
    assert!(n.push(failed()));
    assert!(!n.push(failed()), "already waiting");
    let id = n.take(&I18n::new(&LANGS[0]))[0].id;
    assert!(!n.push(failed()), "already shown");
    assert_eq!(n.close(id).as_ref().and_then(retry_of), Some(KEYBOARD));
    assert!(n.push(failed()), "closed, so it may show again");
}

#[test]
fn only_a_failed_module_offers_try_again() {
    let mut n = Notices::default();
    let mut settings = Notice::plain(keys::SETTINGS_RESET);
    settings.module = Some(KEYBOARD);
    n.push(settings);
    n.push(Notice::plain(keys::KEYS_NOT_SENT));
    let ids: Vec<u64> = n.take(&I18n::new(&LANGS[0])).iter().map(|s| s.id).collect();
    assert_eq!(
        n.close(ids[0]).as_ref().and_then(retry_of),
        None,
        "a settings notice"
    );
    assert_eq!(
        n.close(ids[1]).as_ref().and_then(retry_of),
        None,
        "a plain notice"
    );
    assert_eq!(n.close(ids[1]), None, "closed already");
    assert_eq!(n.close(u64::MAX), None, "never shown");
}

#[test]
fn texts_follow_the_language_when_taken() {
    let mut n = Notices::default();
    n.push(failed());
    let ar = crate::find("ar").unwrap();
    let shown = n.take(&I18n::new(ar));
    assert_eq!(shown[0].text, I18n::new(ar).notice(&failed()));
}

#[test]
fn an_asking_notice_shows_its_answers_in_the_keyboards_language() {
    let mut n = Notices::default();
    n.push(asking());
    n.push(Notice::plain(keys::KEYS_NOT_SENT));
    let answers = |yes: &str, no: &str| {
        Some(Answers {
            yes: yes.to_owned(),
            no: no.to_owned(),
        })
    };
    let shown = n.take(&I18n::new(&LANGS[0]));
    assert_eq!(shown[0].ask, answers("Yes", "No"));
    assert_eq!(shown[1].ask, None, "a plain notice asks nothing");
    let ar = I18n::new(crate::find("ar").unwrap());
    assert_eq!(n.shown(&ar)[0].ask, answers("نعم", "لا"), "told again");
}

#[test]
fn a_closed_notice_comes_back_once_with_its_ask() {
    let mut n = Notices::default();
    n.push(asking());
    let id = n.take(&I18n::new(&LANGS[0]))[0].id;
    assert_eq!(n.close(id), Some(asking()), "so the hub can answer it");
    assert_eq!(n.close(id), None, "closed already");
}

#[test]
fn the_shown_notices_are_told_again_in_a_new_language_until_closed() {
    let mut n = Notices::default();
    n.push(failed());
    n.push(Notice::plain(keys::KEYS_NOT_SENT));
    let en = n.take(&I18n::new(&LANGS[0]));
    n.push(Notice::plain(keys::SETTINGS_LOCKED));
    let fr = I18n::new(crate::find("fr").unwrap());
    let again = n.shown(&fr);
    let ids: Vec<u64> = again.iter().map(|s| s.id).collect();
    assert_eq!(ids, [en[0].id, en[1].id], "the shown ones, in order");
    assert_eq!(again[0].text, fr.notice(&failed()));
    assert_eq!((again[0].retry, again[1].retry), (true, false));
    n.close(en[0].id);
    assert_eq!(n.shown(&fr).len(), 1, "a closed one is gone");
}

/// One step on the notices: push sample `n`, take the waiting ones, or close the open one at a place.
#[derive(Clone, Debug)]
enum Op {
    Push(usize),
    Take,
    Close(usize),
}

/// The samples the steps push: failed modules, a module notice without Try again, plain notices and one that asks.
fn sample(n: usize) -> Notice {
    let samples = [
        asking(),
        failed(),
        Notice {
            module: Some(ModuleId::new("layouts")),
            ..failed()
        },
        Notice {
            module: Some(KEYBOARD),
            ..Notice::plain(keys::SETTINGS_RESET)
        },
        Notice::plain(keys::KEYS_NOT_SENT),
        Notice::plain(keys::SETTINGS_LOCKED),
    ];
    samples[n % samples.len()].clone()
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        any::<usize>().prop_map(Op::Push),
        Just(Op::Take),
        any::<usize>().prop_map(Op::Close)
    ]
}

proptest! {
    #[test]
    fn ids_are_unique_and_a_notice_waits_or_shows_once(ops in prop::collection::vec(op(), 0..60)) {
        let (i18n, mut n, mut ids) = (I18n::new(&LANGS[0]), Notices::default(), BTreeSet::new());
        for op in ops {
            match op {
                Op::Push(k) => {
                    let _ = n.push(sample(k));
                }
                Op::Take => {
                    for shown in n.take(&i18n) {
                        prop_assert!(ids.insert(shown.id), "id {} shown twice", shown.id);
                    }
                }
                Op::Close(at) => {
                    let id = n.open.get(at % n.open.len().max(1)).map_or(u64::MAX, |o| o.0);
                    let _ = n.close(id);
                }
            }
            let all: Vec<&Notice> = n.waiting.iter().chain(n.open.iter().map(|o| &o.1)).collect();
            for (i, a) in all.iter().enumerate() {
                prop_assert!(!all[i + 1..].contains(a), "{a:?} waits or shows twice");
            }
        }
    }
}
