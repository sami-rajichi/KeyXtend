use super::*;

/// A translation file with a comment, a term, a message with attributes and a plain message.
const FTL: &str = "## The product's name.\n-app = KeyXtend\n\nwindow =\n    .keyboard = { -app } keyboard\n    \
                   .outline = Scroll outline\ncatch = Click to end\nother =\n    .outline = Not this one\n";

#[test]
fn a_key_names_an_attribute_or_a_plain_message_and_its_terms_are_filled() {
    assert_eq!(
        text(FTL, "window.keyboard").as_deref(),
        Some("KeyXtend keyboard")
    );
    assert_eq!(
        text(FTL, "window.outline").as_deref(),
        Some("Scroll outline")
    );
    assert_eq!(text(FTL, "other.outline").as_deref(), Some("Not this one"));
    assert_eq!(text(FTL, "catch").as_deref(), Some("Click to end"));
    assert_eq!(text(FTL, "-app").as_deref(), Some("KeyXtend"));
}

#[test]
fn a_missing_message_attribute_or_term_gives_none() {
    assert_eq!(text(FTL, "window.nowhere"), None);
    assert_eq!(text(FTL, "nowhere.outline"), None);
    assert_eq!(
        text(FTL, "window"),
        None,
        "a message with no value of its own"
    );
    assert_eq!(text("x =\n    .a = { -gone } here\n", "x.a"), None);
}

#[test]
fn arabic_text_comes_back_as_written() {
    let ar = "-app = KeyXtend\nwindow =\n    .keyboard = لوحة مفاتيح { -app }\n";
    assert_eq!(
        text(ar, "window.keyboard").as_deref(),
        Some("لوحة مفاتيح KeyXtend")
    );
}

#[test]
fn the_languages_are_exactly_the_files_kx_ui_ships() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/kx-ui/i18n");
    let entries = std::fs::read_dir(dir).expect("kx-ui's i18n folder");
    // Feature folders, such as the clipboard panel's, hold no window title, so only the main files count.
    let files = entries
        .map(|e| e.expect("an entry"))
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()));
    let mut shipped: Vec<String> = files
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    shipped.sort();
    let mut ours: Vec<String> = FILES.iter().map(|(tag, ..)| format!("{tag}.ftl")).collect();
    ours.sort();
    assert_eq!(shipped, ours);
}

#[test]
fn a_variables_text_splits_around_it_with_its_terms_filled() {
    let ftl = "-app = KeyXtend\nchip =\n    .name = { -app } pick: { $word }!\n";
    let got = split(ftl, "chip.name", "word");
    assert_eq!(got, Some(("KeyXtend pick: ".into(), "!".into())));
    assert_eq!(split(ftl, "chip.name", "other"), None, "another variable");
    assert_eq!(split(ftl, "chip.none", "word"), None, "no such key");
    let ar = around("chip.name", "word").expect("in all three");
    assert_eq!(ar.len(), FILES.len());
    assert!(ar.iter().all(|(before, _)| !before.is_empty()));
    let e = around("chip.name", "nothing").expect_err("no such variable");
    assert!(e.contains("$nothing") && e.contains("en"), "{e}");
}

#[test]
fn a_string_literal_placeable_is_its_text_and_an_escaped_one_gives_none() {
    let ftl = "x =\n    .join = {\", \"}\n    .ar = {\"، \"}\n    .esc = {\"a\\u0020\"}\n";
    assert_eq!(text(ftl, "x.join").as_deref(), Some(", "));
    assert_eq!(text(ftl, "x.ar").as_deref(), Some("، "));
    assert_eq!(text(ftl, "x.esc"), None, "no escapes are read");
}

#[test]
fn the_clip_panels_texts_are_read_in_every_language_beside_the_main_ones() {
    let insert = around("clip.insert", "count").expect("in all three");
    assert_eq!(insert[0], ("Insert (".into(), ")".into()));
    assert!(
        insert
            .iter()
            .all(|(before, after)| !before.is_empty() && after == ")")
    );
    assert_eq!(
        every("clip.join").expect("in all three"),
        [", ", ", ", "، "]
    );
    assert!(
        every("window.outline").is_ok(),
        "the main files still count"
    );
}

#[test]
fn the_clip_folder_holds_exactly_the_languages_of_the_main_files() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/kx-ui/i18n/clip");
    let entries = std::fs::read_dir(dir).expect("kx-ui's clip folder");
    let mut shipped: Vec<String> = entries
        .map(|e| {
            e.expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    shipped.sort();
    let mut ours: Vec<String> = FILES.iter().map(|(tag, ..)| format!("{tag}.ftl")).collect();
    ours.sort();
    assert_eq!(shipped, ours);
}

#[test]
fn every_shipped_language_has_the_outline_title_and_a_missing_key_names_the_language() {
    let titles = every("window.outline").expect("in all three");
    assert_eq!(titles.len(), FILES.len());
    assert!(titles.iter().all(|t| !t.is_empty() && !t.contains('{')));
    let e = every("window.nowhere").expect_err("in none");
    assert!(e.contains("window.nowhere") && e.contains("en"), "{e}");
}

#[test]
fn a_text_is_cut_at_its_variables_with_terms_and_literals_filled() {
    let ftl = "-app = KeyXtend\npage =\n    .of = { $at } of { $count }\nnote = { -app }{\", \"}{ $n } left\n";
    let (var, words) = (
        |v: &str| Piece::Var(v.into()),
        |w: &str| Piece::Words(w.into()),
    );
    let page = [var("at"), words(" of "), var("count")];
    assert_eq!(pieces(ftl, "page.of").as_deref(), Some(&page[..]));
    let note = [words("KeyXtend"), words(", "), var("n"), words(" left")];
    assert_eq!(pieces(ftl, "note").as_deref(), Some(&note[..]));
    assert_eq!(pieces(ftl, "nowhere"), None);
    let shipped = shapes("page.of").expect("in all three");
    assert_eq!(shipped.len(), FILES.len());
    assert!(shipped.iter().all(|s| s.contains(&var("count"))));
    assert!(shapes("page.nowhere").is_err());
}
