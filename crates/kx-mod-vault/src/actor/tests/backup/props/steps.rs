//! The steps a property run takes: clicks, keys, jobs run or answered at random, waits, notices, failed saves and whole
//! walks through a flow.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    BoxError, FillButton, FillClick, PanelKey, PlaceError, PlaceState, RowButton,
};
use proptest::prelude::*;

use super::super::{KEY, NEW_KEY, OLD_KEY};
use crate::backup::{Answer, Look, Shut};

/// The highest row a click names, past any page.
const MAX_ROW: u8 = 6;
/// What gets typed: the passwords that open things, a short word and a bidi override.
const TYPED: [&str; 5] = [KEY, NEW_KEY, OLD_KEY, "x", "\u{202e}"];
/// The buttons the backup pages take, and two that leave them.
const BUTTONS: [FillButton; 12] = [
    FillButton::Backup,
    FillButton::BackUpNow,
    FillButton::AddPlace,
    FillButton::ChangePassword,
    FillButton::BringBack,
    FillButton::Import,
    FillButton::OtherFolder,
    FillButton::Here,
    FillButton::Done,
    FillButton::Back,
    FillButton::Close,
    FillButton::New,
];

/// A whole way through a flow that ends in a write, which random steps alone rarely finish.
#[derive(Copy, Clone, Debug)]
pub(super) enum Walk {
    /// Add place on the stick, with the question about a box from another computer when it asks.
    AddPlace,
    /// Change password.
    NewPassword,
    /// Bring back the first box found.
    BringBack,
    /// Import the `KeePass` file.
    Import,
}

/// One thing that happens while the backup pages may show.
#[derive(Clone, Debug)]
pub(super) enum Step {
    /// A whole way through a flow, from the Backup page.
    Walk(Walk),
    /// A click on the last view.
    Click(FillClick),
    /// A key from our keyboard.
    Key(PanelKey),
    /// The oldest job runs, as the job thread does.
    Run,
    /// The oldest job gets `answers()`'s answer at this place, wrapped, of its own round or a stale one.
    Answer {
        at: prop::sample::Index,
        stale: bool,
    },
    /// Every wait of the writes passes.
    Wait,
    /// The first memory check falls due.
    Days,
    /// The user answers a notice.
    Notice { check: bool, yes: bool },
    /// The Quick-fill key toggles the panel.
    Toggle,
    /// The next save fails.
    FailSave,
}

/// The answers a job may get in its place, which may not be the kind it asked for.
pub(super) fn answers() -> Vec<Answer> {
    let (failed, gone) = (BoxError::Failed, PlaceError::Missing);
    vec![
        Answer::Padlock(Err(failed)),
        Answer::Suggested(Vec::new()),
        Answer::Folders(Ok(Vec::new())),
        Answer::Folders(Err(gone)),
        Answer::Look(Look::Empty),
        Answer::Look(Look::Unread(gone)),
        Answer::Look(Look::Box {
            mark: None,
            modified_ms: None,
        }),
        Answer::Wrote(Err(BoxError::TooLarge)),
        Answer::Wrote(Err(failed)),
        Answer::Found(Vec::new()),
        Answer::State(PlaceState::Ready),
        Answer::State(PlaceState::Refused),
        Answer::Opened(Err(Shut::Read(gone))),
        Answer::Opened(Err(Shut::Open(BoxError::WrongPassword))),
        Answer::Checked(Ok(true)),
        Answer::Checked(Ok(false)),
        Answer::Checked(Err(BoxError::TimedOut)),
        Answer::Keepass(Err(Shut::Open(BoxError::Unsupported))),
    ]
}

/// Any click, most of them on the buttons and rows.
fn click() -> impl Strategy<Value = FillClick> {
    let button = prop::sample::select(BUTTONS.to_vec()).prop_map(FillClick::Button);
    let buttons = prop::sample::select(vec![RowButton::Open, RowButton::Remove]);
    let row = (0..=MAX_ROW, buttons).prop_map(|(row, button)| FillClick::Row { row, button });
    prop_oneof![
        5 => button,
        5 => row,
        2 => any::<bool>().prop_map(FillClick::Answer),
        1 => any::<bool>().prop_map(|back| FillClick::Page { back }),
        1 => any::<bool>().prop_map(FillClick::Checks),
        1 => any::<u32>().prop_map(|at| FillClick::Caret { at: at.into() }),
    ]
}

/// Any key, most of them typing.
fn key() -> impl Strategy<Value = PanelKey> {
    let typed = prop::sample::select(TYPED.to_vec());
    prop_oneof![
        3 => typed.prop_map(|t| PanelKey::Text(Redacted::new(t.to_owned()))),
        1 => Just(PanelKey::Word {
            erase: 1,
            text: Redacted::new(TYPED[0].to_owned()),
        }),
        1 => Just(PanelKey::Up),
        1 => Just(PanelKey::Down),
        1 => Just(PanelKey::Enter),
        1 => Just(PanelKey::Esc),
        1 => Just(PanelKey::Tick),
        1 => Just(PanelKey::Erase),
        1 => Just(PanelKey::Delete),
        1 => Just(PanelKey::Left),
        1 => Just(PanelKey::Right),
    ]
}

/// Any step, most of them clicks, keys and jobs run.
pub(super) fn step() -> impl Strategy<Value = Step> {
    let ways = vec![
        Walk::AddPlace,
        Walk::NewPassword,
        Walk::BringBack,
        Walk::Import,
    ];
    prop_oneof![
        4 => prop::sample::select(ways).prop_map(Step::Walk),
        8 => click().prop_map(Step::Click),
        4 => key().prop_map(Step::Key),
        8 => Just(Step::Run),
        2 => (any::<prop::sample::Index>(), any::<bool>())
            .prop_map(|(at, stale)| Step::Answer { at, stale }),
        1 => Just(Step::Wait),
        1 => Just(Step::Days),
        1 => (any::<bool>(), any::<bool>()).prop_map(|(check, yes)| Step::Notice { check, yes }),
        1 => Just(Step::Toggle),
        1 => Just(Step::FailSave),
    ]
}
