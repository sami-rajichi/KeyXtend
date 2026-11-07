//! Junctions on Windows: one inside a copy goes as a link with its target untouched, and a copy behind one, or one that
//! is one, is never listed.

use super::*;
use crate::testlinks::junction;

/// A folder with one file, standing for data a delete must never reach.
fn precious(tree: &Tree) -> PathBuf {
    let target = tree.top.join("Precious");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("keep.txt"), b"keep").unwrap();
    target
}

#[test]
fn a_junction_inside_a_copy_is_deleted_as_a_link_and_its_target_stays() {
    let tree = Tree::new("redirect-junction");
    let target = precious(&tree);
    fs::create_dir(tree.copy_in(PKG)).unwrap();
    junction(&tree.copy_in(PKG).join("link"), &target);
    assert_eq!(tree.hidden().remove(&tree.listed(PKG, "Claude")), Ok(()));
    assert!(!tree.copy_in(PKG).exists());
    assert_eq!(names(&target), ["keep.txt"]);
}

#[test]
fn a_copy_behind_a_junction_or_that_is_one_is_never_listed() {
    let tree = Tree::new("redirect-behind");
    let steps = &table().unwrap().mirror;
    let elsewhere = tree.top.join("Elsewhere");
    let rest: PathBuf = steps[1..].iter().collect();
    fs::create_dir_all(elsewhere.join(rest).join(APP_DIR_NAME)).unwrap();
    let linked = tree.root.join("Linked_x");
    fs::create_dir(&linked).unwrap();
    junction(&linked.join(&steps[0]), &elsewhere);
    junction(&tree.copy_in(PKG), &precious(&tree));
    assert_eq!(tree.hidden().copies(), Ok(vec![]));
    let forged = HiddenCopy::new(tree.copy_in(PKG), PKG.into(), "Claude".into());
    assert_eq!(tree.hidden().remove(&forged), Err(HiddenError::Refused));
    assert_eq!(names(&tree.top.join("Precious")), ["keep.txt"]);
}

#[test]
fn a_junction_from_a_mirror_back_to_the_local_folder_is_a_normal_run() {
    let tree = Tree::new("redirect-loop");
    let steps = &table().unwrap().mirror;
    let (last, first) = steps.split_last().unwrap();
    let cache = tree
        .root
        .join("Linked_x")
        .join(first.iter().collect::<PathBuf>());
    fs::create_dir_all(&cache).unwrap();
    junction(&cache.join(last), &tree.local);
    fs::create_dir(tree.data()).unwrap();
    assert_eq!(tree.hidden().probe(&tree.data()), Ok(Probe::Normal));
    assert!(names(&tree.data()).is_empty(), "the probe file is gone");
}

#[test]
fn a_hidden_run_leaves_no_empty_copy_behind() {
    // The data folder is a junction into the mirror, standing in for Windows' redirection.
    let tree = Tree::new("redirect-empty");
    fs::create_dir(tree.copy_in(PKG)).unwrap();
    junction(&tree.data(), &tree.copy_in(PKG));
    let want = Probe::Hidden {
        package: PKG.into(),
        app: "Claude".into(),
    };
    assert_eq!(tree.hidden().probe(&tree.data()), Ok(want));
    assert!(!tree.copy_in(PKG).exists(), "the empty copy is gone");
    assert!(tree.mirror(PKG).is_dir());
}
