use super::*;
use kx_module_api::US_PER_MS;
use proptest::collection::vec;
use proptest::prelude::*;

/// The owner's defaults, in ms: the first repeat after 0.8 s, then 10 a second.
const DELAY_MS: u64 = 800;
const EVERY_MS: u64 = 100;
const A: &str = "c-0x001E";
const B: &str = "c-0x0030";

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// The time `n` ms after the start.
fn at(n: u64) -> Mono {
    Mono::default().saturating_add(ms(n))
}

fn machine() -> Repeat<&'static str> {
    Repeat::new(ms(DELAY_MS), ms(EVERY_MS))
}

/// A machine with `A` pressed at the start, repeating when `repeats`.
fn held(repeats: bool) -> Repeat<&'static str> {
    let mut r = machine();
    r.press(A, repeats, at(0));
    r
}

#[test]
fn a_release_before_the_delay_types_once_on_release() {
    let mut r = held(true);
    assert_eq!((r.tick(at(100)), r.tick(at(DELAY_MS - 1))), (None, None));
    assert_eq!(r.let_go(&A), Some(A));
    assert_eq!(r.tick(at(DELAY_MS * 2)), None, "nothing after let-go");
    assert!(!r.bursting());
}

#[test]
fn a_hold_types_at_the_delay_then_every_interval() {
    let mut r = held(true);
    assert_eq!(r.tick(at(DELAY_MS)), Some(A));
    assert!(r.bursting());
    assert_eq!(r.tick(at(DELAY_MS + EVERY_MS / 2)), None);
    for beat in 1..=3 {
        assert_eq!(
            r.tick(at(DELAY_MS + beat * EVERY_MS)),
            Some(A),
            "beat {beat}"
        );
    }
}

#[test]
fn a_hold_of_exactly_the_delay_types_once_not_twice() {
    let mut r = held(true);
    assert_eq!(r.tick(at(DELAY_MS)), Some(A));
    assert_eq!(r.tick(at(DELAY_MS)), None, "the same tick again");
    assert_eq!(r.let_go(&A), None, "the let-go adds nothing");
}

#[test]
fn letting_go_after_a_repeat_types_nothing() {
    let mut r = held(true);
    r.tick(at(DELAY_MS));
    assert_eq!(r.let_go(&A), None);
    assert_eq!(r.tick(at(DELAY_MS * 3)), None);
    assert!(!r.bursting());
}

#[test]
fn leaving_the_key_stops_it_and_a_later_let_go_types_nothing() {
    let mut r = held(true);
    r.tick(at(DELAY_MS));
    r.leave();
    assert_eq!(r.tick(at(DELAY_MS + EVERY_MS)), None);
    assert_eq!(r.let_go(&A), None);
    let mut early = held(true);
    early.leave();
    assert_eq!(
        (early.tick(at(DELAY_MS)), early.let_go(&A)),
        (None, None),
        "left before the delay"
    );
}

#[test]
fn a_key_that_does_not_repeat_types_only_on_let_go() {
    let mut r = held(false);
    assert_eq!(r.tick(at(DELAY_MS * 10)), None);
    assert!(!r.bursting());
    assert_eq!(r.let_go(&A), Some(A));
}

#[test]
fn a_late_tick_types_once_and_skips_the_missed_beats() {
    let mut r = held(true);
    r.tick(at(DELAY_MS));
    let late = DELAY_MS + EVERY_MS * 5 + EVERY_MS / 2;
    assert_eq!(r.tick(at(late)), Some(A), "once, not five times");
    assert_eq!(r.tick(at(late + 1)), None);
    assert_eq!(
        r.tick(at(DELAY_MS + EVERY_MS * 6)),
        Some(A),
        "back on the pace"
    );
}

#[test]
fn a_let_go_of_another_key_types_nothing_and_ends_the_hold() {
    let mut r = held(true);
    assert_eq!(r.let_go(&B), None);
    assert_eq!(r.tick(at(DELAY_MS)), None);
}

#[test]
fn a_new_press_drops_the_last_hold() {
    let mut r = held(true);
    r.tick(at(DELAY_MS));
    r.press(B, true, at(DELAY_MS + 50));
    assert_eq!(r.tick(at(DELAY_MS + EVERY_MS)), None, "A stopped, B waits");
    assert_eq!(r.tick(at(DELAY_MS * 2 + 50)), Some(B));
}

#[test]
fn debug_never_shows_the_key() {
    let mut r = held(true);
    r.tick(at(DELAY_MS));
    let shown = format!("{r:?}");
    assert!(!shown.contains(A) && shown.contains("repeating"), "{shown}");
}

/// One input from the pointer or the hold timer.
#[derive(Clone, Debug)]
enum Step {
    /// A key goes down, repeating or not.
    Press(u8, bool),
    /// The timer ticks this many ms later.
    Tick(u64),
    /// The pointer lets go of a key.
    LetGo(u8),
    /// The pointer leaves the key.
    Leave,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        (0u8..3, any::<bool>()).prop_map(|(k, r)| Step::Press(k, r)),
        (0..DELAY_MS / 2).prop_map(Step::Tick),
        (0..DELAY_MS / 2).prop_map(Step::Tick),
        (0u8..3).prop_map(Step::LetGo),
        Just(Step::Leave),
    ]
}

/// The key held now, whether it repeats, when it went down, and the beat of each repeat it typed.
type Model = Option<(u8, bool, Mono, Vec<u64>)>;

/// Checks one tick that typed `key` at `now` against the model, and notes its beat.
fn typed_on_tick(model: &mut Model, key: u8, now: Mono) -> Result<(), TestCaseError> {
    let Some((held, repeats, since, beats)) = model.as_mut() else {
        return Err(TestCaseError::fail("a tick typed after let-go or leave"));
    };
    prop_assert_eq!(key, *held);
    prop_assert!(*repeats, "a key that does not repeat typed on a tick");
    let start = since.saturating_add(ms(DELAY_MS)).as_us();
    prop_assert!(now.as_us() >= start, "a repeat before the delay");
    let beat = (now.as_us() - start) / (EVERY_MS * u64::from(US_PER_MS));
    prop_assert!(beats.last().is_none_or(|&b| beat > b), "two in one beat");
    beats.push(beat);
    Ok(())
}

proptest! {
    #[test]
    fn a_hold_types_only_while_held_once_a_beat_and_only_for_a_repeating_key(steps in vec(step(), 0..60)) {
        let mut r = Repeat::new(ms(DELAY_MS), ms(EVERY_MS));
        let (mut now, mut model): (Mono, Model) = (Mono::default(), None);
        for s in steps {
            match s {
                Step::Press(k, rep) => {
                    r.press(k, rep, now);
                    model = Some((k, rep, now, Vec::new()));
                }
                Step::Tick(n) => {
                    now = now.saturating_add(ms(n));
                    if let Some(k) = r.tick(now) {
                        typed_on_tick(&mut model, k, now)?;
                    }
                }
                Step::LetGo(k) => {
                    let click = model.take().is_some_and(|(h, _, _, b)| h == k && b.is_empty());
                    prop_assert_eq!(r.let_go(&k).is_some(), click, "a let-go types only a click");
                }
                Step::Leave => {
                    r.leave();
                    model = None;
                }
            }
        }
    }
}
