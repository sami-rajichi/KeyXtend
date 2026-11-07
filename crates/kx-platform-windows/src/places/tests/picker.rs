//! The folder list: a folder's subfolders and its files of one extension, A to Z ignoring case, without hidden or
//! system entries and never through a junction, stopping at the table's limit.

use super::*;
use crate::places::list;
use crate::places::table::Table;
use crate::testlinks::{attrib, junction};

#[test]
fn subfolders_list_a_to_z_without_hidden_or_system_ones_or_junctions() {
    let tree = Tree::new("places-folders");
    let top = tree.make("drive");
    for name in ["b", "A", "c/d", "hidden", "system"] {
        tree.make(&format!("drive/{name}"));
    }
    attrib("+h", &top.join("hidden"));
    attrib("+s", &top.join("system"));
    junction(&top.join("link"), &tree.make("Elsewhere"));
    fs::write(top.join("file.txt"), b"x").unwrap();
    let folder = |name| place(PlaceKind::Folder, name, &top.join(name));
    let want = vec![folder("A"), folder("b"), folder("c")];
    assert_eq!(tree.places().folders(Some(&top)), Ok(want));
}

#[test]
fn files_of_one_extension_list_a_to_z_in_any_case_without_hidden_ones() {
    let tree = Tree::new("places-files");
    let top = tree.make("drive");
    for name in ["x.kdbx", "B.KDBX", "a.txt", "kdbx", "h.kdbx"] {
        fs::write(top.join(name), b"x").unwrap();
    }
    tree.make("drive/folder.kdbx");
    attrib("+h", &top.join("h.kdbx"));
    let file = |name| place(PlaceKind::File, name, &top.join(name));
    let want = vec![file("B.KDBX"), file("x.kdbx")];
    let places = tree.places();
    assert_eq!(places.files(&top, "kdbx"), Ok(want.clone()));
    assert_eq!(places.files(&top, "KDBX"), Ok(want), "any case");
}

#[test]
fn a_list_stops_at_the_table_limit() {
    let tree = Tree::new("places-limit");
    let top = tree.make("drive");
    for name in ["a", "b", "c"] {
        tree.make(&format!("drive/{name}"));
        fs::write(top.join(format!("{name}.kdbx")), b"x").unwrap();
    }
    let mut table = Table::parse(crate::places::TABLE).unwrap();
    table.max_listed = 2;
    let folders = list::folders(&top, &table).unwrap();
    let files = list::files(&top, "kdbx", &table).unwrap();
    assert_eq!((folders.len(), files.len()), (2, 2));
}
