//! The tracker's emails and plans: `@@`, one `@`, known-email starts, finished emails and what a pick erases.

use kx_module_api::Redacted;

use super::*;

#[test]
fn a_typed_at_at_asks_for_every_email_and_one_at_for_domains() {
    let mut t = tracker(2);
    type_text(&mut t, "@@");
    let show = t.show(&knows);
    assert_eq!(show.words, None);
    assert_eq!(
        show.emails,
        Some(EmailAsk::All(Redacted::new("@@".to_owned())))
    );
    let mut t = tracker(2);
    type_text(&mut t, "sami@g");
    let show = t.show(&knows);
    assert_eq!(show.words, None);
    assert_eq!(
        show.emails,
        Some(EmailAsk::Domains(Redacted::new("g".to_owned())))
    );
    let mut t = tracker(2);
    type_text(&mut t, "@");
    assert_eq!(t.show(&knows).emails, None);
}

#[test]
fn an_at_at_that_came_from_a_read_never_lists_emails() {
    let mut t = tracker(2);
    t.read(Some("write to @@"), &knows);
    assert_eq!(t.show(&knows).emails, None);
    type_text(&mut t, "@@");
    assert_eq!(
        t.show(&knows).emails,
        Some(EmailAsk::All(Redacted::new("@@".to_owned())))
    );
}

#[test]
fn the_start_of_a_token_may_start_a_known_email() {
    let mut t = tracker(2);
    type_text(&mut t, "(sami.r");
    let show = t.show(&knows);
    assert_eq!(
        show.emails,
        Some(EmailAsk::Start(Redacted::new("sami.r".to_owned())))
    );
}

#[test]
fn a_finished_email_is_told_never_learned_and_clears_the_context() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "mail a@b.co ");
    assert!(told.contains(&Told::Email(Redacted::new("a@b.co".to_owned()))));
    assert_eq!(learned(&told), [("mail".to_owned(), owned(&[]))]);
    assert_eq!(t.context(), Some(&[][..]));
}

#[test]
fn plans_erase_what_they_replace_and_add_a_space_only_after_a_word() {
    let mut t = tracker(2);
    type_text(&mut t, "say héllo");
    let query = t.show(&knows).words.unwrap();
    let plan = query.plan("hello").unwrap();
    assert_eq!(
        (
            plan.erase,
            plan.gone.expose().as_str(),
            plan.text.expose().as_str()
        ),
        (5, "héllo", "hello ")
    );
    let all = EmailAsk::All(Redacted::new("@@".to_owned()))
        .plan("a@b.co")
        .unwrap();
    assert_eq!((all.erase, all.text.expose().as_str()), (2, "a@b.co"));
    let domain = EmailAsk::Domains(Redacted::new("gm".to_owned()))
        .plan("gmail.com")
        .unwrap();
    assert_eq!(
        (domain.erase, domain.text.expose().as_str()),
        (2, "gmail.com")
    );
}

#[test]
fn an_elision_chip_adds_no_space_and_one_that_changes_nothing_is_no_change() {
    let mut t = tracker(2);
    type_text(&mut t, "l");
    let query = t.show(&knows).words.unwrap();
    let plan = query.plan("l'").map(|p| p.text.expose().clone());
    assert_eq!(plan.as_deref(), Some("l'"));
    assert!(query.changes("l'") && query.changes("l"));
    type_text(&mut t, "'");
    let query = t.show(&knows).words.unwrap();
    assert!(!query.changes("l'"));
    assert!(query.changes("l'école"));
}
