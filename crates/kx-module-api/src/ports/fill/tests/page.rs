//! The page's rules: a hidden value is carried only while shown, only New and Edit switch a hidden flag, a plain note
//! names no number, every sheet is sampled, the filters' order, and each listed enum in declaration order.

use super::sheets::place;
use super::*;
use crate::ports::{MAX_VAULT_ENTRIES, MAX_VAULT_FIELDS};

/// The texts a field line holds, `None` for a mask.
fn held(line: &FieldLine) -> Option<&str> {
    line.value.text().map(|t| t.expose().as_str())
}

/// A field line of secret `n`, hidden or not, and shown or not.
fn line(n: usize, hidden: bool, shown: bool) -> FieldLine {
    FieldLine {
        label: Label::Table("password".to_owned()),
        value: FieldValue::new(secret(n), hidden, shown),
        make: false,
    }
}

#[test]
fn a_hidden_value_is_carried_only_while_shown() {
    let masked = FieldValue::new(secret(0), true, false);
    assert_eq!(masked.text(), None);
    assert!(masked.hidden() && !masked.shown());
    let shown = FieldValue::new(secret(0), true, true);
    assert_eq!(shown.text(), Some(&secret(0)));
    assert!(shown.hidden() && shown.shown());
    let plain = FieldValue::new(secret(1), false, false);
    assert_eq!(plain.text(), Some(&secret(1)));
    assert!(
        !plain.hidden() && !plain.shown(),
        "a plain value is never shown or hidden"
    );
}

#[test]
fn an_empty_hidden_value_carries_its_empty_text_and_still_offers_show() {
    for value in [FieldValue::new, FieldValue::edit] {
        let empty = value(Secret::default(), true, false);
        assert_eq!(empty.text(), Some(&Secret::default()), "so no mask draws");
        assert!(empty.hidden() && !empty.shown(), "it still offers Show");
        assert_eq!(empty.count(), None, "and no dot");
    }
}

#[test]
fn only_a_masked_value_on_edit_tells_its_count() {
    let masked = FieldValue::edit(secret(1), true, false);
    assert_eq!(masked.text(), None);
    assert_eq!(masked.count(), Some(4), "Unicode scalars, not bytes");
    let read = FieldValue::new(secret(1), true, false);
    assert_eq!(read.count(), None, "a read page's mask tells no length");
    let shown = FieldValue::edit(secret(1), true, true);
    assert_eq!(shown.count(), None, "its text says it");
    let plain = FieldValue::edit(secret(1), false, false);
    assert_eq!(plain.count(), None, "its text says it");
}

#[test]
fn a_masked_value_needs_no_copy_of_its_text() {
    let read = FieldValue::new(secret(0), true, false);
    assert_eq!(FieldValue::masked(), read);
    let count = SECRETS[0].chars().count();
    let edit = FieldValue::edit(secret(0), true, false);
    assert_eq!(FieldValue::masked_edit(count), edit);
}

#[test]
fn a_count_never_prints() {
    let printed = format!("{:?}", FieldValue::masked_edit(7919));
    assert!(!printed.contains("7919"), "{printed}");
}

#[test]
fn only_a_new_or_edit_value_switches_its_hidden_flag() {
    let edit = FieldValue::edit(secret(0), true, false);
    assert!(edit.switches() && edit.hidden());
    assert_eq!(edit.text(), None, "a hidden value stays out while unshown");
    assert!(FieldValue::edit(secret(1), false, false).switches());
    assert!(!FieldValue::new(secret(1), false, false).switches());
    let switch = FillClick::Field {
        field: 0,
        button: FieldButton::Hidden,
    };
    let show = FillClick::Field {
        field: 0,
        button: FieldButton::Show,
    };
    assert_ne!(switch, show, "switching the flag is not showing the value");
}

#[test]
fn a_label_counts_every_value_at_the_settings_tops() {
    // Every field of every entry under one label, at the tops of `max_entries` and `max_fields`.
    let count = u32::try_from(MAX_VAULT_ENTRIES * MAX_VAULT_FIELDS).unwrap();
    let row = LabelCount {
        label: Label::Own(text(0)),
        count,
    };
    assert_eq!(row.count, count);
}

#[test]
fn an_entry_page_holds_only_the_values_it_shows() {
    let fields = vec![
        line(0, true, false),
        line(1, false, false),
        line(2, true, true),
    ];
    let sheet = Sheet::Entry(EntrySheet {
        kind: EntryKind::Login,
        name: text(1),
        fields,
    });
    let Sheet::Entry(entry) = page(sheet).sheet else {
        unreachable!("the page shows the entry it was given")
    };
    let texts: Vec<_> = entry.fields.iter().map(held).collect();
    assert_eq!(texts, [None, Some(SECRETS[1]), Some(SECRETS[2])]);
}

#[test]
fn a_plain_note_names_no_number() {
    let note = PageNote::plain(FillNote::Typed);
    assert_eq!(note.note, FillNote::Typed);
    assert_eq!(note.counts, NoteCounts::default());
}

#[test]
fn paste_tells_an_empty_a_busy_and_an_off_clipboard_apart() {
    let paste = [FillNote::NoText, FillNote::ClipBusy, FillNote::ClipOff];
    assert!(paste.iter().all(|n| FillNote::ALL.contains(n)));
}

#[test]
fn the_backup_tells_a_box_too_large_and_a_failed_worker_apart() {
    let notes = [FillNote::TooLarge, FillNote::BoxFailed];
    assert!(notes.iter().all(|n| FillNote::ALL.contains(n)));
    assert_ne!(notes[0], notes[1]);
}

#[test]
fn the_backup_page_asks_one_question_at_a_time_on_a_row_or_for_the_page() {
    let asking = |ask| BackupSheet {
        places: Vec::new(),
        checks: false,
        ask,
    };
    let remove = asking(Some(BackupAsk::Remove(2)));
    assert_eq!(remove.ask, Some(BackupAsk::Remove(2)));
    assert_ne!(remove, asking(Some(BackupAsk::Replace)));
    assert_ne!(remove, asking(None));
}

#[test]
fn a_held_backup_has_its_own_note() {
    assert!(FillNote::ALL.contains(&FillNote::Paused));
}

#[test]
fn a_failed_save_has_its_own_note() {
    assert!(FillNote::ALL.contains(&FillNote::SaveFailed));
}

#[test]
fn the_last_wrong_check_and_an_unread_box_have_their_own_notes() {
    let notes = [FillNote::TriesUsed, FillNote::Unread];
    assert!(notes.iter().all(|n| FillNote::ALL.contains(n)));
    assert_ne!(notes[0], notes[1]);
}

#[test]
fn every_sheet_is_sampled_once_for_the_debug_check() {
    let places: Vec<_> = sheets().iter().map(place).collect();
    let want: Vec<_> = (0..places.len()).collect();
    assert_eq!(places, want);
}

#[test]
fn the_filters_step_from_all_through_each_kind_once_to_by_label() {
    assert_eq!(FillFilter::ALL.first(), Some(&FillFilter::All));
    assert_eq!(FillFilter::ALL.last(), Some(&FillFilter::Labels));
}

/// Checks that each listed enum's `ALL` holds every variant once, in declaration order.
macro_rules! in_order {
    ($($name:ty),+) => {
        $(for (at, v) in <$name>::ALL.into_iter().enumerate() {
            assert_eq!(v as usize, at, "{v:?}");
        })+
    };
}

#[test]
fn every_listed_enum_holds_each_variant_once_in_declaration_order() {
    // `listed!` declares each enum with its `ALL`, so a new variant is listed by being declared.
    in_order!(
        EntryKind,
        FillFilter,
        RowButton,
        FieldButton,
        FillButton,
        FillNote,
        Pick
    );
}
