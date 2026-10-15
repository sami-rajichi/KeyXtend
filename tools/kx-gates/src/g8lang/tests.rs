//! Tests of `g8 arabic`: which layout it asks for, and that the owner's comes back however the run ends.

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_platform_fake::FakeLayoutSource;

use super::*;

/// The owner's layout, US English, and Arabic (102) AZERTY on Arabic (Tunisia), sign-extended.
const OWNER: LayoutId = LayoutId::new(0x0409_0409);
const AR: LayoutId = LayoutId::new(0xFFFF_FFFF_F029_1C01);

/// An app in front whose layout switches at once, unless the switch is to a layout it `refuses`.
struct Fake {
    now: Cell<Option<LayoutId>>,
    refuses: Vec<LayoutId>,
    asked: RefCell<Vec<LayoutId>>,
}

impl Fake {
    fn on(now: Option<LayoutId>, refuses: &[LayoutId]) -> Self {
        Self {
            now: Cell::new(now),
            refuses: refuses.to_vec(),
            asked: RefCell::new(Vec::new()),
        }
    }
}

impl Front for Fake {
    fn layout(&self) -> Option<LayoutId> {
        self.now.get()
    }

    fn switch(&self, id: LayoutId) -> bool {
        self.asked.borrow_mut().push(id);
        let shows = !self.refuses.contains(&id);
        if shows {
            self.now.set(Some(id));
        }
        shows
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
    let front = Fake::on(Some(OWNER), &[]);
    let e = within(&front, AR, || Err("no keyboard".into())).expect_err("broke");
    assert!(e.starts_with("no keyboard") && e.contains(BACK), "{e}");
    assert_eq!(front.now.get(), Some(OWNER));
    assert_eq!(*front.asked.borrow(), [AR, OWNER]);
}

#[test]
fn a_run_that_panics_still_puts_the_owners_layout_back() {
    let front = Fake::on(Some(OWNER), &[]);
    let run = || -> Result<Value, String> { panic!("a gate bug") };
    let caught = catch_unwind(AssertUnwindSafe(|| within(&front, AR, run)));
    assert!(caught.is_err(), "the panic goes on");
    assert_eq!(front.now.get(), Some(OWNER));
}

#[test]
fn a_run_that_held_names_its_layout_and_passes_only_when_it_held_and_the_owners_came_back() {
    let front = Fake::on(Some(OWNER), &[]);
    let v = within(&front, AR, || Ok(result(AR))).expect("ran");
    assert_eq!(v["layout"], json!(AR.to_string()));
    let flags = (&v["layout_held"], &v["layout_back"], &v["pass"]);
    assert_eq!(flags, (&json!(true), &json!(true), &json!(true)));
    let english = within(&front, AR, || Ok(result(OWNER))).expect("ran");
    assert_eq!(english["layout_held"], json!(false), "measured in English");
    assert_eq!(english["pass"], json!(false));
    let stuck = Fake::on(Some(OWNER), &[OWNER]);
    let v = within(&stuck, AR, || Ok(result(AR))).expect("ran");
    assert_eq!(
        (&v["layout_back"], &v["pass"]),
        (&json!(false), &json!(false))
    );
}

#[test]
fn a_layout_that_never_shows_stops_before_the_run_and_puts_the_owners_back() {
    let front = Fake::on(Some(OWNER), &[AR]);
    let never = || -> Result<Value, String> { panic!("ran with the owner's layout in front") };
    let e = within(&front, AR, never).expect_err("stopped");
    assert!(e.contains(&AR.to_string()) && e.contains(BACK), "{e}");
    assert_eq!(*front.asked.borrow(), [AR, OWNER]);
}

#[test]
fn with_no_app_in_front_nothing_is_asked_for() {
    let front = Fake::on(None, &[]);
    let e = within(&front, AR, || Ok(result(AR))).expect_err("nothing to ask");
    assert_eq!(e, NO_FRONT);
    assert!(front.asked.borrow().is_empty());
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
