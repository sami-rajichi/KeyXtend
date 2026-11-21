//! Any run of clicks on the cloud pages, late ones and older answers included: a dimmed cloud button never acts, an
//! answer to an older Paste or test never changes the key page, and the shown page stays within its list.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    ClipText, CloudButton, CloudFault, CloudLine, ModelSide, PanelButton, PanelKey, VoiceClick,
    VoicePanel, VoiceSheet, VoiceTab,
};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

use super::super::super::rig::Rig;
use super::super::super::{Do, Panel};
use super::{apply, on_cloud, put};
use crate::services::tests::shipped::{entry, first};

/// The highest row place a click names, past a page's rows.
const MAX_ROW: u8 = 7;
/// The most rounds an answer lags.
const MAX_LAG: u32 = 3;
/// The most steps in one case.
const MAX_STEPS: usize = 40;
/// The most characters a caret click lands past.
const MAX_AT: u32 = 8;

/// One thing the cloud pages meet.
#[derive(Clone, Debug)]
enum Step {
    /// A click on the view drawn now.
    Click(VoiceClick),
    /// A key typed on the key page's line.
    Key(PanelKey),
    /// The answer to the Paste this many rounds back.
    Pasted(u32),
    /// The answer to the test this many rounds back, a pass or a refusal.
    Tested(u32, bool),
}

fn click() -> impl Strategy<Value = VoiceClick> {
    let lines = || select(CloudLine::ALL.to_vec());
    let caret = |(line, at)| VoiceClick::Caret {
        line,
        at: Redacted::new(at),
    };
    prop_oneof![
        2 => select(vec![VoiceTab::Cloud, VoiceTab::Models]).prop_map(VoiceClick::Open),
        1 => any::<bool>().prop_map(VoiceClick::Cloud),
        6 => select(CloudButton::ALL.to_vec()).prop_map(VoiceClick::Service),
        3 => (0..MAX_ROW).prop_map(VoiceClick::Row),
        1 => any::<bool>().prop_map(|back| VoiceClick::Page { back }),
        2 => Just(VoiceClick::Button(PanelButton::Back)),
        1 => select(ModelSide::ALL.to_vec()).prop_map(VoiceClick::Side),
        1 => any::<bool>().prop_map(VoiceClick::KeepNothing),
        2 => lines().prop_map(VoiceClick::Save),
        1 => (lines(), 0..MAX_AT).prop_map(caret),
        2 => any::<bool>().prop_map(VoiceClick::Confirm),
    ]
}

fn key() -> impl Strategy<Value = PanelKey> {
    let text = PanelKey::Text(Redacted::new("ab".to_owned()));
    select(vec![
        PanelKey::Up,
        PanelKey::Down,
        PanelKey::Enter,
        PanelKey::Erase,
        text,
    ])
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        8 => click().prop_map(Step::Click),
        2 => key().prop_map(Step::Key),
        1 => (1..=MAX_LAG).prop_map(Step::Pasted),
        1 => (1..=MAX_LAG, any::<bool>()).prop_map(|(lag, ok)| Step::Tested(lag, ok)),
    ]
}

/// The cloud buttons `view` draws live.
fn live(view: &VoicePanel) -> Vec<CloudButton> {
    match &view.sheet {
        VoiceSheet::Cloud(s) => s.acts.clone(),
        VoiceSheet::Service(s) => s.acts.clone(),
        VoiceSheet::Card(c) => c.site.iter().map(|_| CloudButton::Link).collect(),
        _ => Vec::new(),
    }
}

/// What a click or an answer that does nothing leaves as it was.
fn shot(p: &Panel) -> String {
    let c = &p.cloud;
    let parts = (
        p.page,
        p.at,
        &p.ticks,
        &c.shown,
        c.check,
        c.renews_at,
        c.unkept,
        c.dots,
    );
    format!("{parts:?} {:?} {:?}", c.take, c.test)
}

/// Runs `step`; a dimmed cloud button or an older answer leaves the panel as it was.
fn run(rig: &mut Rig, step: &Step) -> Result<Vec<Do>, TestCaseError> {
    let Some(view) = rig.view() else {
        return Ok(rig.panel.open());
    };
    let before = shot(&rig.panel);
    let older = |rig: &Rig, lag: u32| rig.panel.cloud.round.wrapping_sub(lag);
    let (out, still) = match step {
        Step::Click(click) => {
            let dimmed = matches!(click, VoiceClick::Service(b) if !live(&view).contains(b));
            (rig.click(click), dimmed)
        }
        Step::Key(key) => (rig.with(|p, f| p.line_key(key, f)), false),
        Step::Pasted(lag) => {
            let round = older(rig, *lag);
            (rig.with(|p, _| p.pasted(round, ClipText::Busy)), true)
        }
        Step::Tested(lag, ok) => {
            let (ticket, answer) = (older(rig, *lag), ok.then_some(()).ok_or(CloudFault::Key));
            rig.with(|p, f| p.tested(ticket, answer, f.now));
            (Vec::new(), true)
        }
    };
    if still {
        prop_assert!(out.is_empty(), "{step:?} acted: {out:?}");
        prop_assert_eq!(before, shot(&rig.panel), "{:?} changed the panel", step);
    }
    Ok(out)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_dimmed_cloud_button_or_an_older_answer_never_acts_and_the_pages_hold(
        steps in vec(step(), 1..MAX_STEPS),
    ) {
        let mut rig = on_cloud("");
        put(&mut rig, "services", &entry(first(), ""));
        rig.keyed.insert(first().id.clone());
        for step in &steps {
            let out = run(&mut rig, step)?;
            apply(&mut rig, &out);
            if let Some(view) = rig.view() {
                prop_assert!(view.pages.at < view.pages.count, "{:?}", view.pages);
            }
        }
    }
}
