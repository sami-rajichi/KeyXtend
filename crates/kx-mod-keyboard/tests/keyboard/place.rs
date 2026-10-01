//! The place the user moves the keyboard to is saved through the module's own settings handle.

use crate::host::{FakeHost, fake, saved, start_settings};
use kx_mod_keyboard::Placer;
use kx_mod_keyboard::place::home;
use kx_module_api::WriteError;
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};

/// The size-1 board in logical pixels.
const PLATE: (f64, f64) = (670.78, 196.0);

fn screens() -> [Screen; 1] {
    [Screen {
        rect: Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1040,
        },
        dpi: BASE_DPI,
        primary: true,
    }]
}

fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

fn placer(host: &mut FakeHost) -> Placer {
    let settings = start_settings(host);
    Placer::new(&settings, host.handle())
}

/// The keys of each write so far, in order.
fn keys(host: &FakeHost) -> Vec<Vec<String>> {
    let writes = host.sink.writes().into_iter();
    writes
        .map(|(_, values)| values.keys().cloned().collect())
        .collect()
}

#[test]
fn it_starts_at_home_and_saves_nothing() {
    let mut host = fake(None);
    let mut p = placer(&mut host);
    assert_eq!(p.start(PLATE, &screens()), home(PLATE, &screens()));
    assert_eq!(p.at(), home(PLATE, &screens()));
    assert!(keys(&host).is_empty() && p.saved().is_none());
}

#[test]
fn a_move_saves_x_y_and_placed_in_one_write() {
    let mut host = fake(None);
    let mut p = placer(&mut host);
    assert_eq!(
        p.moved(at(300, 200), PLATE, &screens()),
        Ok(Some(at(300, 200)))
    );
    assert_eq!(keys(&host), [["placed", "x", "y"]]);
    assert_eq!(saved(&host).place(), Some(at(300, 200)));
    assert_eq!(
        p.moved(at(300, 200), PLATE, &screens()),
        Ok(Some(at(300, 200)))
    );
    assert_eq!(keys(&host).len(), 1, "the same place is not saved again");
}

#[test]
fn a_move_past_the_edge_is_saved_at_the_edge() {
    let mut host = fake(None);
    let mut p = placer(&mut host);
    assert_eq!(
        p.moved(at(1800, 200), PLATE, &screens()),
        Ok(Some(at(1249, 200)))
    );
    assert_eq!(saved(&host).place(), Some(at(1249, 200)));
}

#[test]
fn a_saved_place_is_where_it_starts() {
    let mut host = fake(Some(("placed", true.into())));
    let mut p = placer(&mut host);
    assert_eq!(p.saved(), Some(at(0, 0)));
    assert_eq!(p.start(PLATE, &screens()), Some(at(0, 0)));
}

#[test]
fn a_saved_place_off_every_screen_starts_home_and_is_kept_for_its_screen() {
    let mut host = fake(Some(("x", 9000.into())));
    let settings = {
        let mut s = start_settings(&host);
        s.placed = true;
        s
    };
    let mut p = Placer::new(&settings, host.handle());
    assert_eq!(p.start(PLATE, &screens()), home(PLATE, &screens()));
    assert_eq!(p.saved(), Some(at(9000, 0)), "the screen may come back");
    assert!(keys(&host).is_empty());
}

#[test]
fn an_unsaved_place_holds_in_the_store_which_saves_it_later() {
    let mut host = fake(None);
    let mut p = placer(&mut host);
    host.sink.refuse(Some(WriteError::Unsaved));
    let away = at(10, 20);
    assert_eq!(p.moved(away, PLATE, &screens()), Err(WriteError::Unsaved));
    assert_eq!((p.at(), p.saved()), (Some(away), Some(away)));
    assert_eq!(saved(&host).place(), Some(away), "the store holds it");
    host.sink.refuse(None);
    assert_eq!(p.moved(away, PLATE, &screens()), Ok(Some(away)));
    assert!(
        keys(&host).is_empty(),
        "the kernel saves it, not a second write"
    );
}

#[test]
fn after_an_unsaved_move_a_move_back_is_the_place_stop_saves() {
    let (mut host, first, away) = (fake(None), at(300, 200), at(10, 20));
    let mut p = placer(&mut host);
    assert_eq!(p.moved(first, PLATE, &screens()), Ok(Some(first)));
    host.sink.refuse(Some(WriteError::Unsaved));
    assert_eq!(p.moved(away, PLATE, &screens()), Err(WriteError::Unsaved));
    let back = p.moved(first, PLATE, &screens());
    assert_eq!(
        back,
        Err(WriteError::Unsaved),
        "the move back is written too"
    );
    assert_eq!(saved(&host).place(), Some(first), "the last place");
}

#[test]
fn a_place_that_fails_to_save_in_part_leaves_the_saved_place_whole() {
    let mut host = fake(Some(("placed", true.into())));
    let mut p = placer(&mut host);
    assert_eq!(
        p.moved(at(300, 200), PLATE, &screens()),
        Ok(Some(at(300, 200)))
    );
    let refused = WriteError::Refused(kx_mod_keyboard::ID);
    host.sink.refuse_key("y", refused.clone());
    let failed = p.moved(at(10, 20), PLATE, &screens());
    assert_eq!(failed, Err(refused));
    let kept = saved(&host).place();
    assert_eq!(
        kept,
        Some(at(300, 200)),
        "never x from one place and y from another"
    );
}

#[test]
fn with_no_screen_a_move_saves_nothing() {
    let mut host = fake(None);
    let mut p = placer(&mut host);
    assert_eq!(p.moved(at(10, 20), PLATE, &[]), Ok(None));
    assert_eq!(p.start(PLATE, &[]), None);
    assert!(keys(&host).is_empty());
}
