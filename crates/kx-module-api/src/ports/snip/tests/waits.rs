//! The jobs' waits and the pick job's limits: each value is refused past its ends, in both asks that carry the waits,
//! and the first restart wait never passes the longest.

use super::record::waits;
use crate::ports::Rect;
use crate::ports::snip::{AskError, FileKind, PickLimits, RecordAsk, RecordError, RecordWaits};
use crate::ports::snip::{Part, SnipFile, Speed, SpeedAsk};
use std::ops::RangeInclusive;
use std::path::PathBuf;

/// A value's name, its range, and how to set it.
type Field<T, V> = (&'static str, RangeInclusive<V>, fn(&mut T, V));

/// Each wait.
fn wait_fields() -> [Field<RecordWaits, u32>; 6] {
    [
        ("start_ms", RecordWaits::START_MS, |w, v| w.start_ms = v),
        ("tick_ms", RecordWaits::TICK_MS, |w, v| w.tick_ms = v),
        ("end_ms", RecordWaits::END_MS, |w, v| w.end_ms = v),
        ("switch_ms", RecordWaits::SWITCH_MS, |w, v| w.switch_ms = v),
        ("speed_ms", RecordWaits::SPEED_MS, |w, v| w.speed_ms = v),
        ("speed_mb_ms", RecordWaits::SPEED_MB_MS, |w, v| {
            w.speed_mb_ms = v;
        }),
    ]
}

/// Each pick limit counted in `u32`; `dialog_min` and `idle_s` are `u16` and have their own rows.
fn limit_fields() -> [Field<PickLimits, u32>; 3] {
    [
        ("job_mb", PickLimits::JOB_MB, |l, v| l.job_mb = v),
        ("restart_ms", PickLimits::RESTART_MS, |l, v| {
            l.restart_ms = v;
        }),
        ("restart_max_ms", PickLimits::RESTART_MAX_MS, |l, v| {
            l.restart_max_ms = v;
        }),
    ]
}

/// The short pick limits.
fn short_fields() -> [Field<PickLimits, u16>; 2] {
    [
        ("dialog_min", PickLimits::DIALOG_MIN, |l, v| {
            l.dialog_min = v;
        }),
        ("idle_s", PickLimits::IDLE_S, |l, v| l.idle_s = v),
    ]
}

/// Each pick limit at the start of its range.
const fn limits() -> PickLimits {
    PickLimits {
        job_mb: *PickLimits::JOB_MB.start(),
        dialog_min: *PickLimits::DIALOG_MIN.start(),
        idle_s: *PickLimits::IDLE_S.start(),
        restart_ms: *PickLimits::RESTART_MS.start(),
        restart_max_ms: *PickLimits::RESTART_MAX_MS.start(),
    }
}

/// Each pick limit at the end of its range.
const fn longest() -> PickLimits {
    PickLimits {
        job_mb: *PickLimits::JOB_MB.end(),
        dialog_min: *PickLimits::DIALOG_MIN.end(),
        idle_s: *PickLimits::IDLE_S.end(),
        restart_ms: *PickLimits::RESTART_MS.end(),
        restart_max_ms: *PickLimits::RESTART_MAX_MS.end(),
    }
}

/// `base` with `set` putting `at` in it.
fn with<T: Copy, V>(base: T, set: fn(&mut T, V), at: V) -> T {
    let mut changed = base;
    set(&mut changed, at);
    changed
}

/// The two asks carrying `waits`.
fn asks(waits: RecordWaits) -> (RecordAsk, SpeedAsk) {
    let square = Rect {
        x: 0,
        y: 0,
        w: 64,
        h: 64,
    };
    let record = RecordAsk {
        part: Part::new(square).unwrap(),
        fps: RecordAsk::FPS[0],
        mic: None,
        sound: false,
        max_min: *RecordAsk::MAX_MIN.start(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits,
    };
    let file = SnipFile::new(PathBuf::from(r"C:\Snips\unsaved\a.mp4"), FileKind::Mp4);
    let speed = SpeedAsk {
        from: file.unwrap(),
        speed: Speed::new(SpeedAsk::SPEEDS[0]).unwrap(),
        min_free_mb: record.min_free_mb,
        job_mb: record.job_mb,
        waits,
    };
    (record, speed)
}

#[test]
fn a_wait_past_its_range_is_refused_by_both_asks() {
    for (name, range, set) in wait_fields() {
        for at in [range.start() - 1, range.end() + 1] {
            let (record, speed) = asks(with(waits(), set, at));
            assert_eq!(record.check(), Err(RecordError::Range(name)), "{at}");
            assert_eq!(speed.check(), Err(RecordError::Range(name)), "{at}");
        }
        let (record, speed) = asks(with(waits(), set, *range.end()));
        assert_eq!((record.check(), speed.check()), (Ok(()), Ok(())));
    }
}

#[test]
fn pick_limits_take_their_ends_and_refuse_past_them() {
    assert_eq!((limits().check(), longest().check()), (Ok(()), Ok(())));
    let refused = |l: PickLimits, name| assert_eq!(l.check(), Err(AskError::Range(name)));
    for (name, range, set) in limit_fields() {
        refused(with(limits(), set, range.start() - 1), name);
        refused(with(longest(), set, range.end() + 1), name);
    }
    for (name, range, set) in short_fields() {
        refused(with(limits(), set, range.start() - 1), name);
        refused(with(longest(), set, range.end() + 1), name);
    }
}

#[test]
fn a_first_restart_wait_past_the_longest_is_refused() {
    let most = *PickLimits::RESTART_MAX_MS.start();
    let order = PickLimits {
        restart_ms: most + 1,
        restart_max_ms: most,
        ..limits()
    };
    assert_eq!(order.check(), Err(AskError::Range("restart_ms")));
    let even = PickLimits {
        restart_ms: most,
        ..order
    };
    assert_eq!(even.check(), Ok(()));
}
