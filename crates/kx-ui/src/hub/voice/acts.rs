//! The kinds of click QML names to `Hub::voice_act`, and what each one sends. A button, page or download button goes by
//! the id the view gives it; a row's place, a page's way or an answer is the argument.

use crate::i18n::keys::{
    bar_button_id, cloud_button_id, fetch_button_id, model_side_id, panel_button_id, voice_tab_id,
};
use kx_module_api::ports::{
    BarButton, CloudButton, FetchButton, ModelSide, PanelButton, VoiceClick, VoiceTab,
};
use std::cmp::Ordering;

/// The bar button `id`.
pub const BAR: &str = "bar";
/// The gear: open the Voice panel.
pub const GEAR: &str = "gear";
/// The pointer came over the bar with 1, or left it with 0.
pub const OVER: &str = "over";
/// A home row: open the page `id`.
pub const OPEN: &str = "open";
/// The row at the argument's place on the page.
pub const ROW: &str = "row";
/// Turn a page: back when the argument is below 0, on when above.
pub const PAGE: &str = "page";
/// A notch of the mouse wheel: it turns a page as `PAGE` does, silently.
pub const WHEEL: &str = "wheel";
/// The download button `id` on the model row at the argument's place.
pub const FETCH: &str = "fetch";
/// The switch of the option at the argument's place.
pub const FLIP: &str = "flip";
/// Step the option at the argument's place: `id` is `PLUS` or `MINUS`.
pub const STEP: &str = "step";
/// A step's plus, or a choice's next.
pub const PLUS: &str = "plus";
/// A step's minus, or a choice's last.
pub const MINUS: &str = "minus";
/// The panel button `id`.
pub const BUTTON: &str = "button";
/// Start Compare models with the set text.
pub const COMPARE: &str = "compare";
/// Answer the shown question, a card's switch-on among them: 1 is yes, 0 is no.
pub const CONFIRM: &str = "confirm";
/// The `cloud` switch: 1 turns cloud services on, 0 off.
pub const CLOUD: &str = "cloud";
/// The cloud button `id`.
pub const SERVICE: &str = "service";
/// The keep-nothing switch: 1 asks only for hosts that keep nothing, 0 not.
pub const KEEP: &str = "keep";
/// Show the Models page's side `id`.
pub const SIDE: &str = "side";
/// Use the local model after a cloud note.
pub const LOCAL: &str = "local";

/// True for a click that plays no key sound: the pointer's move and the wheel.
pub(super) fn silent(kind: &str) -> bool {
    matches!(kind, OVER | WHEEL)
}

/// A click QML should never send: an unknown kind or id, or an argument or round out of range.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Bad;

/// What a click asks of the hub.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Click {
    /// Send this click.
    Send(VoiceClick),
    /// Open the panel.
    Gear,
    /// Start Compare models, with the set text the hub words.
    Compare,
}

/// The round QML names, as the module counts it.
pub(super) fn round_of(round: i64) -> Result<u32, Bad> {
    u32::try_from(round).map_err(|_| Bad)
}

/// The click `kind` names with `id` and `arg`.
pub(super) fn click_of(kind: &str, id: &str, arg: i32) -> Result<Click, Bad> {
    let click = match kind {
        GEAR => return Ok(Click::Gear),
        COMPARE => return Ok(Click::Compare),
        BAR => VoiceClick::Bar(by_id(BarButton::ALL, bar_button_id, id)?),
        OVER => VoiceClick::Over(switch(arg)?),
        OPEN => VoiceClick::Open(by_id(VoiceTab::ALL, voice_tab_id, id)?),
        ROW => VoiceClick::Row(place(arg)?),
        PAGE | WHEEL => VoiceClick::Page { back: back(arg)? },
        FETCH => VoiceClick::Fetch {
            row: place(arg)?,
            button: by_id(FetchButton::ALL, fetch_button_id, id)?,
        },
        FLIP => VoiceClick::Flip(place(arg)?),
        STEP => VoiceClick::Step {
            row: place(arg)?,
            up: up(id)?,
        },
        BUTTON => VoiceClick::Button(by_id(PanelButton::ALL, panel_button_id, id)?),
        CONFIRM => VoiceClick::Confirm(switch(arg)?),
        CLOUD => VoiceClick::Cloud(switch(arg)?),
        SERVICE => VoiceClick::Service(by_id(CloudButton::ALL, cloud_button_id, id)?),
        KEEP => VoiceClick::KeepNothing(switch(arg)?),
        SIDE => VoiceClick::Side(by_id(ModelSide::ALL, model_side_id, id)?),
        LOCAL => VoiceClick::Local,
        _ => return Err(Bad),
    };
    Ok(Click::Send(click))
}

/// The member of `all` whose id, by `id_of`, is `id`.
fn by_id<T: Copy, const N: usize>(
    all: [T; N],
    id_of: fn(T) -> &'static str,
    id: &str,
) -> Result<T, Bad> {
    all.into_iter().find(|&t| id_of(t) == id).ok_or(Bad)
}

/// A place on the page.
fn place(arg: i32) -> Result<u8, Bad> {
    u8::try_from(arg).map_err(|_| Bad)
}

/// A page's way: back below 0, on above.
fn back(arg: i32) -> Result<bool, Bad> {
    match arg.cmp(&0) {
        Ordering::Less => Ok(true),
        Ordering::Greater => Ok(false),
        Ordering::Equal => Err(Bad),
    }
}

/// A step's way.
fn up(id: &str) -> Result<bool, Bad> {
    match id {
        PLUS => Ok(true),
        MINUS => Ok(false),
        _ => Err(Bad),
    }
}

/// 1 as on, 0 as off.
fn switch(arg: i32) -> Result<bool, Bad> {
    match arg {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Bad),
    }
}
