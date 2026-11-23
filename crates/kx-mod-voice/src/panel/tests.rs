//! The panel (plan task 19): the Models page with its folder line, Change and Open; tick, then act; a download's
//! progress, Pause and Cancel on its row; and a bad fingerprint's note.

use std::path::Path;

use kx_module_api::ports::{
    BarHint, FetchButton, FetchNote, ModelError, ModelState, ModelsSheet, PanelButton, VoiceClick,
    VoicePanel, VoiceSheet, VoiceTab,
};

use super::rig::{DIR, Rig, row};
use super::*;

mod cloud;
mod compare;
mod device;
mod folder;
mod insert;
mod lists;
mod props;

/// The Models page as shown.
fn models(view: &VoicePanel) -> &ModelsSheet {
    match &view.sheet {
        VoiceSheet::Models(sheet) => sheet,
        other => panic!("not Models: {other:?}"),
    }
}

/// The rig on the Models page, with the folder holding `held`.
fn on_models(held: &[&str]) -> Rig {
    let mut rig = Rig::foldered();
    let rows: Vec<_> = held.iter().map(|id| row(id)).collect();
    rig.holds(&rows);
    rig.click(&VoiceClick::Open(VoiceTab::Models));
    rig
}

/// The shown place of the row named `name` on the Models page's first page.
fn place(rig: &mut Rig, name: &str) -> u8 {
    let view = rig.view().unwrap();
    let at = models(&view)
        .rows
        .iter()
        .position(|r| r.name.expose() == name);
    u8::try_from(at.unwrap()).unwrap()
}

#[test]
fn the_models_page_shows_the_folder_with_change_and_open() {
    let mut rig = on_models(&[]);
    let view = rig.view().unwrap();
    let line = models(&view).folder.clone().unwrap();
    assert_eq!(
        (line.path.expose().as_str(), line.free),
        (DIR, 900_000 * MB_TEST)
    );
    assert!(view.acts.contains(&PanelButton::Change) && view.acts.contains(&PanelButton::Open));
    assert_eq!(
        rig.click(&VoiceClick::Button(PanelButton::Open)),
        [Do::Reveal(DIR.into())]
    );
    rig.click(&VoiceClick::Button(PanelButton::Change));
    assert_eq!(rig.panel.page(), Some(Page::Folders));
}

/// Bytes in a MB, as the rig's drives count them.
const MB_TEST: u64 = kx_module_api::ports::MB;

#[test]
fn the_models_page_marks_default_ready_download_and_names_the_best() {
    let parakeet = row("parakeet-v3-q8");
    let mut rig = on_models(&["parakeet-v3-q8"]);
    let view = rig.view().unwrap();
    let sheet = models(&view);
    let first = &sheet.rows[0];
    assert_eq!(first.name.expose(), &parakeet.name);
    assert!(first.default && first.used && first.state == ModelState::Ready);
    assert!(
        sheet.rows[1..]
            .iter()
            .all(|r| !r.default && r.state == ModelState::Missing)
    );
    assert_eq!(sheet.best.as_deref(), Some(parakeet.name.as_str()));
    assert!(!sheet.untested);
}

#[test]
fn nothing_acts_until_a_row_is_ticked() {
    let small = row("whisper-small-q5km");
    let mut rig = on_models(&[]);
    let view = rig.view().unwrap();
    for act in [PanelButton::Use, PanelButton::Download, PanelButton::Delete] {
        assert!(!view.acts.contains(&act), "{act:?}");
    }
    assert!(
        rig.click(&VoiceClick::Button(PanelButton::Download))
            .is_empty()
    );
    let at = place(&mut rig, &small.name);
    rig.click(&VoiceClick::Row(at));
    let view = rig.view().unwrap();
    assert_eq!(view.ticked, 1);
    assert!(view.acts.contains(&PanelButton::Download) && !view.acts.contains(&PanelButton::Use));
    let out = rig.click(&VoiceClick::Button(PanelButton::Download));
    assert!(matches!(&out[..], [Do::Fetch { row, .. }] if *row == small.id));
}

#[test]
fn a_ready_row_is_used_for_the_language_or_deleted() {
    let small = row("whisper-small-q5km");
    let mut rig = on_models(&["parakeet-v3-q8", "whisper-small-q5km"]);
    let at = place(&mut rig, &small.name);
    rig.click(&VoiceClick::Row(at));
    let out = rig.click(&VoiceClick::Button(PanelButton::Use));
    let Some(Do::Set(key, value)) = out.first() else {
        panic!("{out:?}");
    };
    assert_eq!(
        (*key, value.get("en").and_then(|v| v.as_str())),
        (crate::settings::PICKS, Some(small.id.as_str()))
    );
    rig.click(&VoiceClick::Row(at));
    let out = rig.click(&VoiceClick::Button(PanelButton::Delete));
    assert_eq!(
        out,
        [Do::Remove(Path::new(DIR).join(&small.file)), Do::Scan]
    );
}

#[test]
fn the_model_in_use_keeps_its_file() {
    let parakeet = row("parakeet-v3-q8");
    let mut rig = on_models(&["parakeet-v3-q8"]);
    let at = place(&mut rig, &parakeet.name);
    rig.click(&VoiceClick::Row(at));
    let acts = rig.view().unwrap().acts;
    assert!(!acts.contains(&PanelButton::Delete) && !acts.contains(&PanelButton::Use));
    assert!(
        rig.click(&VoiceClick::Button(PanelButton::Delete))
            .is_empty()
    );
}

#[test]
fn a_download_shows_progress_with_pause_and_cancel_on_its_row() {
    let small = row("whisper-small-q5km");
    let mut rig = on_models(&[]);
    let at = place(&mut rig, &small.name);
    rig.click(&VoiceClick::Row(at));
    rig.click(&VoiceClick::Button(PanelButton::Download));
    rig.panel.fetch_note(
        &small.id,
        FetchNote::Progress {
            done: 9,
            total: small.bytes,
        },
    );
    let view = rig.view().unwrap();
    let fetch = models(&view).rows[usize::from(at)].fetch.unwrap();
    assert_eq!(
        (fetch.done, fetch.total, fetch.paused),
        (9, small.bytes, false)
    );
    let pause = VoiceClick::Fetch {
        row: at,
        button: FetchButton::Pause,
    };
    assert_eq!(
        rig.click(&pause),
        [Do::FetchCall {
            row: small.id.clone(),
            button: FetchButton::Pause
        }]
    );
    let cancel = VoiceClick::Fetch {
        row: at,
        button: FetchButton::Cancel,
    };
    assert_eq!(rig.click(&cancel).len(), 1);
    assert!(
        models(&rig.view().unwrap()).rows[usize::from(at)]
            .fetch
            .is_none()
    );
}

#[test]
fn a_finished_download_is_ready_and_a_bad_fingerprint_tells_a_note() {
    let small = row("whisper-small-q5km");
    let mut rig = on_models(&[]);
    rig.with(|p, f| p.download(&small.id, f));
    assert_eq!(rig.panel.fetch_note(&small.id, FetchNote::Done), [Do::Scan]);
    let at = usize::from(place(&mut rig, &small.name));
    assert_eq!(
        models(&rig.view().unwrap()).rows[at].state,
        ModelState::Ready
    );
    let turbo = row("whisper-turbo-q5km");
    rig.with(|p, f| p.download(&turbo.id, f));
    rig.panel
        .fetch_note(&turbo.id, FetchNote::Fault(ModelError::Print));
    assert_eq!(rig.view().unwrap().note, Some(BarHint::Print));
}
