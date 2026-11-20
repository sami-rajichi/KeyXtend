//! A pick that left the service on the bar (final review m4, voice-a M10): it moved, to another service or to the local
//! model, while that service stays on, so the bar never says it is off; a new model of the same service is the one sent,
//! counted and kept.

use super::*;

/// Groq's other model, as the same service's new pick.
const OTHER: &str = "groq:other-model";

#[test]
fn a_pick_moved_to_another_service_says_it_moved_while_the_first_stays_on() {
    let mut rig = Rig::cloud();
    rig.book.2.insert("groq".into());
    rig.listen();
    rig.book.1.insert(
        "fr".into(),
        Net {
            service: "custom".into(),
            ..groq()
        },
    );
    assert!(uploads(&rig.hand_over()).is_empty());
    assert_eq!(
        rig.shown(),
        cloud_note(CloudWhy::Moved, LocalUse::Ready, true)
    );
}

#[test]
fn a_pick_moved_to_the_local_model_says_it_moved_and_a_service_switched_off_says_off() {
    for (on, why) in [(true, CloudWhy::Moved), (false, CloudWhy::Off)] {
        let mut rig = Rig::cloud();
        rig.book.2.extend(on.then(|| "groq".to_owned()));
        rig.listen();
        rig.book.1.clear();
        let settings = Box::new(VoiceSettings::from_defaults().unwrap());
        assert!(uploads(&rig.on(In::Settings(settings))).is_empty());
        assert_eq!(rig.shown(), cloud_note(why, LocalUse::Ready, true), "{on}");
    }
}

#[test]
fn transcribe_anyway_after_the_same_services_model_changed_sends_counts_and_keeps_the_new_one() {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.hand_over();
    rig.bar(BarButton::Cancel);
    let other = Net {
        model: Redacted::new(OTHER.into()),
        ..groq()
    };
    rig.book.1.insert("fr".into(), other);
    assert_eq!(uploads(&rig.bar(BarButton::Anyway)).len(), 1);
    let out = rig.reply(&["bonjour"]);
    let count = out.iter().find_map(|o| match o {
        Out::Count(count) => Some(count.model.expose().clone()),
        _ => None,
    });
    let kept = out.iter().find_map(|o| match o {
        Out::Keep(kept) => Some(kept.model.expose().clone()),
        _ => None,
    });
    assert_eq!(
        (count.as_deref(), kept.as_deref()),
        (Some(OTHER), Some(OTHER))
    );
}
