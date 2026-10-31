use kx_test_support::tempdir::TempDir;

use super::*;
use crate::config;
use crate::ownkeys::fake::{Fake, OUR_TAB, OWNER_TAB, hwnd, notepad};

/// Runs `test` with the shipped settings, a fresh out folder and the gate's Notepad, whose file is in it.
fn with_ctx(test: impl FnOnce(&Ctx, &Opened)) {
    let cfg = config::load().expect("settings load");
    let spike = spike_core::config::load().expect("spike settings");
    let target = kx_target_window::config::load().expect("target settings");
    let dir = TempDir::new("gates-readback").expect("temp folder");
    let (out, base) = (dir.path().to_path_buf(), "t".to_string());
    let app = notepad(Some(out.join("g20-keyxtend-1.txt")));
    let (cfg, spike, target) = (&cfg, &spike, &target);
    let ctx = Ctx {
        cfg,
        spike,
        target,
        out,
        base,
    };
    test(&ctx, &app);
}

#[test]
fn the_read_back_never_saves_the_owners_tab() {
    with_ctx(|ctx, app| {
        let desk = Fake::new(app.hwnd, &[OWNER_TAB]);
        let e = read_on(ctx, &desk, app).expect_err("the owner's tab");
        assert!(e.contains("our tab is not shown"), "{e}");
        assert!(saved_on(ctx, &desk, app, |_| true).is_err());
        assert!(desk.quiet(), "no Ctrl+S went out");
    });
}

#[test]
fn the_read_back_sends_nothing_while_another_window_is_in_front() {
    with_ctx(|ctx, app| {
        let desk = Fake::new(hwnd(2), &[OUR_TAB]);
        assert!(read_on(ctx, &desk, app).is_err());
        assert!(desk.quiet());
    });
}
