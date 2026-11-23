//! Compare's pages of cloud picks, which the fit test measures: its set text with cloud models, the offered models
//! longest by name plus mark beside one or two typed Other ids, a short last page, a last page at the pairing edge, and
//! a page at the fewest rows.

use super::cloud::named;
use super::long::{per, services};
use super::{Shown, paged, shown};
use kx_mod_voice::services::Service;
use kx_module_api::ports::{
    CompareCloud, CompareMic, CompareSheet, DeviceLabel, DeviceName, ServiceName, TAG_LEN,
    VoiceSheet,
};
use kx_module_api::{Redacted, Settings};
use kx_ui::UiSettings;
use std::cmp::Reverse;

/// The fewest rows a page may show, as the `[ui]` settings allow them.
pub fn fewest() -> usize {
    let mut s = UiSettings::from_defaults().expect("the [ui] defaults");
    let default = s.page_rows;
    let fits = |n: &usize| {
        s.page_rows = *n;
        s.check().is_ok()
    };
    (0..=default).find(fits).expect("the default fits")
}

/// Row `id` of the services table.
fn row(id: &str) -> &'static Service {
    services().get(id).expect("a row of the table")
}

/// Compare's cloud model `name` of `service`, ticked.
fn pick(name: &str, service: ServiceName) -> CompareCloud {
    CompareCloud {
        name: Redacted::new(name.to_owned()),
        service,
        tick: true,
    }
}

/// A pick's width in characters, as the review counts it: its name and its service's mark.
fn width(c: &CompareCloud) -> usize {
    let mark = match &c.service {
        ServiceName::Named(n) => n.chars().count(),
        ServiceName::Custom => 0,
    };
    c.name.expose().chars().count() + mark
}

/// Each offered row's listed models as picks, in the table's order.
fn offered() -> impl Iterator<Item = CompareCloud> {
    let rows = services().rows().iter().filter(|r| r.offered);
    rows.flat_map(|r| r.models.iter().map(|m| pick(&m.name, r.shown())))
}

/// The offered picks, widest first; ties keep the table's order.
fn widest() -> Vec<CompareCloud> {
    let mut all: Vec<CompareCloud> = offered().collect();
    all.sort_by_key(|c| Reverse(width(c)));
    all
}

/// Row `id`'s typed Other id at its most, `c` over and over.
fn typed(c: char, id: &str) -> CompareCloud {
    pick(&c.to_string().repeat(*TAG_LEN.end()), row(id).shown())
}

/// Compare, ready, with `cloud` on its page and `send` in its warning.
fn ready(cloud: Vec<CompareCloud>, send: Vec<ServiceName>) -> VoiceSheet {
    VoiceSheet::Compare(CompareSheet::Ready {
        models: 2,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        mic: CompareMic::Closed,
        cloud,
        send,
    })
}

/// Every offered service, as the send warning names them with each ticked.
fn every() -> Vec<ServiceName> {
    let rows = services().rows().iter().filter(|r| r.offered);
    rows.map(Service::shown).collect()
}

/// Compare's set text with every cloud model of Groq and Custom ticked.
fn compare_long() -> VoiceSheet {
    let cloud = vec![
        pick("Whisper Large v3 Turbo", named("Groq")),
        pick("Whisper Large v3", named("Groq")),
        pick("whisper-large-v3-turbo", ServiceName::Custom),
    ];
    ready(cloud, vec![named("Groq"), ServiceName::Custom])
}

/// Compare's widest page (T24 re-review 1): the offered picks longest by name plus mark, then `OpenRouter`'s typed id.
fn compare_wide() -> VoiceSheet {
    let mut cloud: Vec<CompareCloud> = widest().into_iter().take(per() - 1).collect();
    cloud.push(typed('m', "openrouter"));
    ready(cloud, every())
}

/// Custom's typed id first and `OpenRouter`'s last, as the Cloud page's order puts them, with the widest offered picks
/// between them.
fn compare_typed() -> VoiceSheet {
    let mut cloud = vec![typed('m', "custom")];
    cloud.extend(widest().into_iter().take(per() - 2));
    cloud.push(typed('w', "openrouter"));
    ready(cloud, every())
}

/// Compare's short last page (T24 re-review 2): `OpenRouter`'s longest model id typed as its Other id, then Gemini's
/// pick, which fit whole.
fn compare_short() -> VoiceSheet {
    let router = row("openrouter");
    let ids = router.models.iter().map(|m| &m.id);
    let id = ids.max_by_key(|i| i.len()).expect("listed models");
    let gemini = row("gemini");
    let name = &gemini.models.first().expect("a listed model").name;
    let cloud = vec![pick(id, router.shown()), pick(name, gemini.shown())];
    ready(cloud, every())
}

/// Compare's last page at the pairing edge (T24 re-review 3): `OpenRouter`'s Whisper Large v3 Turbo and Voxtral picks,
/// then a typed id, in its order; each is too wide to share a row with the next, so they take a full page's rows
/// paired. MAI in Whisper's place would share Voxtral's row in Native.
fn compare_edge() -> VoiceSheet {
    let router = row("openrouter");
    let listed = |name: &str| {
        let m = router.models.iter().find(|m| m.name == name);
        pick(&m.expect("a listed model").name, router.shown())
    };
    let cloud = vec![
        listed("Whisper Large v3 Turbo"),
        listed("Voxtral Mini Transcribe 3"),
        pick(EDGE_ID, router.shown()),
    ];
    ready(cloud, every())
}

/// A typed Other id for `OpenRouter`, shorter than its longest listed id and well short of `TAG_LEN`.
const EDGE_ID: &str = "mistralai/voxtral-small-24b-2507";

/// Compare's page at the fewest rows a page may show: that many of the widest offered picks.
fn compare_few() -> VoiceSheet {
    ready(widest().into_iter().take(fewest()).collect(), every())
}

/// Compare's longest panels, by name; the wide pages show the pager, and the short and edge ones are each the last of
/// two.
pub fn panels() -> Vec<(String, Shown)> {
    let two = |sheet| paged(shown(sheet, &[]), 0, 2, 0);
    [
        ("compareLong", shown(compare_long(), &[])),
        ("compareWide", two(compare_wide())),
        ("compareTyped", two(compare_typed())),
        ("compareShort", paged(shown(compare_short(), &[]), 1, 2, 0)),
        ("compareEdge", paged(shown(compare_edge(), &[]), 1, 2, 0)),
        ("compareFew", two(compare_few())),
    ]
    .into_iter()
    .map(|(name, s)| (name.to_owned(), s))
    .collect()
}

/// The picks on Compare's page `sheet`.
fn picks(sheet: VoiceSheet) -> Vec<CompareCloud> {
    let VoiceSheet::Compare(CompareSheet::Ready { cloud, .. }) = sheet else {
        panic!("Compare, ready")
    };
    cloud
}

/// The wide page is full, no offered pick left off it is wider than one on it, and its typed id is at its most.
#[test]
fn the_wide_compare_page_holds_the_widest_offered_picks_and_a_typed_id() {
    let cloud = picks(compare_wide());
    assert_eq!(cloud.len(), per(), "a full page");
    let (on, typed) = cloud.split_at(per() - 1);
    let least = on.iter().map(width).min().expect("offered picks");
    let wider: Vec<CompareCloud> = offered()
        .filter(|c| !on.contains(c) && width(c) > least)
        .collect();
    assert_eq!(wider, [], "no wider pick is left off");
    assert_eq!(typed[0].name.expose().len(), *TAG_LEN.end());
}

/// The typed page holds Custom's and `OpenRouter`'s typed ids at their most round the widest offered picks, and both
/// rows take an Other id.
#[test]
fn the_typed_compare_page_holds_two_typed_ids_round_the_widest_picks() {
    let cloud = picks(compare_typed());
    assert_eq!(cloud.len(), per(), "a full page");
    let rows = ["custom", "openrouter"].map(row);
    assert!(
        rows.iter().all(|r| r.offered && r.other),
        "both take a typed id"
    );
    let ends = [&cloud[0], &cloud[per() - 1]].map(|c| (c.service.clone(), c.name.expose().len()));
    assert_eq!(ends, rows.map(|r| (r.shown(), *TAG_LEN.end())));
    assert_eq!(cloud[1..per() - 1], widest()[..per() - 2]);
}

/// The edge page holds three picks of `OpenRouter`, a row that takes a typed id: two listed models, then `EDGE_ID`.
#[test]
fn the_edge_compare_page_holds_three_picks_the_last_typed() {
    let cloud = picks(compare_edge());
    let router = row("openrouter");
    assert!(router.offered && router.other, "it takes a typed id");
    assert!(cloud.iter().all(|c| c.service == router.shown()));
    let names: Vec<&str> = cloud.iter().map(|c| c.name.expose().as_str()).collect();
    let listed = |n: &str| router.models.iter().any(|m| m.name == n);
    assert_eq!(names.len(), 3, "two listed picks and a typed id");
    assert!(listed(names[0]) && listed(names[1]), "two listed first");
    assert_eq!(names[2], EDGE_ID, "then the typed id");
    let ids = router.models.iter().map(|m| m.id.len());
    let longest = ids.max().expect("listed models");
    let typed = router.models.iter().all(|m| m.id != EDGE_ID);
    assert!(
        typed && EDGE_ID.len() < longest && EDGE_ID.len() < *TAG_LEN.end(),
        "typed, shorter than the longest listed id"
    );
}

/// The short page holds fewer picks than a page, the first a typed id of a row that takes one.
#[test]
fn the_short_compare_page_holds_a_typed_id_and_one_pick() {
    let cloud = picks(compare_short());
    assert!(cloud.len() < per(), "a short page");
    let router = row("openrouter");
    assert!(router.offered && router.other, "it takes a typed id");
    assert_eq!(cloud[0].service, router.shown());
}
