//! Automatic's rule and a picked card's (ADR-0044 decision 8), and a card check's verdict (decision 7, edge case 27).

use super::*;
use kx_module_api::ports::{DeviceKind, Memory};
use kx_module_api::{Redacted, Zeroizing};

/// The model's card need and the margin, in MB.
const NEED: u32 = 960;
const MARGIN: u32 = 512;
/// The room a card must have free.
const ROOM: u32 = NEED + MARGIN;

/// A card named `id` with `free_mb` free.
fn card(id: &str, free_mb: u32) -> Device {
    let memory = Memory {
        total_mb: 8192,
        free_mb,
    };
    let kind = DeviceKind::Card(CardId::new(id).unwrap());
    let name = format!("{id} card");
    Device {
        kind,
        name,
        memory,
        driver: "1.0".into(),
    }
}

/// A pass `x` hundredths faster.
fn pass(x: u32) -> Verdict {
    Verdict::Passed {
        gain: Hundredths(x),
    }
}

/// Where `pick` runs the model among `cards`, each with its check.
fn run(pick: &DevicePick, cards: &[(&Device, Option<Verdict>)]) -> Serve {
    let seen: Vec<Seen<'_>> = cards
        .iter()
        .map(|&(device, check)| Seen { device, check })
        .collect();
    serve(pick, &seen, NEED, MARGIN)
}

/// The card use `id` gets.
fn on(id: &str) -> Serve {
    let id = CardId::new(id).unwrap();
    Serve::Card(CardUse {
        id,
        need_mb: NEED,
        margin_mb: MARGIN,
    })
}

/// The processor, as the card has only `free_mb` free.
fn crowded(free_mb: u32) -> Serve {
    Serve::Cpu(Why::NoRoom {
        need_mb: ROOM,
        free_mb,
    })
}

#[test]
fn the_processor_pick_never_uses_a_card() {
    let fast = card("Vulkan0", 8000);
    assert_eq!(
        run(&DevicePick::Cpu, &[(&fast, Some(pass(900)))]),
        Serve::Cpu(Why::Picked)
    );
}

#[test]
fn automatic_uses_the_processor_until_a_card_passes_its_check() {
    let a = card("Vulkan0", 8000);
    let slow = Verdict::Slow {
        gain: Hundredths(120),
        need: Hundredths(150),
    };
    for cards in [vec![], vec![(&a, None)], vec![(&a, Some(slow))]] {
        assert_eq!(
            run(&DevicePick::Auto, &cards),
            Serve::Cpu(Why::NoneChecked),
            "{cards:?}"
        );
    }
    assert_eq!(
        run(&DevicePick::Auto, &[(&a, Some(pass(900)))]),
        on("Vulkan0")
    );
}

#[test]
fn automatic_takes_the_fastest_checked_card_with_room() {
    let (a, b, c) = (
        card("Vulkan0", 4000),
        card("Vulkan1", 7000),
        card("Vulkan2", 7500),
    );
    let three = [(&a, Some(pass(400))), (&b, Some(pass(900))), (&c, None)];
    assert_eq!(run(&DevicePick::Auto, &three), on("Vulkan1"));
    let tied = [(&a, Some(pass(900))), (&b, Some(pass(900)))];
    assert_eq!(
        run(&DevicePick::Auto, &tied),
        on("Vulkan1"),
        "a tie goes to the most free"
    );
    let full = card("Vulkan1", ROOM - 1);
    let busy = [(&a, Some(pass(400))), (&full, Some(pass(900)))];
    assert_eq!(
        run(&DevicePick::Auto, &busy),
        on("Vulkan0"),
        "the faster card has no room"
    );
}

#[test]
fn automatic_with_no_room_on_any_checked_card_uses_the_processor_and_says_why() {
    let (full, fuller) = (card("Vulkan0", ROOM - 1), card("Vulkan1", 100));
    let both = [(&fuller, Some(pass(900))), (&full, Some(pass(500)))];
    assert_eq!(
        run(&DevicePick::Auto, &both),
        crowded(ROOM - 1),
        "the nearest to fitting"
    );
    let edge = card("Vulkan0", ROOM);
    assert_eq!(
        run(&DevicePick::Auto, &[(&edge, Some(pass(900)))]),
        on("Vulkan0")
    );
}

#[test]
fn a_picked_card_serves_only_once_checked_present_and_with_room() {
    let pick = DevicePick::Card(CardId::new("Vulkan1").unwrap());
    let (other, it, full) = (
        card("Vulkan0", 8000),
        card("Vulkan1", 8000),
        card("Vulkan1", 100),
    );
    let differs = Verdict::Differs {
        words: 3,
        allowed: 1,
    };
    let cases = [
        ((&other, Some(pass(900))), Serve::Cpu(Why::Gone)),
        ((&it, None), Serve::Cpu(Why::Unchecked)),
        ((&it, Some(differs)), Serve::Cpu(Why::Failed(differs))),
        ((&it, Some(pass(605))), on("Vulkan1")),
        ((&full, Some(pass(605))), crowded(100)),
    ];
    for (one, want) in cases {
        assert_eq!(run(&pick, &[one]), want, "{one:?}");
    }
}

/// A check's numbers with these texts.
fn done(card_ms: u32, cpu_ms: u32, card_text: &str, cpu_text: &str) -> CheckDone {
    let secret = |t: &str| Redacted::new(Zeroizing::new(t.to_owned()));
    let (card_text, cpu_text) = (secret(card_text), secret(cpu_text));
    CheckDone {
        card_ms,
        cpu_ms,
        card_text,
        cpu_text,
    }
}

/// The verdict at `card_gain` 1.5 and `card_diff` `diff`.
fn judge(done: &CheckDone, lang: &str, diff: u8) -> Verdict {
    verdict(
        done,
        lang,
        &CardRules {
            gain: Hundredths(150),
            diff,
        },
    )
}

#[test]
fn a_check_passes_with_the_same_words_and_enough_speed() {
    let fr = "Bonjour, je dicte un texte.";
    assert_eq!(judge(&done(520, 3150, fr, fr), "fr", 1), pass(605));
    let near = done(520, 3150, "bonjour je dicte un texte", fr);
    assert_eq!(
        judge(&near, "fr", 1),
        pass(605),
        "case and punctuation are no difference"
    );
    let arabic = done(250, 2700, "ذهبَ الولدُ إلى المدرسةِ", "ذهب الولد إلى المدرسة");
    assert_eq!(
        judge(&arabic, "ar", 0),
        pass(1080),
        "harakat are no difference"
    );
    assert_eq!(
        judge(&done(0, 2700, fr, fr), "fr", 1),
        pass(u32::MAX),
        "an instant card"
    );
}

#[test]
fn a_check_fails_on_text_before_speed() {
    let set = "one two three four";
    let slow = Verdict::Slow {
        gain: Hundredths(125),
        need: Hundredths(150),
    };
    assert_eq!(judge(&done(2000, 2500, set, set), "en", 1), slow);
    let one = done(100, 2500, "one two tree four", set);
    assert_eq!(
        judge(&one, "en", 1),
        pass(2500),
        "one word apart is allowed"
    );
    assert_eq!(
        judge(&one, "en", 0),
        Verdict::Differs {
            words: 1,
            allowed: 0
        }
    );
    let two = done(2000, 2500, "one too tree four", set);
    assert_eq!(
        judge(&two, "en", 1),
        Verdict::Differs {
            words: 2,
            allowed: 1
        },
        "text first"
    );
}

#[test]
fn a_check_fails_on_a_wrong_arabic_letter_form() {
    let forms = done(250, 2700, "ذهب الولد إلى المدرسة", "ذهب الولد الى المدرسه");
    assert_eq!(
        judge(&forms, "ar", 1),
        Verdict::Differs {
            words: 2,
            allowed: 1
        },
        "a lost hamza and ه for ة are two words apart"
    );
}
