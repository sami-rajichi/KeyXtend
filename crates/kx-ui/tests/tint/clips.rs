//! The clipboard panel's picture provider serves the kept view's pixels at their own size, row by row, for its round
//! only, and one clear pixel for an old round, a row with no picture or a closed panel.

use kx_module_api::ports::{
    BASE_DPI, ClipKind, ClipPage, ClipPanel, ClipRow, ClipView, Date, Filter, Pages, Pixels, Point,
    Preview, Rect, Room, Screen, When,
};
use kx_module_api::{Bus, Redacted};
use kx_platform_fake::{
    FakeBackdrop, FakeMemory, FakeReveal, FakeScreens, FakeSpeaker, FakeSystemLook, FakeTray,
    FakeWindowGuard,
};
use kx_test_support::host::FakeCore;
use kx_ui::{Hub, UiPorts};
use std::sync::Arc;

/// The kept view's round.
const ROUND: u32 = 3;
/// A row's thumbnail: 2 by 1 pixels, red then green.
const THUMB: (u32, u32, [u8; 8]) = (2, 1, [255, 0, 0, 255, 0, 255, 0, 255]);
/// The preview: 1 by 2 pixels, blue over half-clear white.
const LARGE: (u32, u32, [u8; 8]) = (1, 2, [0, 0, 255, 255, 255, 255, 255, 128]);
/// One clear pixel, which QML draws as nothing.
const CLEAR: (u32, [u8; 4]) = (1, [0; 4]);

/// `p` as checked pixels.
fn pixels(p: (u32, u32, [u8; 8])) -> Result<Pixels, String> {
    Pixels::new(p.0, p.1, p.2.to_vec(), p.0.max(p.1)).ok_or_else(|| "bad test pixels".to_owned())
}

/// A hub over fake ports on one plain screen.
fn hub() -> Result<Arc<Hub>, String> {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    let screen = Screen {
        rect,
        dpi: BASE_DPI,
        primary: true,
    };
    let ports = UiPorts {
        guard: Arc::new(FakeWindowGuard::new(0)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look: Arc::new(FakeSystemLook::new()),
        backdrop: Arc::new(FakeBackdrop::new()),
        tray: Arc::new(FakeTray::new(1)),
        memory: Arc::new(FakeMemory::new()),
        speaker: Arc::new(FakeSpeaker::new()),
        reveal: Arc::new(FakeReveal::new()),
    };
    Ok(Hub::new(ports).map_err(|e| e.to_string())?.0)
}

/// Image row `id` copied today, with `thumb`.
fn row(id: u64, thumb: Option<Pixels>) -> Result<ClipRow, String> {
    let date = Date::new(2026, 10, 31).ok_or("bad test date")?;
    Ok(ClipRow {
        id,
        kind: ClipKind::Image,
        title: Redacted::new(String::new()),
        rtl: false,
        arabic: false,
        app: None,
        when: When {
            days_ago: 0,
            date,
            hour: 0,
            minute: 0,
        },
        bytes: Some(1),
        tick: None,
        pinned: false,
        expires: false,
        thumb,
    })
}

/// Round `ROUND` with row 7 thumbed, row 8 not, and the preview image.
fn page() -> Result<ClipPage, String> {
    Ok(ClipPage {
        round: ROUND,
        rows: vec![row(7, Some(pixels(THUMB)?))?, row(8, None)?],
        highlight: None,
        filter: Filter::All,
        search: Redacted::new(String::new()),
        pages: Pages::default(),
        ticked: 0,
        unpin: false,
        preview: Some(Preview::Image(Some(pixels(LARGE)?))),
        confirm: None,
        unpinned: 2,
        room: Some(Room {
            used: 0,
            free: 0,
            drive: String::new(),
        }),
        cap: 1,
        note: None,
        kept_days: 1,
        empty: false,
    })
}

/// Checks that `id` serves `width` and `rgba`.
fn serves(id: &str, width: u32, rgba: &[u8]) -> Result<(), String> {
    let got = kx_ui::clip_pixels(id);
    if got.0 == width && got.1 == rgba {
        return Ok(());
    }
    Err(format!("{id} served {got:?}, not {width} wide {rgba:?}"))
}

/// The provider copies the kept round's pictures and nothing else; it installs the binary's one hub.
pub fn serves_the_kept_round_only() -> Result<(), String> {
    let hub = hub()?;
    if !kx_ui::install(Arc::clone(&hub)) {
        return Err("a hub was installed already".to_owned());
    }
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).map_err(|e| e.to_string())?;
    serves(&format!("{ROUND}/7"), CLEAR.0, &CLEAR.1)?;
    bus.publish(&ClipPanel { open: true })
        .map_err(|e| e.to_string())?;
    bus.publish(&ClipView::new(page()?))
        .map_err(|e| e.to_string())?;
    serves(&format!("{ROUND}/7"), THUMB.0, &THUMB.2)?;
    serves(&format!("{ROUND}/preview"), LARGE.0, &LARGE.2)?;
    for gone in [
        format!("{}/7", ROUND - 1),
        format!("{ROUND}/8"),
        "7".to_owned(),
    ] {
        serves(&gone, CLEAR.0, &CLEAR.1)?;
    }
    bus.publish(&ClipPanel { open: false })
        .map_err(|e| e.to_string())?;
    serves(&format!("{ROUND}/7"), CLEAR.0, &CLEAR.1)
}
