//! Tests of `apps`.

use super::*;

fn app(args: &[&str], class: Option<&str>) -> AppConfig {
    AppConfig {
        exe: "x.exe".into(),
        args: args.iter().map(ToString::to_string).collect(),
        class: class.map(String::from),
        focus_class: None,
        new_window: true,
        wait_ms: 0,
        ready_ms: 0,
        done_ms: 0,
    }
}

#[test]
fn attach_needs_a_class_and_no_prepared_input() {
    let word = app(&["/q", "/w"], Some("OpusApp"));
    assert_eq!(attach_class("word", &word), Ok("OpusApp"));
    let notepad = app(&[INPUT], Some("Notepad"));
    assert!(attach_class("notepad", &notepad).is_err());
    let chrome = app(&["--new-window", INPUT], Some("Chrome_WidgetWin_1"));
    assert!(attach_class("chrome", &chrome).is_err());
    assert!(attach_class("target", &app(&[], None)).is_err());
}

#[test]
fn only_word_can_be_attached_with_the_shipped_file() {
    let cfg = crate::config::load().expect("kx-gates.toml loads");
    let ok: Vec<&str> = AppKind::ALL
        .iter()
        .map(|k| k.name())
        .filter(|&n| attach_class(n, cfg.app(n).expect("app")).is_ok())
        .collect();
    assert_eq!(ok, [AppKind::Word.name()]);
}

#[test]
fn only_titles_that_start_with_a_test_page_count() {
    let pages = ["KeyXtend probe page", ""];
    assert!(is_test_page(
        "KeyXtend probe page g6-x - Google Chrome",
        &pages
    ));
    assert!(!is_test_page("News - KeyXtend probe page", &pages));
    assert!(!is_test_page("Mail - Google Chrome", &pages));
    assert!(!is_test_page("anything", &[""]));
}

#[test]
fn same_program_compares_file_names_ignoring_case() {
    let word = r"C:\Program Files\Microsoft Office\root\Office16\WINWORD.EXE";
    assert!(same_program(
        r"C:\Program Files\Microsoft Office\root\Office16\winword.exe",
        word
    ));
    assert!(same_program(r"D:\Other\WinWord.exe", "WINWORD.EXE"));
    assert!(!same_program(r"C:\Windows\notepad.exe", word));
    assert!(!same_program("", word));
}
