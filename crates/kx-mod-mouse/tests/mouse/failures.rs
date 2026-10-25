//! What goes wrong: a hook the OS refuses, a setting that cannot be saved, a stop mid-carry, and the warnings; an
//! engine the hook gave up is in `reset`, and the stop line in `stopped`.

use crate::logs::{Fields, MESSAGE, lines_of};
use crate::rig::{A, CODE, Heard, Rig, SHIFT, T0, T1, down, shown, told};
use kx_mod_mouse::{ESC, ID, Msg};
use kx_module_api::ports::GrabState::Off;
use kx_module_api::ports::{AssistChanged, Button, HookError, PointerAct, Toggle};
use kx_module_api::{SettingsError, WriteError, keys};
use tracing::Level;

/// The field that names a refused write's kind.
const KIND: &str = "kind";
/// A value a refusal's text quotes, which no log line may show.
const QUOTED: &str = "rclick = 4321";

/// How many warnings `lines` holds.
fn warnings(lines: &[(Level, Fields)]) -> usize {
    lines.iter().filter(|(l, _)| *l == Level::WARN).count()
}

/// The notice a refused hook publishes.
fn hook_failed() -> Heard {
    told(keys::MOUSE_HOOK_FAILED)
}

#[test]
fn a_refused_hook_tells_the_user_keeps_the_keys_off_and_the_next_toggle_tries_again() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.hook.fail_next(HookError::Refused(CODE));
    rig.toggle(Toggle::RightClick);
    assert!(!rig.hook.running());
    assert_eq!(rig.take(), [hook_failed()]);
    rig.send(Msg::Ask);
    assert_eq!(rig.take(), [shown(false, Off)]);
    rig.toggle(Toggle::RightClick);
    assert!(rig.hook.running(), "the next toggle tries again");
}

#[test]
fn a_saved_right_click_the_hook_refuses_starts_off_and_says_why() {
    let lines = lines_of(|| {
        let mut rig = Rig::saved_rclick();
        rig.hook.fail_next(HookError::Thread);
        rig.begin();
        let told = [
            hook_failed(),
            shown(false, Off),
            Heard::AskLatches,
            Heard::AskScroll,
            Heard::AskPresses,
        ];
        assert_eq!(rig.take(), told, "the user expects Right-click on");
        assert!(
            rig.host.sink.writes().is_empty(),
            "the saved value waits for the next start"
        );
    });
    assert_eq!(warnings(&lines), 1);
}

#[test]
fn a_hook_refused_for_latches_tells_no_one_and_warns_once_per_streak() {
    let lines = lines_of(|| {
        let mut rig = Rig::new();
        rig.begin();
        rig.take();
        for _ in 0..2 {
            rig.hook.fail_next(HookError::Refused(CODE));
            rig.send(Msg::Latches(vec![SHIFT]));
        }
        assert!(rig.take().is_empty(), "the user pressed no mouse key");
        rig.send(Msg::Latches(vec![SHIFT]));
        assert!(rig.hook.running(), "a start that works ends the streak");
        rig.send(Msg::Latches(Vec::new()));
        rig.settle(T0);
        rig.hook.fail_next(HookError::Refused(CODE));
        rig.send(Msg::Latches(vec![SHIFT]));
        assert!(rig.take().is_empty());
    });
    assert_eq!(warnings(&lines), 2, "one per streak: {lines:?}");
}

#[test]
fn a_mouse_key_refused_after_a_latch_failure_still_tells_the_user() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.hook.fail_next(HookError::Refused(CODE));
    rig.send(Msg::Latches(vec![SHIFT]));
    rig.hook.fail_next(HookError::Refused(CODE));
    rig.toggle(Toggle::Grab);
    assert_eq!(rig.take(), [hook_failed()]);
}

#[test]
fn a_right_click_that_cannot_be_saved_still_works_and_adds_no_notice() {
    let lines = lines_of(|| {
        let mut rig = Rig::new();
        rig.begin();
        rig.take();
        rig.host.sink.refuse(Some(WriteError::Unsaved));
        rig.toggle(Toggle::RightClick);
        rig.settle(T0);
        // The kernel's store tells the user once per run that settings are not saved.
        assert_eq!(rig.take(), [shown(true, Off)]);
        rig.toggle(Toggle::RightClick);
        rig.settle(T1);
        assert_eq!(rig.take(), [shown(false, Off)]);
        assert!(!rig.hook.running(), "the toggle works in memory");
    });
    assert_eq!(warnings(&lines), 0, "the kernel already said it: {lines:?}");
}

#[test]
fn a_refused_right_click_write_logs_only_its_kind_and_tells_no_one() {
    let invalid = WriteError::Invalid(SettingsError::Invalid(QUOTED.to_owned()));
    for error in [WriteError::Refused(ID), invalid] {
        let kind = error.kind();
        let lines = lines_of(|| {
            let mut rig = Rig::new();
            rig.begin();
            rig.take();
            rig.host.sink.refuse(Some(error.clone()));
            rig.toggle(Toggle::RightClick);
            rig.settle(T0);
            assert_eq!(rig.take(), [shown(true, Off)], "no notice");
            assert!(rig.hook.running(), "the toggle works in memory");
        });
        let warned: Vec<_> = lines.iter().filter(|(l, _)| *l == Level::WARN).collect();
        assert_eq!(warned.len(), 1, "{lines:?}");
        let fields = &warned[0].1;
        assert_eq!(fields.get(KIND).map(String::as_str), Some(kind));
        let names: Vec<_> = fields.keys().copied().collect();
        assert_eq!(names, [KIND, MESSAGE], "only the kind");
        assert!(fields.values().all(|v| !v.contains(QUOTED)), "{fields:?}");
    }
}

#[test]
fn stopping_during_a_carry_cancels_it_with_esc_and_lets_go_of_the_button() {
    let mut rig = Rig::new();
    rig.begin();
    rig.toggle(Toggle::Grab);
    rig.settle(T0);
    let _ = rig.feed(down(A), T1);
    rig.take();
    let _ = rig.hook.take_sent();
    assert!(!rig.send(Msg::Stop), "the actor is done");
    assert!(!rig.hook.running());
    let cancel = [
        PointerAct::KeyDown(ESC),
        PointerAct::KeyUp(ESC),
        PointerAct::Up(Button::Left, A),
    ];
    assert_eq!(rig.hook.take_sent(), cancel);
    let off = AssistChanged {
        live: false,
        rclick: false,
        grab: Off,
    };
    assert_eq!(rig.take(), [Heard::Changed(off)]);
}

#[test]
fn a_stop_keeps_the_saved_right_click_in_its_last_word() {
    let mut rig = Rig::saved_rclick();
    rig.begin();
    rig.toggle(Toggle::Grab);
    rig.settle(T0);
    rig.take();
    rig.send(Msg::Stop);
    let off = AssistChanged {
        live: false,
        rclick: true,
        grab: Off,
    };
    assert_eq!(rig.take(), [Heard::Changed(off)]);
}

#[test]
fn a_refused_hook_logs_one_warning_with_its_error() {
    let lines = lines_of(|| {
        let mut rig = Rig::new();
        rig.begin();
        rig.hook.fail_next(HookError::Refused(CODE));
        rig.toggle(Toggle::RightClick);
    });
    let warned: Vec<_> = lines.iter().filter(|(l, _)| *l == Level::WARN).collect();
    assert_eq!(warned.len(), 1, "{lines:?}");
    let error = HookError::Refused(CODE).to_string();
    assert!(warned[0].1.values().any(|v| *v == error), "{warned:?}");
}
