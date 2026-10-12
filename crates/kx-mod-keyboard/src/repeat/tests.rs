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

/// A machine with `A` pressed, and so typed, at the start.
fn pressed() -> Repeat<&'static str> {
    let mut r = machine();
    r.press(A, at(0));
    r
}

#[test]
fn a_press_repeats_nothing_before_the_delay() {
    let mut r = pressed();
    assert!(r.held());
    assert_eq!((r.tick(at(0)), r.tick(at(DELAY_MS - 1))), (None, None));
}

#[test]
fn the_key_held_is_the_last_one_pressed_until_the_hold_ends() {
    let mut r = pressed();
    assert_eq!(r.key(), Some(&A));
    r.press(B, at(1));
    assert_eq!(r.key(), Some(&B));
    r.leave();
    assert_eq!(r.key(), None);
}

#[test]
fn a_hold_types_at_the_delay_then_every_interval() {
    let mut r = pressed();
    assert_eq!(r.tick(at(DELAY_MS)), Some(A));
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
fn a_tick_at_the_same_time_again_types_once_not_twice() {
    let mut r = pressed();
    assert_eq!(r.tick(at(DELAY_MS)), Some(A));
    assert_eq!(r.tick(at(DELAY_MS)), None, "the same tick again");
}

#[test]
fn leaving_stops_it_before_or_after_a_repeat() {
    let mut r = pressed();
    r.tick(at(DELAY_MS));
    r.leave();
    assert!(!r.held());
    assert_eq!(r.tick(at(DELAY_MS + EVERY_MS)), None);
    let mut early = pressed();
    early.leave();
    assert_eq!(early.tick(at(DELAY_MS)), None, "left before the delay");
}

#[test]
fn nothing_pressed_never_types() {
    let mut r = machine();
    assert!(!r.held());
    assert_eq!(r.tick(at(DELAY_MS * 10)), None);
}

#[test]
fn a_late_tick_types_once_and_skips_the_missed_beats() {
    let mut r = pressed();
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
fn a_new_press_drops_the_last_hold() {
    let mut r = pressed();
    r.tick(at(DELAY_MS));
    r.press(B, at(DELAY_MS + 50));
    assert_eq!(r.tick(at(DELAY_MS + EVERY_MS)), None, "A stopped, B waits");
    assert_eq!(r.tick(at(DELAY_MS * 2 + 50)), Some(B));
}

#[test]
fn debug_never_shows_the_key() {
    let mut r = pressed();
    r.tick(at(DELAY_MS));
    let shown = format!("{r:?}");
    assert!(!shown.contains(A) && shown.contains("held"), "{shown}");
}

/// One input from the pointer or the hold timer.
#[derive(Clone, Debug)]
enum Step {
    /// A key goes down and types.
    Press(u8),
    /// The timer ticks this many ms later.
    Tick(u64),
    /// The pointer lets go, Qt takes the press away, or the keyboard hides.
    Leave,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        (0u8..3).prop_map(Step::Press),
        (0..DELAY_MS / 2).prop_map(Step::Tick),
        (0..DELAY_MS / 2).prop_map(Step::Tick),
        Just(Step::Leave),
    ]
}

/// The key held now, when it went down, and the beat of each repeat it typed.
type Model = Option<(u8, Mono, Vec<u64>)>;

/// Checks one tick that typed `key` at `now` against the model, and notes its beat.
fn typed_on_tick(model: &mut Model, key: u8, now: Mono) -> Result<(), TestCaseError> {
    let Some((held, since, beats)) = model.as_mut() else {
        return Err(TestCaseError::fail("a tick typed after let-go"));
    };
    prop_assert_eq!(key, *held);
    let start = since.saturating_add(ms(DELAY_MS)).as_us();
    prop_assert!(now.as_us() >= start, "a repeat before the delay");
    let beat = (now.as_us() - start) / (EVERY_MS * u64::from(US_PER_MS));
    prop_assert!(beats.last().is_none_or(|&b| beat > b), "two in one beat");
    beats.push(beat);
    Ok(())
}

proptest! {
    #[test]
    fn a_hold_repeats_only_while_held_and_once_a_beat_after_the_delay(steps in vec(step(), 0..60)) {
        let mut r = Repeat::new(ms(DELAY_MS), ms(EVERY_MS));
        let (mut now, mut model): (Mono, Model) = (Mono::default(), None);
        for s in steps {
            match s {
                Step::Press(k) => {
                    r.press(k, now);
                    model = Some((k, now, Vec::new()));
                }
                Step::Tick(n) => {
                    now = now.saturating_add(ms(n));
                    if let Some(k) = r.tick(now) {
                        typed_on_tick(&mut model, k, now)?;
                    }
                }
                Step::Leave => {
                    r.leave();
                    model = None;
                }
            }
        }
    }
}
