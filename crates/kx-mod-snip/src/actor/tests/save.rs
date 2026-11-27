//! Save: the system's dialog proposes the snip's date and time in the save folder, and the snip is written only where
//! it answered; Cancel comes back to the strip, and a full or read-only place gives a note (edge 6). A hidden run dims
//! Save (edge 21).

use kx_module_api::Redacted;
use kx_module_api::ports::{
    AskError, AskKey, Dim, FileKind, FolderKey, Keep, PickLimits, SaveError, SnipClick, SnipNote,
    SnipStage, StripKey, Taken,
};
use kx_platform_fake::{Pick, Stored};
use toml::Value;

use super::fakes::{CHOSEN, Fakes, PNG, SNIPS, dir, file};
use super::rig::{Rig, held};

/// The strip's keys and its tick, told last.
fn strip(rig: &Rig) -> Taken {
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("a strip shows");
    };
    taken
}

#[test]
fn save_proposes_the_snips_date_and_time_in_the_snips_folder_and_writes_the_png_there() {
    let chosen = file(CHOSEN, "shot.png", FileKind::Png);
    let dialog_min = *PickLimits::DIALOG_MIN.end();
    let mut rig = Rig::build(|fake, host| {
        fake.dialogs.queue_save(Pick::File(chosen.clone()));
        host.put("dialog_min", Value::Integer(dialog_min.into()));
    });
    rig.snap();
    rig.keep(Keep::Save);
    let proposed = file(SNIPS, "2026-11-24 14-05-00.png", FileKind::Png);
    assert_eq!(rig.fake.dialogs.saves(), [proposed]);
    let limits = rig.fake.dialogs.limits();
    assert_eq!(limits, [rig.actor.held.pick_limits()], "the snip's own");
    assert_eq!(limits[0].dialog_min, dialog_min);
    let png = Stored::Png(Redacted::new(PNG.to_vec()));
    assert_eq!(rig.fake.folder.stored(&chosen), Some(png));
    assert_eq!(
        rig.fake.folder.files(),
        [chosen],
        "nothing written anywhere else"
    );
    assert_eq!(strip(&rig).done, Some(Keep::Save));
}

#[test]
fn save_opens_at_save_dir_when_it_is_set() {
    let mut rig = Rig::build(|_, host| host.put("save_dir", Value::String(CHOSEN.into())));
    rig.snap();
    rig.keep(Keep::Save);
    let proposed = file(CHOSEN, "2026-11-24 14-05-00.png", FileKind::Png);
    assert_eq!(rig.fake.dialogs.saves(), [proposed]);
}

#[test]
fn a_folder_changed_on_the_panel_during_a_snip_counts_for_its_save() {
    let mut rig = Rig::build(|fake, _| fake.dialogs.queue_folder(Ok(Some(dir(CHOSEN)))));
    rig.snap();
    rig.act(SnipClick::Open);
    rig.act(SnipClick::Folder(FolderKey::Change));
    rig.act(SnipClick::ClosePanel);
    rig.keep(Keep::Save);
    let proposed = file(CHOSEN, "2026-11-24 14-05-00.png", FileKind::Png);
    assert_eq!(rig.fake.dialogs.saves(), [proposed]);
}

#[test]
fn edge_6_a_cancelled_dialog_comes_back_to_the_strip() {
    let mut rig = Rig::new();
    rig.snap();
    rig.keep(Keep::Save);
    assert_eq!(rig.fake.dialogs.saves().len(), 1);
    let taken = strip(&rig);
    assert_eq!((taken.done, rig.page().note), (None, None));
    assert!(rig.fake.folder.files().is_empty());
    rig.act(SnipClick::Strip(StripKey::Close));
    assert!(rig.page().ask.is_some(), "still not kept, so Close asks");
}

#[test]
fn the_view_tells_the_save_dialog_until_it_answers_and_never_for_a_copy() {
    let mut rig = Rig::new();
    rig.snap();
    rig.actor.ports.spawn = held;
    rig.keep(Keep::Save);
    assert!(rig.page().dialog, "the Save dialog shows");
    rig.drain();
    assert!(!rig.page().dialog, "it was cancelled");
    rig.keep(Keep::Copy);
    assert!(!rig.page().dialog, "Copy shows no dialog");
}

#[test]
fn edge_6_a_full_or_read_only_place_gives_a_note_and_the_strip_stays() {
    for error in [SaveError::Full, SaveError::ReadOnly] {
        let chosen = file(CHOSEN, "shot.png", FileKind::Png);
        let mut rig =
            Rig::build(|fake, _| fake.dialogs.queue_save(Pick::Stuck(chosen.clone(), error)));
        rig.snap();
        rig.keep(Keep::Save);
        assert_eq!(rig.page().note, Some(SnipNote::SaveFailed), "{error:?}");
        assert_eq!(strip(&rig).done, None);
        assert!(rig.fake.folder.files().is_empty(), "nothing half-written");
    }
}

#[test]
fn a_dialog_that_never_showed_gives_a_note() {
    let mut rig = Rig::build(|fake, _| fake.dialogs.queue_save(Pick::Fail(AskError::Gone)));
    rig.snap();
    rig.keep(Keep::Save);
    assert_eq!(rig.page().note, Some(SnipNote::SaveFailed));
}

#[test]
fn a_place_the_checks_refuse_says_to_choose_another_and_the_strip_stays() {
    let mut rig = Rig::build(|fake, _| fake.dialogs.queue_save(Pick::Fail(AskError::BadPath)));
    rig.snap();
    rig.keep(Keep::Save);
    assert_eq!(rig.page().note, Some(SnipNote::PlaceRefused));
    assert_eq!(strip(&rig).done, None);
    assert!(rig.fake.folder.files().is_empty(), "nothing written");
}

#[test]
fn save_copies_a_video_where_the_dialog_said() {
    let chosen = file(CHOSEN, "clip.mp4", FileKind::Mp4);
    let mut rig = Rig::build(|fake, _| fake.dialogs.queue_save(Pick::File(chosen.clone())));
    rig.recorded(3_000);
    rig.keep(Keep::Save);
    let proposed = file(SNIPS, "2026-11-24 14-05-00.mp4", FileKind::Mp4);
    assert_eq!(rig.fake.dialogs.saves(), [proposed]);
    assert!(matches!(
        rig.fake.folder.stored(&chosen),
        Some(Stored::Video { length_ms: 3_000 })
    ));
    rig.act(SnipClick::Strip(StripKey::Close));
    assert_eq!(
        rig.fake.folder.files(),
        [chosen],
        "the saved video's recording goes at its end"
    );
}

#[test]
fn edge_21_a_hidden_run_dims_save_and_still_copies() {
    let mut rig = Rig::with(Fakes::hidden(), |_, _| {});
    rig.snap();
    let save = strip(&rig)
        .keys
        .into_iter()
        .find(|k| k.key == StripKey::Save)
        .unwrap();
    assert_eq!(save.dim, Some(Dim::Hidden));
    rig.keep(Keep::Save);
    assert!(
        rig.fake.dialogs.saves().is_empty(),
        "a dimmed key does nothing"
    );
    rig.act(SnipClick::Strip(StripKey::Close));
    assert_eq!(rig.page().ask.map(|a| a.save_dim), Some(Some(Dim::Hidden)));
    rig.act(SnipClick::Ask(AskKey::Back));
    rig.keep(Keep::Copy);
    assert_eq!(rig.fake.clip.writes().len(), 1);
}
