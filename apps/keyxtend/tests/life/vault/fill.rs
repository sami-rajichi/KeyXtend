//! Both from the panel (ADR-0038 decision 4): the keyboard types the saved user, one Tab and the password into the app
//! in front, the password after the Tab's gap and never Enter; nothing reaches the clipboard, and prediction hears no
//! value.

use super::{click, logins, names};
use crate::rig::{Rig, WAIT, hear};
use kx_mod_keyboard::BoardSpec;
use kx_mod_keyboard::spec::RowKey;
use kx_mod_vault::VaultSettings;
use kx_mod_vault::book::Notebook;
use kx_mod_vault::entry::{Draft, Stamp};
use kx_mod_vault::store::Store;
use kx_module_api::ports::{
    EntryKind, FaceAction, FillType, FillTyped, FillView, Piece, RowButton, Scan, Typed,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_platform_fake::Stroke;
use kx_ui::fill_acts::ROW;
use kx_ui::i18n::keys::row_button_id;
use std::time::Duration;

/// The saved login's name, user and password.
const NAME: &str = "Bank";
const USER: &str = "sam-user";
const PASSWORD: &str = "zebra-quartz";
/// The app in front, which must still be in front when the password goes.
const TARGET: &str = "target.exe";
/// When the saved login last changed.
const SAVED: Stamp = Stamp::from_unix_ms(1_700_000_000_000);

/// Saves a login into the rig's notebook store, as an earlier run would have.
fn saved_login(rig: &Rig) {
    let mut login = Draft::new(EntryKind::Login);
    login.name = Redacted::new(NAME.to_owned());
    for (field, value) in login.fields.iter_mut().zip([USER, PASSWORD]) {
        field.value = Redacted::new(Zeroizing::new(value.to_owned()));
    }
    let limits = VaultSettings::from_defaults().unwrap().limits();
    let mut book = Notebook::default();
    book.add(login, SAVED, &rig.random, &limits).unwrap();
    Store::new(rig.vault.clone()).save(&book).unwrap();
}

/// The scan code of the key a fill's Tab part taps, as `board.toml`'s `[fill]` names it.
fn tab() -> Scan {
    let spec = BoardSpec::shipped().unwrap();
    let mut keys = (0..spec.rows.len()).flat_map(|r| spec.row_keys(r));
    let named = |k: RowKey<'_>| match k {
        RowKey::Named(key) if key.id == spec.fill.tab => key.sc,
        _ => None,
    };
    keys.find_map(named).unwrap()
}

/// True when `piece` holds the saved user or password.
fn holds_value(piece: &Piece) -> bool {
    let Piece::Text(text) = piece else {
        return false;
    };
    [USER, PASSWORD].iter().any(|v| text.expose().contains(v))
}

/// Checks `strokes` are the saved user, one Tab and the password, with no Enter.
fn assert_login(strokes: &[Stroke]) {
    let [
        Stroke::Text(user, _),
        Stroke::Tap(key),
        Stroke::Text(password, _),
    ] = strokes
    else {
        panic!("three strokes, no Enter: {strokes:?}");
    };
    let sent = (user.expose().as_str(), *key, password.expose().as_str());
    assert_eq!(
        sent,
        (USER, tab(), PASSWORD),
        "the user, one Tab, the password"
    );
}

#[test]
fn both_types_the_user_a_tab_and_the_password_through_the_keyboard_alone() {
    let rig = Rig::new();
    saved_login(&rig);
    rig.injector.set_target(Some(TARGET));
    let mut boot = rig.load();
    let bus = boot.kernel.bus();
    let ((_v, views), (_w, words), (_d, done)) = (
        hear::<FillView>(&bus),
        hear::<Typed>(&bus),
        hear::<FillTyped>(&bus),
    );
    boot.start().unwrap();
    // Heard after the keyboard's own handler, so the parts before the gap went out once it is heard.
    let (_a, asks) = hear::<FillType>(&bus);
    assert_eq!(names(&logins(&boot, &views)), [NAME]);
    click(&boot, ROW, row_button_id(RowButton::Both), 0);
    let ask = asks.recv_timeout(WAIT).unwrap();
    let gap = VaultSettings::from_defaults().unwrap().tab_gap_ms;
    assert_eq!(ask.tab_gap, Duration::from_millis(gap.into()));
    rig.clock().advance(ask.tab_gap);
    let _ = boot.hub.act(FaceAction::Relabel);
    let answer = done.recv_timeout(WAIT).unwrap();
    assert_eq!((answer.round, answer.ok), (ask.round, true));
    assert_login(&rig.injector.strokes());
    assert!(
        rig.writer.writes().is_empty(),
        "nothing reached the clipboard"
    );
    let heard: Vec<Typed> = words.try_iter().collect();
    assert!(
        !heard.iter().any(|t| holds_value(&t.piece)),
        "prediction heard a value"
    );
}
