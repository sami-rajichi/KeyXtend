//! The places on Windows, held to the port over temp trees: the cloud folders found, the drives, the folder list, finds,
//! capped reads, writes in one step by any number of writers, missing and full places, the data folder refused, and
//! the disk dialogs off.

use std::fs;
use std::path::Path;

use kx_module_api::Redacted;
use kx_module_api::ports::{Place, PlaceKind, Places, box_file};

use super::table;
use tree::Tree;

mod disk;
mod find;
mod picker;
mod quiet;
mod suggest;
mod temps;
mod tree;

/// The shipped table.
fn shipped() -> &'static table::Table {
    table().unwrap()
}

/// A place of `kind` named `name` at `path`.
fn place(kind: PlaceKind, name: &str, path: &Path) -> Place {
    Place {
        kind,
        name: Redacted::new(name.to_owned()),
        path: Redacted::new(path.to_path_buf()),
    }
}

/// The drive at `root` labelled `label`, as the port lists it.
fn drive(kind: PlaceKind, root: &Path, label: &str) -> Place {
    place(kind, &shipped().drive_name(root, label), root)
}

#[test]
fn the_port_serves_another_thread() {
    let tree = Tree::new("places-thread");
    let folder = tree.make("OneDrive");
    let port: std::sync::Arc<dyn Places> = std::sync::Arc::new(tree.places());
    let wrote = std::thread::spawn(move || port.write(&folder, b"box"));
    assert_eq!(wrote.join().unwrap(), Ok(()));
    assert_eq!(
        fs::read(tree.top.join("OneDrive").join(box_file())).unwrap(),
        b"box"
    );
}
