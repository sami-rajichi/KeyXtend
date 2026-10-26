//! What a pick sends: the exact common start of what it replaces and the chip stays, by `char` with case and marks,
//! so only the rest is erased and typed; the tracker then follows those pieces to the chip's word.

use kx_module_api::Redacted;

use super::*;

/// The open word's plan for `guess` after typing `typed`: its erase, what it replaces, and its text.
fn word_plan(typed: &str, guess: &str) -> Option<(u16, String, String)> {
    let mut t = tracker(2);
    type_text(&mut t, typed);
    let plan = t.show(&knows).words?.plan(guess)?;
    Some(parts_of(&plan))
}

/// A plan's erase, what it replaces, and its text.
fn parts_of(plan: &Plan) -> (u16, String, String) {
    let (gone, text) = (plan.gone.expose(), plan.text.expose());
    (plan.erase, gone.clone(), text.clone())
}

/// Sends `plan` to `t` as the keyboard tells it: one erase per character, then the text.
fn apply(t: &mut Tracker, plan: &Plan) -> Vec<Told> {
    let mut told = Vec::new();
    for _ in 0..plan.erase {
        told.extend(t.typed(&Piece::Erase, &knows));
    }
    told.extend(t.typed(&Piece::Text(plan.text.clone()), &knows));
    told
}

#[test]
fn a_word_plan_keeps_the_common_start_and_erases_only_the_rest() {
    let cases = [
        ("he", "help", (0, "he", "lp ")),
        ("hEl", "help", (2, "hEl", "elp ")),
        ("say héllo", "hello", (4, "héllo", "ello ")),
        ("ان", "أن", (2, "ان", "أن ")),
        ("كت", "كتاب", (0, "كت", "اب ")),
    ];
    for (typed, guess, (erase, gone, text)) in cases {
        let want = (erase, gone.to_owned(), text.to_owned());
        assert_eq!(word_plan(typed, guess), Some(want), "{typed} {guess}");
    }
}

#[test]
fn a_start_that_differs_only_in_case_erases_from_the_first_different_char() {
    let want = (3, "Hel".to_owned(), "help ".to_owned());
    assert_eq!(word_plan("Hel", "help"), Some(want));
    let want = (0, "Hel".to_owned(), "p ".to_owned());
    assert_eq!(word_plan("Hel", "Help"), Some(want));
}

#[test]
fn an_apostrophe_pick_keeps_the_common_start_by_exact_char() {
    let curly = "l\u{2019}éc";
    let styled = (0, curly.to_owned(), "ole ".to_owned());
    assert_eq!(word_plan(curly, "l\u{2019}école"), Some(styled));
    let plain = (3, curly.to_owned(), "'école ".to_owned());
    assert_eq!(word_plan(curly, "l'école"), Some(plain), "another form");
    let elision = (0, "l".to_owned(), "'".to_owned());
    assert_eq!(word_plan("l", "l'"), Some(elision), "no gap after it");
    assert_eq!(word_plan("l'", "l'"), None, "a plan that changes nothing");
}

#[test]
fn email_plans_type_the_rest_and_an_at_at_is_erased() {
    let ask = |a: EmailAsk, text: &str| a.plan(text).as_ref().map(parts_of);
    let at = EmailAsk::All(Redacted::new("@@".to_owned()));
    let all = (2, "@@".to_owned(), "a@b.co".to_owned());
    assert_eq!(ask(at, "a@b.co"), Some(all));
    let gm = EmailAsk::Domains(Redacted::new("gm".to_owned()));
    let domain = (0, "gm".to_owned(), "ail.com".to_owned());
    assert_eq!(ask(gm, "gmail.com"), Some(domain));
    let sam = EmailAsk::Start(Redacted::new("sam".to_owned()));
    let start = (0, "sam".to_owned(), "i@x.org".to_owned());
    assert_eq!(ask(sam, "sami@x.org"), Some(start));
    let cap = EmailAsk::Start(Redacted::new("Sam".to_owned()));
    let whole = (3, "Sam".to_owned(), "sami@x.org".to_owned());
    assert_eq!(ask(cap, "sami@x.org"), Some(whole));
}

#[test]
fn a_plan_still_names_its_whole_chip() {
    let mut t = tracker(2);
    type_text(&mut t, "hEl");
    let plan = t.show(&knows).words.unwrap().plan("help").unwrap();
    assert_eq!(plan.chip().expose(), "help ");
    let sam = EmailAsk::Start(Redacted::new("Sam".to_owned()));
    let whole = sam.plan("sami@x.org").unwrap().chip();
    assert_eq!(whole.expose(), "sami@x.org");
    let gm = EmailAsk::Domains(Redacted::new("gm".to_owned()));
    assert_eq!(gm.plan("gmail.com").unwrap().chip().expose(), "gmail.com");
}

#[test]
fn a_box_that_cannot_confirm_gets_a_plan_only_when_it_erases_nothing() {
    let mut t = tracker(2);
    type_text(&mut t, "he");
    let query = t.show(&knows).words.unwrap();
    let plan = query.plan("help").unwrap();
    assert_eq!(plan.rest(), Some(plan.clone()));
    let plan = query.plan("Help").unwrap();
    assert_eq!(plan.rest(), None, "it erases the typed h");
}

#[test]
fn after_a_word_pick_the_tracker_holds_the_chip_word() {
    for (typed, guess) in [("he", "help"), ("hEl", "help"), ("ان", "أن")] {
        let mut t = tracker(2);
        type_text(&mut t, typed);
        let plan = t.show(&knows).words.unwrap().plan(guess).unwrap();
        let told = apply(&mut t, &plan);
        assert_eq!(learned(&told), [(guess.to_owned(), owned(&[]))], "{typed}");
        assert_eq!(t.context(), Some(&[guess.to_owned()][..]));
    }
}

#[test]
fn after_an_email_pick_the_tracker_holds_the_whole_address() {
    let mut t = tracker(2);
    type_text(&mut t, "sami@gm");
    let ask = t.show(&knows).emails.unwrap();
    apply(&mut t, &ask.plan("gmail.com").unwrap());
    let told = type_text(&mut t, " ");
    assert_eq!(
        told,
        [Told::Email(Redacted::new("sami@gmail.com".to_owned()))]
    );
    let mut t = tracker(2);
    type_text(&mut t, "to @@");
    let ask = t.show(&knows).emails.unwrap();
    apply(&mut t, &ask.plan("a@b.co").unwrap());
    let told = type_text(&mut t, " ");
    assert_eq!(told, [Told::Email(Redacted::new("a@b.co".to_owned()))]);
}
