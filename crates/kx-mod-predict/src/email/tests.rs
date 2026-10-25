//! Emails: the table, the shape check, domain chips, known-email chips, `@@` with More, and the paged list.

use super::*;

/// The settings' defaults as Task 12 fills them.
fn cfg() -> EmailConfig {
    let domains = [
        "gmail.com",
        "outlook.com",
        "hotmail.com",
        "yahoo.com",
        "icloud.com",
    ];
    EmailConfig {
        max_emails: 50,
        domains: domains.map(str::to_owned).to_vec(),
        min_start: 2,
    }
}

/// Each chip's text and kind.
fn seen(chips: &[Chip]) -> Vec<(&str, ChipKind)> {
    chips
        .iter()
        .map(|c| (c.text.expose().as_str(), c.kind))
        .collect()
}

/// Known emails added oldest first, so the last is the most recent.
fn known(emails: &[&str]) -> Emails {
    let mut known = Emails::default();
    for e in emails {
        assert!(known.add(e, cfg().max_emails), "{e}");
    }
    known
}

#[test]
fn the_table_parses_and_names_the_at_sign() {
    assert!(MARKS.is_some());
    assert_eq!(at(), Some('@'));
    assert_eq!(at_run(), Some(2));
    assert!(is_email_char('+') && is_email_char('é') && !is_email_char(','));
}

#[test]
fn only_local_at_domain_dot_tld_looks_like_an_email() {
    let good = [
        "a@b.co",
        "first.last+tag@mail.example.com",
        "x_y-z@a-b.org",
        "o'brien@x.com",
        "!#$%&'*+/=?^_`{|}~-.@x.co",
    ];
    for ok in good {
        assert!(looks_like(ok), "{ok}");
    }
    let bad = [
        "a@b", "@b.co", "a@@b.co", "a@b..co", "a@-b.co", "a@b.co.", "a b@c.co", "a,b@c.co", "ab.co",
    ];
    for no in bad {
        assert!(!looks_like(no), "{no}");
    }
}

#[test]
fn an_arabic_address_looks_like_an_email_and_its_chip_reads_right_to_left() {
    let email = "سامي@مثال.شبكة";
    assert!(looks_like(email));
    let chips = known(&[email]).all(cfg().domains.len());
    assert!(chips.first().is_some_and(|c| c.rtl));
}

#[test]
fn the_tail_is_the_email_like_end_of_a_token() {
    assert_eq!(tail("mailto:sami@x.org"), "sami@x.org");
    assert_eq!(tail("(sami.r"), "sami.r");
    assert_eq!(tail("hello,"), "");
}

#[test]
fn a_capture_takes_the_run_around_the_at_sign_less_the_trailing_marks() {
    assert_eq!(capture("o'brien@x.com"), Some("o'brien@x.com"));
    let wrapped = [
        "sami@x.org.",
        "sami@x.org,",
        "(sami@x.org)",
        "<sami@x.org>",
        "mailto:sami@x.org",
        "sami@x.org;bob@y.org",
    ];
    for token in wrapped {
        assert_eq!(capture(token), Some("sami@x.org"), "{token}");
    }
    assert_eq!(capture("سامي@مثال.شبكة»"), Some("سامي@مثال.شبكة"));
    assert_eq!(capture("a@b@c.co"), None);
    assert_eq!(capture("hello"), None);
}

#[test]
fn sami_at_g_lists_gmail_first_and_never_the_domain_already_typed() {
    let cfg = cfg();
    let lane = cfg.domains.len();
    assert_eq!(
        seen(&domains("g", &cfg, lane)),
        [("gmail.com", ChipKind::Domain)]
    );
    assert_eq!(
        seen(&domains("GM", &cfg, lane)),
        [("gmail.com", ChipKind::Domain)]
    );
    assert!(domains("gmail.com", &cfg, lane).is_empty());
    let all = domains("", &cfg, lane);
    let texts: Vec<&str> = seen(&all).iter().map(|s| s.0).collect();
    assert_eq!(texts, cfg.domains);
    assert_eq!(domains("", &cfg, 2).len(), 2);
}

#[test]
fn the_start_of_a_known_email_shows_it_from_two_characters_and_before_any_at() {
    let cfg = cfg();
    let emails = known(&["sami@x.org"]);
    let lane = cfg.domains.len();
    assert_eq!(
        seen(&emails.starting("sa", &cfg, lane)),
        [("sami@x.org", ChipKind::Email)]
    );
    assert_eq!(
        seen(&emails.starting("SA", &cfg, lane)),
        [("sami@x.org", ChipKind::Email)]
    );
    assert!(emails.starting("s", &cfg, lane).is_empty());
    assert!(emails.starting("sami@", &cfg, lane).is_empty());
    assert!(emails.starting("sami@x.org", &cfg, lane).is_empty());
    assert!(emails.starting("so", &cfg, lane).is_empty());
}

#[test]
fn a_typed_at_at_lists_known_emails_most_recent_first_with_more_past_the_lane() {
    let emails = known(&["a@x.co", "b@x.co", "c@x.co", "d@x.co", "e@x.co", "f@x.co"]);
    let chips = emails.all(4);
    let expected = [
        ("f@x.co", ChipKind::Email),
        ("e@x.co", ChipKind::Email),
        ("d@x.co", ChipKind::Email),
        ("", ChipKind::More),
    ];
    assert_eq!(seen(&chips), expected);
    assert_eq!(emails.all(6).len(), 6);
    assert!(emails.all(6).iter().all(|c| c.kind == ChipKind::Email));
    assert!(emails.all(0).is_empty());
}

#[test]
fn a_finished_email_is_added_once_and_moves_to_the_front_when_reused() {
    let mut emails = known(&["a@b.co", "c@d.co"]);
    assert!(emails.add("A@B.co", cfg().max_emails));
    assert_eq!(emails.list(), ["A@B.co", "c@d.co"]);
    assert!(!emails.add("not an email", cfg().max_emails));
    assert!(!emails.add("word", cfg().max_emails));
    assert_eq!(emails.list().len(), 2);
}

#[test]
fn the_list_keeps_the_most_recent_up_to_its_cap() {
    let mut emails = Emails::default();
    for e in ["a@x.co", "b@x.co", "c@x.co", "d@x.co"] {
        emails.add(e, 3);
    }
    assert_eq!(emails.list(), ["d@x.co", "c@x.co", "b@x.co"]);
    let saved = ["a@x.co", "b@x.co", "A@x.co", "x"].map(str::to_owned);
    let loaded = Emails::new(&saved, 3);
    assert_eq!(loaded.list(), ["a@x.co", "b@x.co"]);
}

#[test]
fn the_list_pages_by_rows_and_a_page_past_the_end_shows_the_last() {
    let mut emails = Emails::default();
    for i in 0..12 {
        emails.add(&format!("u{i}@x.co"), cfg().max_emails);
    }
    let first = emails.page(0, 5);
    assert_eq!((first.page, first.pages, first.items.len()), (0, 3, 5));
    assert_eq!(
        first.items.first().map(|e| e.expose().as_str()),
        Some("u11@x.co")
    );
    let last = emails.page(9, 5);
    assert_eq!((last.page, last.items.len()), (2, 2));
    let empty = Emails::default().page(0, 5);
    assert_eq!((empty.page, empty.pages, empty.items.len()), (0, 1, 0));
}

#[test]
fn debug_never_shows_an_email() {
    let emails = known(&["secret@x.co"]);
    assert!(!format!("{emails:?}").contains("secret"));
}
