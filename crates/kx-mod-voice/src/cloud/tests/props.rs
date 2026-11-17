//! Property (ADR-0047 decision 12): no sequence of settings changes, picks or table loads moves a language from a local
//! model to a service by itself; only the user's own pick of that service for that language does, and a hidden run or
//! `cloud` off never dictates in the cloud.

use super::*;
use crate::services::Services;
use proptest::collection::vec;
use proptest::prelude::*;

/// The keyboard languages the steps touch; `tlh` is one no Groq model knows.
const LANGS: [&str; 4] = ["en", "fr", "ar", "tlh"];
/// The services a step names; `openai` is in no table here.
const SERVICES: [&str; 3] = ["groq", "custom", "openai"];
/// The models a step names: Groq's two, and two typed ids.
const MODELS: [&str; 4] = [TURBO, "whisper-large-v3", "my-model", "other-model"];
/// Local rows a step picks.
const LOCAL: [&str; 2] = ["whisper-turbo-q5km", "canary-v2-q8"];

/// One thing the user, the file or an update does.
#[derive(Clone, Debug)]
enum Step {
    /// The `cloud` switch on the Cloud page.
    Switch(bool),
    /// A card confirmed.
    Confirm(usize),
    /// A service switched off, or its newer terms declined.
    TurnOff(usize),
    /// The user picks a service's model for a language.
    Pick(usize, usize, usize),
    /// The user picks a local row for a language.
    Local(usize, usize),
    /// A language's pick is cleared.
    Unpick(usize),
    /// An Other model id typed.
    Type(usize, usize),
    /// Another version's table loads, and `[voice]` is read again.
    Load(usize),
    /// `cloud` written in the file, then `[voice]` read again.
    Raw(bool),
}

fn step() -> impl Strategy<Value = Step> {
    let (l, s, m) = (0..LANGS.len(), 0..SERVICES.len(), 0..MODELS.len());
    prop_oneof![
        any::<bool>().prop_map(Step::Switch),
        s.clone().prop_map(Step::Confirm),
        s.clone().prop_map(Step::TurnOff),
        (l.clone(), s.clone(), m.clone()).prop_map(|(l, s, m)| Step::Pick(l, s, m)),
        (l.clone(), 0..LOCAL.len()).prop_map(|(l, r)| Step::Local(l, r)),
        l.prop_map(Step::Unpick),
        (s, m).prop_map(|(s, m)| Step::Type(s, m)),
        (0..TABLES).prop_map(Step::Load),
        any::<bool>().prop_map(Step::Raw),
    ]
}

/// How many tables a load may bring.
const TABLES: usize = 5;

/// The shipped table, then Groq's terms raised, Groq not offered, Groq's turbo gone, and Groq with no Other line.
fn tables() -> Vec<Services> {
    let edits = [
        ("", ""),
        ("terms = 1", "terms = 2"),
        ("offered = true", "offered = false"),
        (
            "id = \"whisper-large-v3-turbo\"",
            "id = \"whisper-large-v3-turbo-2\"",
        ),
        ("other = true", "other = false"),
    ];
    let tables: Vec<Services> = edits
        .iter()
        .map(|(from, to)| Services::parse(&TEXT.replacen(from, to, 1)).unwrap())
        .collect();
    assert_eq!(tables.len(), TABLES);
    tables
}

/// The service each language dictates with, or `None` for its local model.
fn used(table: &Services, s: &VoiceSettings) -> [Option<String>; 4] {
    let rules = Cloud::new(table, false);
    LANGS.map(|lang| rules.choice(s, lang).map(|c| c.service.id.clone()))
}

/// Carries out `step` on `s` with table `at`, which a load changes.
fn apply(step: &Step, tables: &[Services], at: &mut usize, s: &mut VoiceSettings) {
    let models = models::table().unwrap();
    let rules = Cloud::new(&tables[*at], false);
    // Each act says whether it took; a refused one changes nothing, which the property checks all the same.
    let _ = match *step {
        Step::Switch(on) => {
            rules.switch(s, on);
            true
        }
        Step::Confirm(i) => rules.confirm(s, SERVICES[i], today()),
        Step::TurnOff(i) => {
            rules.turn_off(s, SERVICES[i]);
            true
        }
        Step::Pick(l, i, m) => rules.pick(s, LANGS[l], SERVICES[i], MODELS[m]),
        Step::Local(l, r) => s.picks.insert(LANGS[l].into(), LOCAL[r].into()).is_some(),
        Step::Unpick(l) => s.picks.remove(LANGS[l]).is_some(),
        Step::Type(i, m) => rules.type_model(s, SERVICES[i], MODELS[m]),
        Step::Load(t) => {
            *at = t;
            s.drop_bad_with(models, &tables[t]).is_empty()
        }
        Step::Raw(on) => {
            if let Some(next) = s.with("cloud", on.into()) {
                *s = next;
            }
            s.drop_bad_with(models, &tables[*at]).is_empty()
        }
    };
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn no_step_moves_a_language_from_local_to_a_service_by_itself(steps in vec(step(), 1..40)) {
        let tables = tables();
        let mut at = 0;
        let mut s = VoiceSettings::from_defaults().unwrap();
        for step in &steps {
            let before = used(&tables[at], &s);
            apply(step, &tables, &mut at, &mut s);
            let after = used(&tables[at], &s);
            for (i, lang) in LANGS.iter().enumerate() {
                let moved = before[i].is_none() && after[i].is_some();
                let asked = matches!(step, Step::Pick(l, svc, _) if *l == i && after[i].as_deref() == Some(SERVICES[*svc]));
                prop_assert!(!moved || asked, "{} moved to {:?} by {:?}", lang, after[i], step);
            }
            let hidden = Cloud::new(&tables[at], true);
            prop_assert!(LANGS.iter().all(|l| hidden.choice(&s, l).is_none()), "a hidden run in the cloud");
            prop_assert!(s.cloud || after.iter().all(Option::is_none), "cloud off in the cloud");
        }
    }
}
