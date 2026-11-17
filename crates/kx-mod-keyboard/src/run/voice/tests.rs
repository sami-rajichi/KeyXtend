use super::*;

/// A text that appears nowhere else.
const SECRET: &str = "quartz-zebra";

/// A text of `t` in pieces of `step` characters, nothing sent yet.
fn text(t: &str, step: usize) -> Text {
    let pace = |step| Pace {
        step,
        gap: Duration::ZERO,
    };
    Text {
        round: 3,
        erase: 2,
        wipe: pace(PACED),
        body: Secret::new(Zeroizing::new(t.to_owned())),
        at: 0,
        pace: pace(step),
        count: 0,
        any: false,
        ok: true,
        keys: 0,
    }
}

#[test]
fn a_text_shows_its_round_and_counts_never_its_text() {
    let shown = format!("{:?}", text(SECRET, PACED));
    assert!(!shown.contains(SECRET), "{shown}");
    assert!(
        shown.contains("round: 3") && shown.contains("erase: 2"),
        "{shown}"
    );
}

#[test]
fn a_paced_run_goes_one_at_a_time_and_an_unpaced_one_in_pieces_of_the_cap() {
    let gap = Duration::from_millis(1);
    assert_eq!(Pace::new(gap, 40), Pace { step: PACED, gap });
    let at_once = Pace::new(Duration::ZERO, 40);
    assert_eq!((at_once.step, at_once.gap), (40, Duration::ZERO));
    assert_eq!(
        Pace::new(Duration::ZERO, 0).step,
        PACED,
        "never an empty piece"
    );
}

#[test]
fn pieces_hold_whole_characters_up_to_the_step() {
    let mut t = text("سَلام", 2);
    let mut pieces = Vec::new();
    while t.at < t.body.expose().len() {
        let (piece, chars) = t.next_piece();
        pieces.push((piece.expose().clone(), chars));
    }
    let want = [("سَ", 2), ("لا", 2), ("م", 1)].map(|(p, n)| (p.to_owned(), n));
    assert_eq!(pieces, want);
}

#[test]
fn a_stopped_text_has_nothing_left_and_answers_no() {
    let mut t = text("ab", PACED);
    assert!(t.left());
    assert_eq!(t.stop(), Part::Done(false));
    assert!(!t.left());
    let done = t.done();
    assert_eq!((done.round, done.count, done.ok), (3, 0, false));
}
