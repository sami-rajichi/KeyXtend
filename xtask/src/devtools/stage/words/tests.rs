use super::*;
use crate::devtools::stage::check;
use crate::test_support::{TempDir, stage_config};
use kx_module_api::ports::PackIndex;

/// The index text listing `isos`, which serve no other code.
fn index_text(isos: &[&str]) -> String {
    PackIndex::new(isos, &std::collections::BTreeMap::new())
        .unwrap()
        .text()
}

/// The credit files the words manifest names.
fn credits() -> Vec<String> {
    ["CREDITS.txt", "TATOEBA-AUTHORS.txt", "LICENSE.txt"]
        .map(String::from)
        .to_vec()
}

/// Writes a packs folder at `dir` whose index lists `isos`, with each pack and every credit.
fn fill(dir: &Path, isos: &[&str]) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(PACK_INDEX), index_text(isos)).unwrap();
    for iso in isos {
        std::fs::write(dir.join(pack_file(iso).unwrap()), iso).unwrap();
    }
    for credit in credits() {
        std::fs::write(dir.join(credit), "c").unwrap();
    }
}

/// A stage whose packs folder lists Arabic, English and French, with everything in place.
fn stage(label: &str) -> TempDir {
    let dir = TempDir::new(label);
    fill(&dir.path().join(PACK_DIR), &["ar", "en", "fr"]);
    dir
}

/// The refusal `check` gives `stage`, if any.
fn refusal(stage: &TempDir) -> Option<StageError> {
    match check(stage.path(), &credits()) {
        Err(DevError::Stage(why)) => Some(why),
        _ => None,
    }
}

/// `name` in the stage's packs folder, as a refusal names it.
fn staged(name: &str) -> String {
    paths::text(&Path::new(PACK_DIR).join(name))
}

#[test]
fn a_complete_packs_folder_passes() {
    assert_eq!(check(stage("words-pass").path(), &credits()), Ok(()));
}

#[test]
fn a_listed_pack_that_is_missing_is_refused() {
    let stage = stage("words-pack");
    std::fs::remove_file(stage.path().join(PACK_DIR).join("fr.kxw")).unwrap();
    assert_eq!(refusal(&stage), Some(StageError::NoWords(staged("fr.kxw"))));
}

#[test]
fn every_credit_file_must_be_there() {
    for credit in credits() {
        let stage = stage("words-credit");
        std::fs::remove_file(stage.path().join(PACK_DIR).join(&credit)).unwrap();
        assert_eq!(refusal(&stage), Some(StageError::NoWords(staged(&credit))));
    }
}

#[test]
fn a_stage_without_the_index_is_refused() {
    let stage = stage("words-index");
    std::fs::remove_file(stage.path().join(PACK_DIR).join(PACK_INDEX)).unwrap();
    let missing = StageError::NoWords(staged(PACK_INDEX));
    assert_eq!(refusal(&stage), Some(missing));
    let bare = TempDir::new("words-none");
    assert_eq!(
        refusal(&bare),
        Some(StageError::NoWords(staged(PACK_INDEX)))
    );
}

#[test]
fn an_index_that_cannot_be_read_keeps_its_own_error() {
    let stage = stage("words-unread");
    let index = stage.path().join(PACK_DIR).join(PACK_INDEX);
    std::fs::remove_file(&index).unwrap();
    std::fs::create_dir(&index).unwrap();
    let checked = check(stage.path(), &credits());
    let named = |t: &String| t.starts_with(&staged(PACK_INDEX));
    assert!(
        matches!(&checked, Err(DevError::Io(t)) if named(t)),
        "{checked:?}"
    );
}

#[test]
fn a_damaged_or_empty_index_is_refused() {
    for text in ["EN\n", ""] {
        let stage = stage("words-damaged");
        std::fs::write(stage.path().join(PACK_DIR).join(PACK_INDEX), text).unwrap();
        let damaged = StageError::BadIndex(staged(PACK_INDEX));
        assert_eq!(refusal(&stage), Some(damaged), "{text:?}");
    }
}

#[test]
fn a_file_the_index_does_not_name_is_refused() {
    let stage = stage("words-extra");
    let extra = Path::new("old").join("de.kxw");
    let at = stage.path().join(PACK_DIR).join(&extra);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, "de").unwrap();
    let shown = paths::text(&Path::new(PACK_DIR).join(extra));
    assert_eq!(refusal(&stage), Some(StageError::Unplanned(shown)));
}

#[test]
fn the_plan_copies_the_index_each_listed_pack_and_the_credits_into_the_packs_folder() {
    let dir = TempDir::new("words-plan");
    fill(dir.path(), &["en", "fr"]);
    std::fs::write(dir.path().join("de.kxw"), "unlisted").unwrap();
    let words = Words {
        dir: dir.path().to_path_buf(),
        credits: credits(),
    };
    let names = [PACK_INDEX, "en.kxw", "fr.kxw"].map(String::from);
    let want: Vec<Copy> = names
        .into_iter()
        .chain(credits())
        .map(|n| Copy {
            from: dir.path().join(&n),
            to: Path::new(PACK_DIR).join(n),
        })
        .collect();
    assert_eq!(plan(&words), Ok(want));
}

#[test]
fn a_source_without_its_index_names_the_folder() {
    let dir = TempDir::new("words-source");
    let words = Words {
        dir: dir.path().to_path_buf(),
        credits: credits(),
    };
    let missing = StageError::NoWords(paths::text(&dir.path().join(PACK_INDEX)));
    assert_eq!(plan(&words), Err(missing.into()));
}

#[test]
fn the_words_manifest_names_the_assets_folder_and_the_three_credit_files() {
    let words = Words::load(Path::new("ws")).unwrap();
    let dir = paths::join_rel(PathBuf::from("ws"), "assets/words");
    assert_eq!((words.dir, words.credits), (dir, credits()));
}

#[test]
fn every_file_the_plan_takes_from_the_committed_packs_is_there_named_exactly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let words = Words::load(&root).unwrap();
    let files = plan(&words).unwrap();
    assert!(
        files
            .iter()
            .any(|c| c.to == Path::new(PACK_DIR).join(PACK_INDEX))
    );
    // A listing gives each name as the disk holds it, so case drift shows on a case-blind disk too.
    let listed: BTreeSet<PathBuf> = paths::files(&words.dir).unwrap().into_iter().collect();
    let named = |c: &&Copy| {
        c.from
            .strip_prefix(&words.dir)
            .is_ok_and(|n| listed.contains(n))
    };
    let gone: Vec<_> = files.iter().filter(|c| !named(c)).collect();
    assert!(gone.is_empty(), "{gone:?}");
}

#[test]
fn only_files_in_the_packs_folder_are_left_to_this_check() {
    let inside_of = |rel: &str| inside(&Path::new(rel).join("x.txt"));
    let variant = PACK_DIR.to_uppercase();
    assert_ne!(variant, PACK_DIR);
    assert!(inside_of(PACK_DIR) && !inside_of(&variant));
    assert!(!inside(Path::new(PACK_DIR)) && !inside_of("qml") && !inside_of(""));
    assert!(!inside(&Path::new("qml").join(PACK_DIR).join("x")));
}

/// The stage check on `stage`, with the program beside its packs and every import found.
fn stage_check(stage: &TempDir) -> Result<check::Summary, DevError> {
    let config = stage_config();
    std::fs::write(stage.path().join(config.program()), "app").unwrap();
    let level = crate::words::download_level().unwrap();
    check::check(stage.path(), &config, level, |_| Ok(Vec::new()), |_| true)
}

#[test]
fn a_packs_folder_named_in_another_case_is_refused_on_any_disk() {
    let dir = TempDir::new("words-case");
    let variant = PACK_DIR.to_uppercase();
    fill(&dir.path().join(&variant), &["en"]);
    let checked = stage_check(&dir);
    let named = |f: &String| Path::new(f).starts_with(&variant);
    assert!(
        matches!(&checked, Err(DevError::Stage(StageError::Unplanned(f))) if named(f)),
        "{checked:?}"
    );
}

#[test]
fn a_listed_file_named_in_another_case_is_refused_on_any_disk() {
    let stage = stage("words-file-case");
    let dir = stage.path().join(PACK_DIR);
    std::fs::remove_file(dir.join(PACK_INDEX)).unwrap();
    std::fs::write(
        dir.join(PACK_INDEX.to_uppercase()),
        index_text(&["ar", "en", "fr"]),
    )
    .unwrap();
    let refused = refusal(&stage);
    // A case-blind disk finds the index, then meets the variant; a case-sensitive one finds no index.
    let (upper, lower) = (staged(&PACK_INDEX.to_uppercase()), staged(PACK_INDEX));
    let case_blind = refused == Some(StageError::Unplanned(upper));
    assert!(
        case_blind || refused == Some(StageError::NoWords(lower)),
        "{refused:?}"
    );
}

#[test]
fn the_stage_check_leaves_the_packs_folder_alone_but_not_the_rest() {
    let stage = stage("words-stage");
    assert!(stage_check(&stage).is_ok());
    std::fs::write(stage.path().join("extra.txt"), "x").unwrap();
    let extra = StageError::Unplanned("extra.txt".to_string());
    assert_eq!(stage_check(&stage), Err(extra.into()));
}
