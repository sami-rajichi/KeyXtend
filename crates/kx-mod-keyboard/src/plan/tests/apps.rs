//! Key apps: the apps that take text as key presses, and when the app in front is asked.

use super::*;

#[test]
fn key_apps_get_text_as_key_presses() {
    let mut g = Given::new(true);
    (g.apps, g.app) = (vec!["notepad.exe".into()], Some("Notepad.exe".into()));
    assert_eq!(g.char(Q), text("q", TextRoute::Keys));
    let plan = g.plan(Role::Send, Some(ESC));
    assert_eq!(plan, send(Route::Tap(ESC), Wait::End), "keys stay keys");
}

#[test]
fn the_app_in_front_is_asked_only_for_text_and_only_when_key_apps_names_one() {
    let mut g = Given::new(true);
    g.char(Q);
    assert_eq!(g.asked.get(), 0, "no key apps");
    (g.apps, g.app) = (vec!["notepad.exe".into()], Some("chrome.exe".into()));
    g.plan(Role::Send, Some(ESC));
    g.plan(Role::Latch(Latch::Shift), Some(SHIFT));
    g.char(HAT);
    assert_eq!(g.asked.get(), 0, "no text");
    assert_eq!(g.char(Q), unicode("q"));
    assert_eq!(g.asked.get(), 1);
}

#[test]
fn a_key_app_is_matched_by_file_name_without_case() {
    let apps = vec![
        "notepad.exe".to_owned(),
        " ApplicationFrameHost.exe ".to_owned(),
    ];
    let route = |app: Option<&str>| text_route(&apps, || app.map(str::to_owned));
    assert_eq!(route(Some("Notepad.EXE")), TextRoute::Keys);
    assert_eq!(
        route(Some("applicationframehost.exe")),
        TextRoute::Keys,
        "every UWP app"
    );
    assert_eq!(route(Some("notepad")), TextRoute::Unicode);
    assert_eq!(route(Some("chrome.exe")), TextRoute::Unicode);
    assert_eq!(route(None), TextRoute::Unicode, "no app in front");
    let none = text_route(&[], || Some("notepad.exe".into()));
    assert_eq!(none, TextRoute::Unicode);
}
