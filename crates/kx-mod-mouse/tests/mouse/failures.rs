//! What goes wrong: a hook the OS refuses, a setting that cannot be saved, a stop mid-carry, and the log lines.

use crate::logs::{MESSAGE, lines_of};
use crate::rig::{A, B, Heard, Rig, T0, T1, down, moved, shown};
use kx_mod_mouse::{ID, Msg};
use kx_module_api::ports::GrabState::Off;
use kx_module_api::ports::{AssistChanged, Button, HookError, HookStats, PointerAct, Toggle};
use kx_module_api::{Notice, NoticeArg, WriteError, args, keys};
use tracing::Level;

/// The OS error code a refused hook carries.
const CODE: i32 = 1428;
/// The message of the line that logs a stopped hook's numbers.
const STOPPED: &str = "mouse hook stopped";
/// Stats whose numbers appear nowhere else.
const STATS: HookStats = HookStats {
    events: 101,
    p99_us: 202,
    max_us: 303,
    max_wait_ms: 404,
    reinstalls: 5,
    send_errors: 6,
    lost_echoes: 7,
};

/// The notice a refused hook publishes.
fn hook_failed() -> Heard {
    Heard::Notice(Notice {
        key: keys::MOUSE_HOOK_FAILED,
        module: Some(ID),
        args: vec![(args::MODULE, NoticeArg::Text(ID.to_string()))],
    })
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
fn a_saved_right_click_the_hook_refuses_starts_off() {
    let mut rig = Rig::saved_rclick();
    rig.hook.fail_next(HookError::Thread);
    rig.begin();
    let told = [hook_failed(), shown(false, Off), Heard::AskLatches];
    assert_eq!(rig.take(), told);
    assert!(
        rig.host.sink.writes().is_empty(),
        "the saved value waits for the next start"
    );
}

#[test]
fn a_right_click_that_cannot_be_saved_tells_settings_unsaved() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.host.sink.refuse(Some(WriteError::Unsaved));
    rig.toggle(Toggle::RightClick);
    rig.settle(T0);
    let unsaved = Notice {
        key: keys::SETTINGS_UNSAVED,
        module: None,
        args: Vec::new(),
    };
    assert_eq!(rig.take(), [shown(true, Off), Heard::Notice(unsaved)]);
}

#[test]
fn stopping_during_a_carry_lets_go_of_the_button() {
    let mut rig = Rig::new();
    rig.begin();
    rig.toggle(Toggle::Grab);
    rig.settle(T0);
    let _ = rig.feed(down(A), T1);
    rig.take();
    let _ = rig.hook.take_sent();
    assert!(!rig.send(Msg::Stop), "the actor is done");
    assert!(!rig.hook.running());
    assert_eq!(rig.hook.take_sent(), [PointerAct::Up(Button::Left, A)]);
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
fn the_stopped_line_logs_the_hook_numbers_and_no_point() {
    let lines = lines_of(|| {
        let mut rig = Rig::saved_rclick();
        rig.hook.set_stats(STATS);
        rig.begin();
        let _ = rig.feed(moved(B), T0);
        let _ = rig.feed(down(A), T1);
        rig.send(Msg::Stop);
    });
    let stopped: Vec<_> = lines
        .iter()
        .filter(|(_, f)| f[MESSAGE] == STOPPED)
        .collect();
    assert_eq!(stopped.len(), 1, "{lines:?}");
    let (level, fields) = stopped[0];
    assert_eq!(*level, Level::INFO);
    let numbers = [
        ("events", STATS.events),
        ("p99_us", STATS.p99_us.into()),
        ("max_us", STATS.max_us.into()),
        ("max_wait_ms", STATS.max_wait_ms.into()),
        ("reinstalls", STATS.reinstalls.into()),
        ("send_errors", STATS.send_errors.into()),
        ("lost_echoes", STATS.lost_echoes.into()),
    ];
    let mut want: Vec<_> = numbers.map(|(k, v)| (k, v.to_string())).to_vec();
    want.push((MESSAGE, STOPPED.to_owned()));
    want.sort_unstable();
    let got: Vec<_> = fields.iter().map(|(k, v)| (*k, v.clone())).collect();
    assert_eq!(got, want);
    let points = [A.x, A.y, B.x, B.y].map(|n| n.to_string());
    for line in lines.iter().flat_map(|(_, f)| f.values()) {
        assert!(points.iter().all(|p| !line.contains(p.as_str())), "{line}");
    }
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
