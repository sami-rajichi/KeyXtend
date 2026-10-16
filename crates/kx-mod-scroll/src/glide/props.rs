//! Properties over any tick pattern: the lines owed add up to the speed times the time, with every gap capped.

use super::tests::EPS;
use super::*;
use crate::settings::{CHECK_MS, SPEED};
use kx_module_api::US_PER_MS;
use proptest::collection::vec;
use proptest::prelude::*;

/// Random runs per property.
const CASES: u32 = 512;
/// Most reckonings in one run.
const MAX_STEPS: usize = 80;
/// How many checks the longest gap tried spans, so many gaps are capped.
const LONG_GAP: u64 = 3;

/// One reckoning after a gap in µs.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// A tick.
    Tick(u64),
    /// A restart, fast or slow, for a tap that changes the speed or the way.
    Restart(u64, bool),
}

impl Step {
    fn gap(self) -> u64 {
        match self {
            Self::Tick(gap) | Self::Restart(gap, _) => gap,
        }
    }
}

/// `ms` in µs.
fn us(ms: u32) -> u64 {
    u64::from(ms) * u64::from(US_PER_MS)
}

/// The cap in µs.
fn cap_us(cfg: GlideCfg) -> u64 {
    u64::try_from(cfg.cap.as_micros()).unwrap()
}

/// Any speeds and check the settings allow.
fn cfg() -> impl Strategy<Value = GlideCfg> {
    (SPEED, SPEED, CHECK_MS).prop_map(|(slow, fast, check)| GlideCfg {
        slow,
        fast,
        cap: Duration::from_micros(us(check)),
    })
}

/// Where a run starts, in µs.
fn start() -> impl Strategy<Value = u64> {
    0..=u64::MAX / 2
}

/// Ticks and restarts after gaps up to `max_gap` µs.
fn run(max_gap: u64) -> impl Strategy<Value = Vec<Step>> {
    let gap = 0..=max_gap;
    let step = prop_oneof![
        4 => gap.clone().prop_map(Step::Tick),
        1 => (gap, any::<bool>()).prop_map(|(gap, fast)| Step::Restart(gap, fast)),
    ];
    vec(step, 0..=MAX_STEPS)
}

/// Lines at `speed` over `gap_us`, as the model counts them.
fn lines(speed: u32, gap_us: u64) -> f64 {
    f64::from(speed) * Duration::from_micros(gap_us).as_secs_f64()
}

/// Whether `got` lines are `want` lines, within float rounding.
fn same(got: f64, want: f64) -> bool {
    (got - want).abs() <= EPS * want.max(1.0)
}

/// Plays `steps` from `start`; returns each reckoning's lines with the speed it was owed at, then the speed after.
fn play(g: &mut Glide, cfg: GlideCfg, fast: bool, start: u64, steps: &[Step]) -> Vec<(f64, u32)> {
    let (mut now, mut speed) = (start, cfg.speed(fast));
    let mut out = Vec::with_capacity(steps.len());
    for &step in steps {
        now += step.gap();
        let at = Mono::from_us(now);
        let got = match step {
            Step::Tick(_) => g.tick(at),
            Step::Restart(_, fast) => g.restart(fast, at),
        };
        out.push((got, speed));
        if let Step::Restart(_, fast) = step {
            speed = cfg.speed(fast);
        }
    }
    out
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Over any ticks and restarts, the lines owed add up to each speed times its time with every gap capped at
    /// `check_ms`, so no time is lost or counted twice, and no reckoning owes more than one check's share.
    #[test]
    fn owed_adds_up_to_speed_times_capped_time(
        cfg in cfg(),
        fast in any::<bool>(),
        start in start(),
        steps in run(LONG_GAP * us(*CHECK_MS.end())),
    ) {
        let mut g = Glide::start(cfg, fast, Mono::from_us(start));
        let owed = play(&mut g, cfg, fast, start, &steps);
        let (mut got, mut want) = (0.0, 0.0);
        for (&step, &(lines_owed, speed)) in steps.iter().zip(&owed) {
            prop_assert!(lines_owed <= lines(speed, cap_us(cfg)) + EPS, "{lines_owed}");
            got += lines_owed;
            want += lines(speed, step.gap().min(cap_us(cfg)));
        }
        prop_assert!(same(got, want), "{got} != {want}");
    }

    /// With no gap past the check, after each tick the lines owed are the speed times the time since the start, so
    /// between ticks they lag by less than one tick's share.
    #[test]
    fn with_no_long_gap_owed_tracks_speed_times_time(
        cfg in cfg(),
        fast in any::<bool>(),
        start in start(),
        gaps in vec(0..=us(*CHECK_MS.start()), 0..=MAX_STEPS),
    ) {
        let mut g = Glide::start(cfg, fast, Mono::from_us(start));
        let (mut now, mut got) = (start, 0.0);
        for gap in gaps {
            now += gap;
            got += g.tick(Mono::from_us(now));
            let want = lines(cfg.speed(fast), now - start);
            prop_assert!(same(got, want), "{got} != {want}");
        }
    }

    /// For any values and times in any order, every reckoning owes a finite amount, never below 0, and the total
    /// never passes the top speed times the span from the start to the latest time.
    #[test]
    fn owed_is_never_negative_or_nan(
        slow in any::<u32>(),
        fast in any::<u32>(),
        cap in any::<u64>(),
        start in any::<u64>(),
        times in vec((any::<u64>(), any::<Option<bool>>()), 0..=MAX_STEPS),
    ) {
        let cfg = GlideCfg { slow, fast, cap: Duration::from_micros(cap) };
        let mut g = Glide::start(cfg, false, Mono::from_us(start));
        let (mut got, mut latest) = (0.0, start);
        for (time, restart) in times {
            let at = Mono::from_us(time);
            let owed = match restart {
                Some(fast) => g.restart(fast, at),
                None => g.tick(at),
            };
            prop_assert!(owed.is_finite() && owed >= 0.0, "{owed}");
            got += owed;
            latest = latest.max(time);
        }
        let most = lines(slow.max(fast), latest - start);
        prop_assert!(got <= most * (1.0 + EPS) + EPS, "{got} > {most}");
    }
}
