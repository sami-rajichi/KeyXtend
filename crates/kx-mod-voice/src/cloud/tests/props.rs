//! Property (ADR-0047 decision 12): no sequence of settings changes, picks or table loads moves a language from a local
//! model to a service by itself; only the user's own pick of that service for that language does, that service's
//! Confirm while the pick waits for it Off (owner, 2026-11-21), or Cloud On back to the pick it kept (ADR-0054). Each
//! case starts with languages in the cloud, so the chains that hide a pick and could bring it back are run often.

use std::cell::Cell;

use super::*;
use crate::services::Services;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{Config, TestCaseError, TestRunner};

/// The keyboard languages the steps touch; `tlh` is one no Groq model knows.
const LANGS: [&str; 4] = ["en", "fr", "ar", "tlh"];
/// The services a step names: `openrouter` has the keep-nothing switch, `openai` is offered only in a later table,
/// and `ovhcloud` is in no table here.
const SERVICES: [&str; 5] = ["groq", "custom", "openrouter", "openai", "ovhcloud"];
/// The models a step names, read from the table: Groq's first two, two typed ids, the `OpenRouter` model the switch
/// rules out, and `OpenAI`'s first.
fn cloud_models() -> [&'static str; 6] {
    let (groq, openai) = (&row("groq").models, &row("openai").models);
    let (two, own) = ((&groq[0].id, &groq[1].id), &openai[0].id);
    [
        two.0,
        two.1,
        "my-model",
        "other-model",
        router_model(true),
        own,
    ]
}
/// Local rows a step picks.
const LOCAL: [&str; 2] = ["whisper-turbo-q5km", "canary-v2-q8"];
/// Addresses a step types: none, one Custom takes, and one no row takes.
const ADDRESSES: [&str; 3] = ["", ADDRESS, "http://stt.example.com/v1"];
/// How many tables a load may bring.
const TABLES: usize = 6;
/// The cases run.
const CASES: u32 = 512;
/// The least share of steps, in percent, that must start with a language in the cloud; runs reach about 62.
const HELD_PCT: u32 = 45;

/// One thing the user, the file or an update does.
#[derive(Clone, Debug)]
enum Step {
    /// The `cloud` switch on the Cloud page.
    Switch(bool),
    /// A card confirmed.
    Confirm(usize),
    /// A service switched Off, which leaves its listed picks waiting.
    TurnOff(usize),
    /// A service's newer terms declined, which leaves its listed picks waiting (ADR-0054).
    Decline(usize),
    /// The user picks a service's model for a language.
    Pick(usize, usize, usize),
    /// The user picks a local row for a language.
    Local(usize, usize),
    /// A language's pick is cleared.
    Unpick(usize),
    /// An Other model id typed.
    Type(usize, usize),
    /// An address typed or cleared.
    Address(usize, usize),
    /// A row's keep-nothing switch, which rules a kept model out but never hides its pick.
    Keep(usize, bool),
    /// Another version's table loads, and `[voice]` is read again.
    Load(usize),
    /// `cloud` written in the file, then `[voice]` read again.
    Raw(bool),
}

impl Step {
    /// True for a step that could hide a cloud pick: a read, a switch off, or a typed line changed.
    fn hides(&self) -> bool {
        matches!(
            self,
            Self::Switch(false)
                | Self::TurnOff(_)
                | Self::Decline(_)
                | Self::Type(..)
                | Self::Address(..)
                | Self::Load(_)
                | Self::Raw(_)
        )
    }
}

fn step() -> impl Strategy<Value = Step> {
    let (l, s, m) = (0..LANGS.len(), 0..SERVICES.len(), 0..cloud_models().len());
    prop_oneof![
        1 => mostly_on().prop_map(Step::Switch),
        3 => s.clone().prop_map(Step::Confirm),
        1 => s.clone().prop_map(Step::TurnOff),
        1 => s.clone().prop_map(Step::Decline),
        6 => (l.clone(), s.clone(), m.clone()).prop_map(|(l, s, m)| Step::Pick(l, s, m)),
        1 => (l.clone(), 0..LOCAL.len()).prop_map(|(l, r)| Step::Local(l, r)),
        1 => l.prop_map(Step::Unpick),
        2 => (s.clone(), m).prop_map(|(s, m)| Step::Type(s, m)),
        2 => (s.clone(), 0..ADDRESSES.len()).prop_map(|(s, a)| Step::Address(s, a)),
        2 => (s, any::<bool>()).prop_map(|(s, on)| Step::Keep(s, on)),
        3 => (0..TABLES).prop_map(Step::Load),
        2 => mostly_on().prop_map(Step::Raw),
    ]
}

/// A `cloud` value, on three times in four, so a case spends most steps where a pick can be held.
fn mostly_on() -> impl Strategy<Value = bool> {
    prop_oneof![1 => Just(false), 3 => Just(true)]
}

/// The shipped table, then Groq's terms raised, Groq not offered, Groq's turbo gone, Groq with no Other line, and
/// `OpenAI`, the first row not offered, offered as a later version may, so its entries and picks meet a load hiding it.
/// Each edit finds its line by key or from the table, so a missing one fails rather than leaves the table as it is.
fn tables() -> Vec<Services> {
    let gone = (
        format!("id = \"{}\"", turbo()),
        format!("id = \"{}-2\"", turbo()),
    );
    let texts = [
        TEXT.to_owned(),
        raised(TEXT, "groq"),
        row_edit(TEXT, "groq", &[("offered", "offered = false")]),
        edits_text(TEXT, &[(&gone.0, &gone.1)]),
        row_edit(TEXT, "groq", &[("other", "other = false")]),
        row_edit(TEXT, "openai", &[("offered", "offered = true")]),
    ];
    let tables: Vec<Services> = texts.iter().map(|text| parse(text).unwrap()).collect();
    assert_eq!(tables.len(), TABLES);
    assert!(
        tables[1..].iter().all(|t| *t != tables[0]),
        "each edit changes the table"
    );
    let offers = |t: &Services| t.get("openai").is_some_and(|r| r.offered);
    assert!(!offers(&tables[0]) && offers(&tables[TABLES - 1]));
    tables
}

/// The start: cloud on, Groq and Custom confirmed, Custom's address and model typed, and three languages in the cloud.
fn seeded(table: &Services) -> VoiceSettings {
    let rules = Cloud::new(table, false);
    let mut s = VoiceSettings::from_defaults().unwrap();
    rules.switch(&mut s, true);
    assert!(rules.confirm(&mut s, "groq", today()) && rules.confirm(&mut s, "custom", today()));
    assert!(rules.type_address(&mut s, "custom", ADDRESS));
    assert!(rules.type_model(&mut s, "custom", "my-model"));
    let picks = [
        ("fr", "groq", turbo()),
        ("ar", "groq", cloud_models()[1]),
        ("en", "custom", "my-model"),
    ];
    for (lang, id, model) in picks {
        assert!(rules.pick(&mut s, lang, id, model), "{lang}");
    }
    s
}

/// The service each language dictates with, or `None` for its local model.
fn used(table: &Services, s: &VoiceSettings) -> [Option<String>; 4] {
    let rules = Cloud::new(table, false);
    LANGS.map(|lang| rules.choice(s, lang).map(|c| c.service.id.clone()))
}

/// The service Off each language's pick waits for, if any.
fn waiting(table: &Services, s: &VoiceSettings) -> [Option<String>; 4] {
    let rules = Cloud::new(table, false);
    LANGS.map(|lang| rules.waits(s, lang).map(|o| o.service.id.clone()))
}

/// The service each language's cloud pick names, whether it serves, waits or not.
fn named(s: &VoiceSettings) -> [Option<String>; 4] {
    LANGS.map(|lang| Some(cloud_pick(s.picks.get(lang)?)?.0.to_owned()))
}

/// Carries out `step` on `s` with table `at`, which a load changes. A refused act changes nothing, and the property
/// checks it all the same.
fn apply(step: &Step, tables: &[Services], at: &mut usize, s: &mut VoiceSettings) {
    let models = models::table().unwrap();
    let rules = Cloud::new(&tables[*at], false);
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
        Step::Decline(i) => {
            rules.decline(s, SERVICES[i]);
            true
        }
        Step::Pick(l, i, m) => rules.pick(s, LANGS[l], SERVICES[i], cloud_models()[m]),
        Step::Local(l, r) => s.picks.insert(LANGS[l].into(), LOCAL[r].into()).is_some(),
        Step::Unpick(l) => s.picks.remove(LANGS[l]).is_some(),
        Step::Type(i, m) => rules.type_model(s, SERVICES[i], cloud_models()[m]),
        Step::Address(i, a) => rules.type_address(s, SERVICES[i], ADDRESSES[a]),
        Step::Keep(i, on) => rules.keep(s, SERVICES[i], on),
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

/// The property after `step`: no language reached a service it did not pick or wait for, no cloud pick is kept that
/// Cloud On would neither serve nor show waiting, and a hidden run or `cloud` off dictates nowhere in the cloud.
fn check(
    step: &Step,
    table: &Services,
    s: &VoiceSettings,
    before: &[[Option<String>; 4]; 3],
) -> Result<(), TestCaseError> {
    let after = used(table, s);
    for (i, lang) in LANGS.iter().enumerate() {
        prop_assert!(
            !moved(step, i, before, &after),
            "{} moved to {:?} by {:?}",
            lang,
            after[i],
            step
        );
    }
    let (rules, on) = (Cloud::new(table, false), s.cloud_on());
    let hidden_pick = s.picks.iter().any(|(l, p)| {
        cloud_pick(p).is_some() && rules.choice(&on, l).is_none() && rules.waits(&on, l).is_none()
    });
    prop_assert!(!hidden_pick, "a cloud pick waits hidden: {:?}", s.picks);
    let hidden = Cloud::new(table, true);
    prop_assert!(
        LANGS.iter().all(|l| hidden.choice(s, l).is_none()),
        "a hidden run in the cloud"
    );
    prop_assert!(
        s.cloud || after.iter().all(Option::is_none),
        "cloud off in the cloud"
    );
    Ok(())
}

/// True when language `i` reached a service by `step` alone: neither its own pick of that service, that service's
/// Confirm while its pick waited for it, nor Cloud On back to the service it named, of `before`'s used, waiting and
/// named services.
fn moved(
    step: &Step,
    i: usize,
    before: &[[Option<String>; 4]; 3],
    after: &[Option<String>; 4],
) -> bool {
    let (now, waited) = (after[i].as_deref(), before[1][i].as_deref());
    let asked = matches!(step, Step::Pick(l, svc, _) if *l == i && now == Some(SERVICES[*svc]));
    let back = matches!(step, Step::Confirm(svc) if waited == Some(SERVICES[*svc]));
    let on = matches!(step, Step::Switch(true) | Step::Raw(true)) && now == before[2][i].as_deref();
    now.is_some() && after[i] != before[0][i] && !asked && !(back && now == waited) && !on
}

#[test]
fn no_step_moves_a_language_from_local_to_a_service_by_itself() {
    let tables = tables();
    let (held, all, chains) = (Cell::new(0_u32), Cell::new(0_u32), Cell::new(0_u32));
    let config = Config {
        cases: CASES,
        source_file: Some(file!()),
        ..Config::default()
    };
    let run = TestRunner::new(config).run(&vec(step(), 1..40), |steps| {
        let (mut at, mut hid, mut chained) = (0, false, false);
        let mut s = seeded(&tables[0]);
        for step in &steps {
            chained |= hid;
            let before = [used(&tables[at], &s), waiting(&tables[at], &s), named(&s)];
            let in_cloud = before[0].iter().any(Option::is_some);
            held.set(held.get() + u32::from(in_cloud));
            all.set(all.get() + 1);
            apply(step, &tables, &mut at, &mut s);
            check(step, &tables[at], &s, &before)?;
            hid |= in_cloud && step.hides();
        }
        chains.set(chains.get() + u32::from(chained));
        Ok(())
    });
    let (held, all, chains) = (held.get(), all.get(), chains.get());
    // Shown with `--nocapture`, so a run tells how often the property reached the state it guards.
    eprintln!(
        "{held} of {all} steps began in the cloud; {chains} of {CASES} cases stepped on after hiding a pick"
    );
    run.unwrap();
    assert!(
        held * 100 >= all * HELD_PCT,
        "the property seldom reaches a cloud pick"
    );
}
