//! A row not offered (ADR-0050) stays in the table, built and tested, but the rules never show, confirm, pick or send
//! to it, even with the entry and key a hand edit or an older settings file left; its saved picks dictate locally.

use super::*;
use crate::services::tests::{NOT_OFFERED, OFFERED};

/// The owner's three keyboard languages.
const LANGS: [&str; 3] = ["en", "fr", "ar"];

/// Cloud on, Groq and `OpenRouter` confirmed, and an entry for each row not offered, as a hand edit leaves.
fn hand_edited() -> VoiceSettings {
    let mut s = on();
    for id in ["groq", "openrouter"] {
        assert!(rules().confirm(&mut s, id, today()), "{id}");
    }
    for id in NOT_OFFERED {
        s.services.insert(id.into(), ServiceOn::new(today(), 1));
    }
    s
}

#[test]
fn a_row_not_offered_never_shows_and_cannot_be_confirmed_picked_or_sent_to() {
    let mut s = hand_edited();
    let ids: Vec<&str> = rules().offered(&s).iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, OFFERED);
    let before = s.clone();
    for id in NOT_OFFERED {
        let row = row(id);
        assert_eq!(
            rules().mark(&s, row, true),
            ServiceMark::Off,
            "{id}: entry and key alike"
        );
        assert!(
            !rules().asks(&s, row) && !rules().confirm(&mut s, id, today()),
            "{id}"
        );
        for (model, lang) in row.models.iter().flat_map(|m| LANGS.map(|l| (m, l))) {
            assert!(!rules().pick(&mut s, lang, id, &model.id), "{id} {lang}");
            let sent = rules().model_for(&s, lang, id, &model.id);
            assert!(sent.is_none(), "{id} {lang}: Compare sends nothing");
        }
        assert!(
            row.other && !rules().type_model(&mut s, id, "my-model"),
            "{id}"
        );
    }
    assert_eq!(s, before, "nothing changed");
    for lang in LANGS {
        let listed = rules().models(&s, lang);
        assert!(!listed.is_empty(), "{lang}: the offered rows list theirs");
        let ids: Vec<&str> = listed.iter().map(|o| o.service.id.as_str()).collect();
        assert!(ids.iter().all(|id| OFFERED.contains(id)), "{lang}: {ids:?}");
    }
}

#[test]
fn a_saved_pick_of_a_row_not_offered_dictates_locally_and_moves_to_no_other_service() {
    let mut s = hand_edited();
    let models = models::table().unwrap();
    for id in NOT_OFFERED {
        for model in &row(id).models {
            for lang in LANGS.into_iter().filter(|l| model.code(l).is_some()) {
                s.picks.insert(lang.into(), pick_text(id, &model.id));
                assert!(rules().choice(&s, lang).is_none(), "{id} {lang}");
                let local = models.default_for(lang).map(Pick::Row);
                assert_eq!(models.used(lang, &s.picks), local, "{id} {lang}");
            }
        }
    }
}
