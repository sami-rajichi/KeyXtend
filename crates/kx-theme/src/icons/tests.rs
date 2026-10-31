//! Icon tests against the shipped `icons.toml`.

use super::*;

fn text() -> String {
    crate::ICONS.to_string()
}

/// The owner's icon for each shortcut with one (P6 plan, 2026-10-23); F1 to F12 show their text.
const SHORTCUTS: [(&str, &str); 25] = [
    ("copy", "copy"),
    ("paste", "clipboard-paste"),
    ("cut", "scissors"),
    ("select-all", "text-select"),
    ("undo", "undo-2"),
    ("redo", "redo-2"),
    ("copy-all", "clipboard-copy"),
    ("cut-all", "square-scissors"),
    ("delete-all", "eraser"),
    ("find", "search"),
    ("save", "save"),
    ("print", "printer"),
    ("emoji", "smile"),
    ("switch-app", "arrow-left-right"),
    ("show-desktop", "monitor-down"),
    ("file-explorer", "folder-open"),
    ("lock-pc", "lock"),
    ("task-manager", "activity"),
    ("close-window", "square-x"),
    ("new-tab", "plus"),
    ("close-tab", "x"),
    ("reopen-tab", "history"),
    ("refresh", "refresh-cw"),
    ("zoom-in", "a-arrow-up"),
    ("zoom-out", "a-arrow-down"),
];

#[test]
fn the_icons_load_with_the_theme_button_on_the_palette() {
    let i = parse(crate::ICONS).expect("icons.toml loads");
    assert_eq!(i.bar.theme, "palette", "the theme button");
    assert_eq!(
        (i.bar.mode.as_str(), i.bar.mode_on.as_str()),
        ("moon", "sun")
    );
    assert_eq!(i.keys["win"], "win-logo", "our own drawing");
    assert_eq!(i.keys["enter"], "corner-down-left", "mock-up keyInner");
    assert_eq!(
        (i.lang.cycle.as_str(), i.lang.list.as_str()),
        ("globe", "chevron-up")
    );
    assert_eq!(i.lang.current, "check", "the list's current row");
    assert_eq!(i.dpad.stop, "square");
    assert_eq!(
        (i.page.back.as_str(), i.page.on.as_str()),
        ("chevron-up", "chevron-down")
    );
    assert_eq!(i.notice.icon, "keyboard");
    assert_eq!(
        (i.tray.show.as_str(), i.tray.quit.as_str()),
        ("keyboard", "power")
    );
    assert!(i.names().contains(&"palette"));
}

#[test]
fn each_shortcut_shows_the_owners_icon_and_the_f_keys_show_their_text() {
    let i = parse(crate::ICONS).expect("icons.toml loads");
    let got: Vec<(&str, &str)> = i
        .shortcuts
        .iter()
        .map(|(id, n)| (id.as_str(), n.as_str()))
        .collect();
    let mut want = SHORTCUTS.to_vec();
    want.sort_unstable();
    assert_eq!(got, want);
    assert!(!i.shortcuts.contains_key("f5"), "F5 shows its text");
    assert_eq!(i.layer.next, "chevron-right", "the Next key");
    let named = i.named();
    assert!(named.contains(&("shortcuts.zoom-in".into(), "a-arrow-up")));
    assert!(named.contains(&("layer.next".into(), "chevron-right")));
}

#[test]
fn a_name_that_is_not_a_plain_file_name_is_refused() {
    for bad in ["../x", "Palette", "", "a b", "x.svg"] {
        let t = text().replacen("theme = \"palette\"", &format!("theme = {bad:?}"), 1);
        let e = parse(&t).expect_err(bad);
        assert!(e.contains("bar.theme"), "{bad}: {e}");
    }
}

#[test]
fn a_bad_tray_or_page_name_is_refused_with_where_it_is_used() {
    let t = text().replacen("quit = \"power\"", "quit = \"Power\"", 1);
    assert!(parse(&t).expect_err("a capital").contains("tray.quit"));
    let t = text().replacen("on = \"chevron-down\"", "on = \"../x\"", 1);
    assert!(parse(&t).expect_err("a path").contains("page.on"));
}

#[test]
fn a_bad_shortcut_or_next_icon_is_refused_with_where_it_is_used() {
    let t = text().replacen("cut = \"scissors\"", "cut = \"Scissors\"", 1);
    assert!(parse(&t).expect_err("a capital").contains("shortcuts.cut"));
    let t = text().replacen("next = \"chevron-right\"", "next = \"../x\"", 1);
    assert!(parse(&t).expect_err("a path").contains("layer.next"));
    let t = text().replacen("next = \"chevron-right\"", "nxt = \"chevron-right\"", 1);
    assert!(parse(&t).expect_err("a typo").contains("nxt"));
}

#[test]
fn the_chip_lane_and_its_menu_show_the_controllers_icons() {
    let i = parse(crate::ICONS).expect("icons.toml loads");
    let w = &i.words;
    let got = [
        &w.domain, &w.email, &w.more, &w.undo, &w.locked, &w.menu, &w.forget, &w.pause, &w.resume,
        &w.power, &w.cancel,
    ];
    let want = [
        "at-sign",
        "at-sign",
        "chevron-right",
        "undo-2",
        "lock",
        "square-menu",
        "eraser",
        "eye-off",
        "eye",
        "power",
        "x",
    ];
    assert_eq!(got.map(String::as_str), want);
    assert!(i.named().contains(&("words.menu".into(), "square-menu")));
}

#[test]
fn a_bad_words_icon_is_refused_with_where_it_is_used() {
    let t = text().replacen("forget = \"eraser\"", "forget = \"Eraser\"", 1);
    assert!(parse(&t).expect_err("a capital").contains("words.forget"));
}

#[test]
fn a_misspelt_or_missing_button_is_refused() {
    let typo = text().replacen("theme = \"palette\"", "theem = \"palette\"", 1);
    assert!(parse(&typo).expect_err("a typo").contains("theem"));
}

#[test]
fn the_clipboard_panel_shows_its_icons_by_role() {
    let i = parse(crate::ICONS).expect("icons.toml loads");
    let want = "title=clipboard-clock search=search close=x kind_text=file-text kind_link=link \
                kind_image=image kind_file=file-text kind_media=film pin=pin unpin=pin-off preview=eye \
                delete=trash-2 insert=clipboard-paste cancel=x shown=check room=hard-drive \
                folder=folder-open back=arrow-left";
    let named = i.clip.named();
    let got = named.map(|(role, name)| format!("{}={name}", role.trim_start_matches("clip.")));
    let want: Vec<&str> = want.split_whitespace().collect();
    assert_eq!(
        got.as_slice(),
        want.as_slice(),
        "mock-up buildClip and TYPEICON"
    );
    assert!(i.named().contains(&("clip.kind_media".into(), "film")));
    let t = text().replacen("unpin = \"pin-off\"", "unpin = \"Pin-off\"", 1);
    assert!(parse(&t).expect_err("a capital").contains("clip.unpin"));
}
