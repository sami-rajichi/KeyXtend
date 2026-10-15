//! Tests of `g8 arabic`: which layout it asks for, that it reads and holds layouts only through its own windows, and
//! that the owner's comes back however the run ends.

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_platform_fake::FakeLayoutSource;

use super::*;

/// The owner's layout, US English, and Arabic (102) AZERTY on Arabic (Tunisia), sign-extended.
const OWNER: LayoutId = LayoutId::new(0x0409_0409);
const AR: LayoutId = LayoutId::new(0xFFFF_FFFF_F029_1C01);
/// Why the fake switch failed.
const REFUSED: &str = "it never typed in it";

/// A session whose layout each target-window types in, numbered as they open; the app in front lags at `OWNER`.
struct World {
    session: Cell<Option<LayoutId>>,
    refuses: Vec<LayoutId>,
    taps: RefCell<Vec<Tap>>,
    asked: RefCell<Vec<(usize, LayoutId)>>,
    closed: RefCell<Vec<usize>>,
}

impl World {
    fn on(session: Option<LayoutId>, refuses: &[LayoutId]) -> Self {
        Self {
            session: Cell::new(session),
            refuses: refuses.to_vec(),
            taps: RefCell::default(),
            asked: RefCell::default(),
            closed: RefCell::default(),
        }
    }
}

impl Own for World {
    type Window = usize;

    fn open(&self, tap: Tap) -> Result<usize, String> {
        self.taps.borrow_mut().push(tap);
        Ok(self.taps.borrow().len())
    }

    fn layout(&self, _: &usize) -> Option<LayoutId> {
        self.session.get()
    }

    fn ask(&self, w: &usize, id: LayoutId) -> Result<(), String> {
        self.asked.borrow_mut().push((*w, id));
        if self.refuses.contains(&id) {
            return Err(REFUSED.into());
        }
        self.session.set(Some(id));
        Ok(())
    }

    fn close(&self, w: usize) {
        self.closed.borrow_mut().push(w);
    }

    fn in_front(&self) -> Option<LayoutId> {
        Some(OWNER)
    }
}

/// G8's result line whose reading after use saw `layout` in front.
fn result(layout: LayoutId) -> Value {
    json!({ "gate": "G8", "use": { "layout": layout.to_string() }, "pass": true })
}

fn shipped() -> G8 {
    crate::config::load().expect("kx-gates.toml loads").g8
}

#[test]
fn a_run_that_breaks_puts_the_owners_layout_back_and_says_so() {
    let world = World::on(Some(OWNER), &[]);
    let e = within(&world, AR, |_| Err("no keyboard".into())).expect_err("broke");
    assert!(e.starts_with("no keyboard") && e.contains(BACK), "{e}");
    assert_eq!(world.session.get(), Some(OWNER));
    assert_eq!(*world.asked.borrow(), [(1, AR), (2, OWNER)]);
}

#[test]
fn a_run_that_panics_still_puts_the_owners_layout_back() {
    let world = World::on(Some(OWNER), &[]);
    let run =
        |_: &dyn Fn() -> Result<(), String>| -> Result<Value, String> { panic!("a gate bug") };
    let caught = catch_unwind(AssertUnwindSafe(|| within(&world, AR, run)));
    assert!(caught.is_err(), "the panic goes on");
    assert_eq!(world.session.get(), Some(OWNER));
    assert_eq!(*world.taps.borrow(), [Tap::Plain, Tap::PutBack]);
}

#[test]
fn only_the_put_back_brings_its_window_forward_with_the_tap_that_still_goes_after_a_stop() {
    let world = World::on(Some(OWNER), &[]);
    let v = within(&world, AR, |hold| hold().map(|()| result(AR))).expect("ran");
    assert_eq!(v["layout_back"], json!(true));
    let taps = [Tap::Plain, Tap::Plain, Tap::PutBack];
    assert_eq!(*world.taps.borrow(), taps, "switch, hold, put-back");
    let refused = World::on(Some(OWNER), &[AR]);
    let never = |_: &dyn Fn() -> Result<(), String>| -> Result<Value, String> { panic!("ran") };
    within(&refused, AR, never).expect_err("stopped");
    assert_eq!(*refused.taps.borrow(), [Tap::Plain, Tap::PutBack]);
}

#[test]
fn the_layout_held_only_when_the_reading_and_g8s_own_window_at_the_end_both_typed_in_it() {
    let world = World::on(Some(OWNER), &[]);
    let v = within(&world, AR, |_| Ok(result(AR))).expect("ran");
    let flags = (&v["layout_held"], &v["layout_back"], &v["pass"]);
    assert_eq!(
        flags,
        (&json!(true), &json!(true), &json!(true)),
        "the app in front lagged"
    );
    assert_eq!(v["layout_end"], json!(AR.to_string()));
    let lost = World::on(Some(OWNER), &[]);
    let v = within(&lost, AR, |_| {
        lost.session.set(Some(OWNER));
        Ok(result(AR))
    })
    .expect("ran");
    let end = (&v["layout_held"], &v["layout_end"], &v["pass"]);
    assert_eq!(
        end,
        (&json!(false), &json!(OWNER.to_string()), &json!(false))
    );
    let english = within(&World::on(Some(OWNER), &[]), AR, |_| Ok(result(OWNER)));
    assert_eq!(english.expect("ran")["layout_held"], json!(false));
}

#[test]
fn a_put_back_that_does_not_take_fails_the_run() {
    let stuck = World::on(Some(OWNER), &[OWNER]);
    let v = within(&stuck, AR, |_| Ok(result(AR))).expect("ran");
    let flags = (&v["layout_back"], &v["pass"]);
    assert_eq!(flags, (&json!(false), &json!(false)));
}

#[test]
fn a_layout_that_never_shows_stops_before_the_run_and_puts_the_owners_back() {
    let world = World::on(Some(OWNER), &[AR]);
    let never = |_: &dyn Fn() -> Result<(), String>| -> Result<Value, String> {
        panic!("ran with the owner's layout in front")
    };
    let e = within(&world, AR, never).expect_err("stopped");
    assert!(e.contains(&AR.to_string()) && e.contains(BACK), "{e}");
    assert!(e.contains(REFUSED), "says why: {e}");
    assert_eq!(*world.asked.borrow(), [(1, AR), (2, OWNER)]);
    assert_eq!(*world.closed.borrow(), [1, 2]);
}

#[test]
fn a_window_that_reads_no_layout_changes_nothing() {
    let world = World::on(None, &[]);
    let e = within(&world, AR, |_| Ok(result(AR))).expect_err("nothing to read");
    assert_eq!(e, NO_LAYOUT);
    assert!(world.asked.borrow().is_empty());
    assert_eq!(*world.closed.borrow(), [1]);
}

#[test]
fn the_switchs_window_stays_for_the_cold_start_and_each_hold_names_the_layout_it_found_in_front() {
    let world = World::on(Some(OWNER), &[]);
    let v = within(&world, AR, |hold| {
        assert!(world.closed.borrow().is_empty(), "still in front");
        hold()?;
        hold()?;
        Ok(result(AR))
    })
    .expect("ran");
    let lagged = OWNER.to_string();
    assert_eq!(v["layout_behind"], json!([lagged, lagged]));
    assert_eq!(*world.closed.borrow(), [4], "only the put-back's");
}

#[test]
fn a_hold_that_finds_the_session_out_of_the_layout_fails_and_closes_its_window() {
    let world = World::on(Some(OWNER), &[]);
    let e = hold(&world, AR).expect_err("the session lost it");
    assert_eq!(
        e,
        format!("the session types in layout {OWNER} instead of {AR}")
    );
    assert_eq!(*world.closed.borrow(), [1]);
    world.session.set(Some(AR));
    assert_eq!(
        hold(&world, AR),
        Ok(Some(OWNER)),
        "what the app in front showed"
    );
    assert_eq!(*world.closed.borrow(), [1], "kept in front");
}

#[test]
fn only_g8_arabic_asks_for_a_layout_and_a_missing_one_stops_it_before_any_change() {
    let source = FakeLayoutSource::golden(&["example"]).expect("the example table");
    let mut g = shipped();
    assert_eq!(wanted(&source, &g, crate::product::NAME), Ok(None));
    let e = wanted(&source, &g, ARABIC).expect_err("the example is French");
    assert!(
        e.contains(&g.arabic) && e.contains("changed nothing"),
        "{e}"
    );
    g.arabic = "fr".into();
    let french = wanted(&source, &g, ARABIC).expect("found");
    assert!(french.is_some_and(|id| source.installed().contains(&id)));
}

#[test]
fn the_shipped_arabic_run_names_arabic_and_waits_for_its_switch() {
    let g = shipped();
    assert_eq!((g.arabic.as_str(), check(&g)), ("ar", Ok(())));
    for blank in [
        G8 {
            arabic: String::new(),
            ..g.clone()
        },
        G8 {
            layout_wait_ms: 0,
            ..g.clone()
        },
    ] {
        assert!(check(&blank).is_err() && blank.check().is_err());
    }
}
