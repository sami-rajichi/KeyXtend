//! Quick-fill's backup and import pages for the fixture: the Backup page with its questions, the places and folders to
//! pick, the `KeePass` pick and its result, the password page for each thing it asks, and the boxes found or foreign.

use super::{page, secret, shown};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    BackupAsk, BackupSheet, BoxState, Date, FillNote, FillPage, FolderSheet, FoundRow, NoteCounts,
    PageNote, PasswordAsk, PasswordSheet, Pick, PlaceKind, PlaceRow, Sheet, Spot, When,
};

/// A folder with an Arabic name, which reads right to left in the Arabic font.
const ARABIC: &str = "مستنداتي";
/// A folder whose one long word runs past a row's question.
const LONG: &str = "Family_photos_and_scanned_documents_from_the_old_computer_before_the_move_to_the_new_flat_sorted_by_year_and_by_person_with_their_captions_and_dates";

/// 10:42 on 1 November 2026, `days_ago` days back.
fn when(days_ago: u32) -> When {
    When {
        days_ago,
        date: Date::new(2026, 11, 1).expect("a real date"),
        hour: 10,
        minute: 42,
    }
}

/// `name`, a place of kind `kind`.
fn spot(kind: PlaceKind, name: &str) -> Spot {
    Spot {
        kind,
        name: shown(name),
    }
}

/// A place whose box was written `days_ago` days back.
fn updated(kind: PlaceKind, name: &str, days_ago: u32) -> PlaceRow {
    PlaceRow {
        place: spot(kind, name),
        state: BoxState::Updated(when(days_ago)),
    }
}

/// Three places: written today, yesterday, and one that waits.
fn places() -> Vec<PlaceRow> {
    vec![
        updated(PlaceKind::Cloud, "OneDrive", 0),
        updated(PlaceKind::Removable, "USB stick (E:)", 1),
        PlaceRow {
            place: spot(PlaceKind::Drive, "Backup (D:)"),
            state: BoxState::Waiting,
        },
    ]
}

/// The Backup page over `places`, asking `ask`, its memory checks `checks`.
fn home(places: Vec<PlaceRow>, checks: bool, ask: Option<BackupAsk>) -> Sheet {
    Sheet::Backup(BackupSheet {
        places,
        checks,
        ask,
    })
}

/// The Backup page with its waiting line, off, full and paged, with no place, asking to remove a row, one with a long
/// name, or to replace the backup, and telling what an import brought in.
pub(super) fn backups() -> Vec<(&'static str, FillPage)> {
    let mut plain = page(home(places(), true, None), 1);
    plain.waiting = Some(spot(PlaceKind::Drive, "Backup (D:)"));
    let mut full = places();
    full.push(updated(PlaceKind::Cloud, "Google Drive", 9));
    full.push(updated(PlaceKind::Cloud, "Dropbox", 30));
    let mut none = page(home(Vec::new(), true, None), 1);
    none.highlight = None;
    let removing = |places| {
        let mut p = page(home(places, true, Some(BackupAsk::Remove(1))), 1);
        p.highlight = Some(1);
        p
    };
    let mut long = places();
    long[1].place = spot(PlaceKind::Folder, LONG);
    let mut replace = page(home(places(), true, Some(BackupAsk::Replace)), 1);
    replace.note = Some(PageNote::plain(FillNote::Paused));
    let mut imported = page(home(places(), true, None), 1);
    imported.note = Some(PageNote {
        note: FillNote::Imported,
        counts: NoteCounts {
            added: 12,
            skipped: 2,
        },
    });
    vec![
        ("backup", plain),
        ("backup-off", page(home(places(), false, None), 1)),
        ("backup-paged", page(home(full, true, None), 2)),
        ("backup-none", none),
        ("remove", removing(places())),
        ("remove-long", removing(long)),
        ("replace", replace),
        ("imported", imported),
    ]
}

/// The list `rows` picks with `pick`, in `folder`, or at the drives.
fn listed(pick: Pick, folder: Option<&str>, rows: Vec<Spot>, count: u16) -> FillPage {
    page(
        Sheet::Folders(FolderSheet {
            pick,
            folder: folder.map(shown),
            rows,
        }),
        count,
    )
}

/// Five folders, one with an Arabic name.
fn folders() -> Vec<Spot> {
    ["Backups", "Documents", ARABIC, "Music", "Pictures"]
        .map(|name| spot(PlaceKind::Folder, name))
        .to_vec()
}

/// The suggested places, one that refuses a pick, then the drives, a folder's page, and the two lists that pick a file.
pub(super) fn picks() -> Vec<(&'static str, FillPage)> {
    let spots = || {
        vec![
            spot(PlaceKind::Cloud, "OneDrive"),
            spot(PlaceKind::Removable, "USB stick (E:)"),
            spot(PlaceKind::Drive, "Backup (D:)"),
        ]
    };
    let mut refused = page(Sheet::Places(spots()), 1);
    refused.note = Some(PageNote::plain(FillNote::DataFolder));
    let drives = vec![
        spot(PlaceKind::Drive, "Windows (C:)"),
        spot(PlaceKind::Drive, "Backup (D:)"),
        spot(PlaceKind::Removable, "USB stick (E:)"),
    ];
    let files = |name| vec![spot(PlaceKind::Folder, "Old"), spot(PlaceKind::File, name)];
    vec![
        ("places", page(Sheet::Places(spots()), 1)),
        ("places-refused", refused),
        ("drives", listed(Pick::Folder, None, drives, 1)),
        (
            "folders",
            listed(Pick::Folder, Some("Documents"), folders(), 2),
        ),
        (
            "files",
            listed(
                Pick::Backup,
                Some("Backups"),
                files("KeyXtend Backup.kxbox"),
                1,
            ),
        ),
        (
            "keepass",
            listed(Pick::Keepass, Some(ARABIC), files("Passwords.kdbx"), 1),
        ),
    ]
}

/// The password page asking `ask`, with `typed` so far.
fn asking(ask: PasswordAsk, typed: &str) -> FillPage {
    let mut p = page(
        Sheet::Password(PasswordSheet {
            ask,
            text: secret(typed),
            caret: Redacted::new(0),
        }),
        1,
    );
    p.highlight = None;
    p
}

/// The password page for each thing it asks, and a memory check after a wrong try.
pub(super) fn asks() -> Vec<(&'static str, FillPage)> {
    let check = PasswordAsk::Check { tries: 3 };
    let mut wrong = asking(check, "");
    wrong.note = Some(PageNote::plain(FillNote::WrongPassword));
    vec![
        (
            "pass-new",
            asking(PasswordAsk::New { min: 8 }, "Tr0ub4dor&3"),
        ),
        ("pass-check", asking(check, "Tr0ub")),
        ("pass-wrong", wrong),
        ("pass-open", asking(PasswordAsk::Open { own: false }, "")),
        ("pass-own", asking(PasswordAsk::Open { own: true }, "")),
        ("pass-foreign", asking(PasswordAsk::Foreign, "")),
        ("pass-keepass", asking(PasswordAsk::Keepass, "")),
    ]
}

/// The boxes found, newest first, and a box from another computer, with its date and without one.
pub(super) fn checks() -> Vec<(&'static str, FillPage)> {
    let boxed = |kind, name, days_ago| FoundRow {
        place: spot(kind, name),
        when: when(days_ago),
    };
    let found = vec![
        boxed(PlaceKind::Cloud, "OneDrive", 0),
        boxed(PlaceKind::Removable, "USB stick (E:)", 9),
    ];
    let mut foreign = page(Sheet::Foreign(boxed(PlaceKind::Cloud, "OneDrive", 2)), 1);
    foreign.highlight = None;
    let mut undated = page(
        Sheet::ForeignUndated(spot(PlaceKind::Removable, "USB stick (E:)")),
        1,
    );
    undated.highlight = None;
    vec![
        ("found", page(Sheet::Found(found), 1)),
        ("foreign", foreign),
        ("foreign-undated", undated),
    ]
}
