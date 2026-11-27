//! Every window of ours takes its title from the hub's lists (`WINDOWS`, `TRAY_WINDOWS`), so a capture's shields find
//! it; a new untitled window, or one titled off the lists, fails here.

use crate::qml_files;
use crate::windows::{root, titles};
use kx_ui::i18n::keys::{
    CATCH, SEL_NAME, WINDOW_CLIP, WINDOW_FILL, WINDOW_OUTLINE, WINDOW_SNIP_EDGE,
    WINDOW_SNIP_SCREEN, WINDOW_VOICE, WINDOW_VOICE_PANEL,
};
use std::collections::BTreeSet;

/// The type every window comes from.
const WINDOW: &str = "Window";
/// What binds a window's title.
const TITLE: &str = "title:";
/// One level of QML indent.
const STEP: usize = 4;
/// What ends a QML file's name.
const EXT: &str = ".qml";
/// What quotes a text key.
const QUOTE: char = '"';
/// What names a block's QML id.
const ID: &str = "id:";
/// Window types whose title a hub view or their maker hands them, by the key it reads.
const GIVEN: [(&str, &str); 9] = [
    ("ClipBox", WINDOW_CLIP),
    ("QuickFillBox", WINDOW_FILL),
    ("VoiceBar", WINDOW_VOICE),
    ("VoiceBox", WINDOW_VOICE_PANEL),
    ("SelChips", SEL_NAME),
    ("Catch", CATCH),
    ("Edge", WINDOW_OUTLINE),
    ("SnipSide", WINDOW_SNIP_EDGE),
    ("SnipOverlay", WINDOW_SNIP_SCREEN),
];
/// Windows with no title, each with why no capture can show it.
const UNTITLED: [(&str, &str); 1] = [(
    "TrayPicture",
    "drawn at opacity 0, only to grab the tray icon's picture",
)];
/// Window types the check must find, so a broken reader cannot pass it: Snip's capture windows, editor, question and
/// panel among them.
const MUST: [&str; 14] = [
    "KbWindow",
    "Glow",
    "PanelGlow",
    "Edge",
    "SnipOverlay",
    "SnipBar",
    "SnipHint",
    "SnipStrip",
    "SnipCount",
    "SnipRecord",
    "SnipSide",
    "SnipEditor",
    "SnipAsk",
    "SnipPanel",
];

/// A QML file's type: its name, the type its root comes from, its lines and its root's line.
struct Type<'a> {
    name: &'a str,
    base: &'a str,
    lines: Vec<&'a str>,
    at: usize,
}

/// The type a block's first line makes: the word before its trailing `{`.
fn made(line: &str) -> Option<&str> {
    line.trim().strip_suffix('{')?.split_whitespace().last()
}

/// The spaces before `line`'s text.
fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// QML file `file` with `text` as a type, or `None` without a root.
fn typed<'a>(file: &'a str, text: &'a str) -> Option<Type<'a>> {
    let top = root(text)?;
    let lines: Vec<&str> = text.lines().collect();
    let at = lines.iter().position(|l| l.trim() == top)?;
    let (name, base) = (file.strip_suffix(EXT)?, made(top)?);
    Some(Type {
        name,
        base,
        lines,
        at,
    })
}

/// Each QML file of `files` as a type.
fn types(files: &[(String, String)]) -> Vec<Type<'_>> {
    files.iter().filter_map(|(f, t)| typed(f, t)).collect()
}

/// The names of the types that make a window, `Window` included.
fn windows<'a>(types: &[Type<'a>]) -> BTreeSet<&'a str> {
    let mut found = BTreeSet::from([WINDOW]);
    while let Some(t) = types
        .iter()
        .find(|t| found.contains(t.base) && !found.contains(t.name))
    {
        found.insert(t.name);
    }
    found
}

/// The title bindings directly inside the block that starts at line `at`.
fn bound<'a>(lines: &[&'a str], at: usize) -> Vec<&'a str> {
    let inner = indent(lines[at]) + STEP;
    let block = lines[at + 1..]
        .iter()
        .take_while(|l| l.trim().is_empty() || indent(l) >= inner);
    let titled = block.filter(|l| indent(l) == inner && l.trim().starts_with(TITLE));
    titled.map(|l| l.trim()).collect()
}

/// The title bindings of window type `name`'s nearest type that binds one, none up to `Window`.
fn own<'a>(types: &[Type<'a>], mut name: &'a str) -> Vec<&'a str> {
    while let Some(t) = types.iter().find(|t| t.name == name) {
        let b = bound(&t.lines, t.at);
        if !b.is_empty() {
            return b;
        }
        name = t.base;
    }
    Vec::new()
}

/// The title bindings at each place a file makes window type `name`; one that copies the file root's bound title, as a
/// shadow's window or tip does, is checked as the root's own bindings.
fn sites<'a>(types: &[Type<'a>], name: &str) -> Vec<Vec<&'a str>> {
    let each = types.iter().flat_map(|t| {
        let at = (0..t.lines.len()).filter(move |&i| i != t.at && made(t.lines[i]) == Some(name));
        let copy = root_id(t).map(|id| format!("{TITLE} {id}.title"));
        let root = bound(&t.lines, t.at);
        at.map(move |i| {
            let b = bound(&t.lines, i);
            let copies = !root.is_empty() && copy.as_deref().is_some_and(|c| b == [c]);
            if copies { root.clone() } else { b }
        })
    });
    each.collect()
}

/// The QML id of type `t`'s root block, if it binds one.
fn root_id<'a>(t: &Type<'a>) -> Option<&'a str> {
    let inner = indent(t.lines[t.at]) + STEP;
    let block = t.lines[t.at + 1..]
        .iter()
        .take_while(|l| l.trim().is_empty() || indent(l) >= inner);
    let id = block
        .filter(|l| indent(l) == inner)
        .find_map(|l| l.trim().strip_prefix(ID))?;
    Some(id.trim())
}

/// Why window type `name`'s title `binding` is not from the lists, or `None` when it is.
fn off(name: &str, binding: &str, listed: &BTreeSet<&str>) -> Option<String> {
    let keys: Vec<&str> = binding
        .split(QUOTE)
        .skip(1)
        .step_by(2)
        .filter(|k| !k.is_empty())
        .collect();
    if keys.is_empty() {
        let given = GIVEN.iter().any(|(n, k)| *n == name && listed.contains(k));
        return (!given)
            .then(|| format!("{name} takes its title from `{binding}`; name its key in GIVEN"));
    }
    let off = keys.into_iter().find(|k| !listed.contains(k))?;
    Some(format!(
        "{name} takes its title from {off}, which is not on the lists"
    ))
}

/// What is wrong with the titles of the windows `files` make, one line each; none when all are from the lists.
fn wrong(files: &[(String, String)]) -> Vec<String> {
    let types = types(files);
    let listed: BTreeSet<&str> = titles().collect();
    let mut wrong = Vec::new();
    for name in windows(&types) {
        if UNTITLED.iter().any(|(n, _)| *n == name) {
            continue;
        }
        let (own, sites) = (own(&types, name), sites(&types, name));
        let titled = !own.is_empty() || (!sites.is_empty() && sites.iter().all(|s| !s.is_empty()));
        if !titled && (name != WINDOW || !sites.is_empty()) {
            wrong.push(format!("{name} has no title"));
        }
        let bindings = own.iter().chain(sites.iter().flatten());
        wrong.extend(bindings.filter_map(|b| off(name, b, &listed)));
    }
    wrong
}

/// A QML file named after type `name`, with `text`.
fn file(name: &str, text: &str) -> (String, String) {
    (format!("{name}{EXT}"), text.to_owned())
}

#[test]
fn every_window_of_ours_takes_its_title_from_the_lists() {
    let files = qml_files();
    let found = windows(&types(&files));
    assert!(MUST.iter().all(|m| found.contains(m)), "found {found:?}");
    assert_eq!(wrong(&files), Vec::<String>::new());
}

#[test]
fn an_untitled_window_or_one_titled_off_the_lists_fails() {
    let bare = [file("Pane", "Window {\n    flags: Qt.Tool\n}")];
    assert_eq!(wrong(&bare), ["Pane has no title"]);
    let off = [file(
        "Pane",
        "Window {\n    title: res.tr(\"window.nope\")\n}",
    )];
    let why = "Pane takes its title from window.nope, which is not on the lists";
    assert_eq!(wrong(&off), [why]);
    let based = [file("Pane", "Window {\n}"), file("Shade", "Pane {\n}")];
    assert_eq!(wrong(&based), ["Pane has no title", "Shade has no title"]);
    let given = [file("Pane", "Window {\n    title: view.name\n}")];
    assert_eq!(wrong(&given).len(), 1, "a handed title names its key");
}

#[test]
fn a_window_titled_by_its_base_or_where_it_is_made_passes() {
    let tip = "    title: res.tr(\"window.tip\")\n";
    let based = [
        file("Pane", &format!("Window {{\n{tip}}}")),
        file("Shade", "Pane {\n}"),
    ];
    assert!(wrong(&based).is_empty(), "{:?}", wrong(&based));
    let host = format!("Item {{\n    Pane {{\n    {tip}    }}\n}}");
    let made = [file("Pane", "Window {\n}"), file("Host", &host)];
    assert!(wrong(&made).is_empty(), "{:?}", wrong(&made));
    let copy = |key: &str| {
        format!(
            "Window {{\n    id: shade\n    title: res.tr(\"{key}\")\n    Window {{\n        title: shade.title\n    }}\n}}"
        )
    };
    let copied = [file("Shade", &copy("window.tip"))];
    assert!(
        wrong(&copied).is_empty(),
        "a window copying its root's title: {:?}",
        wrong(&copied)
    );
    let off = [file("Shade", &copy("window.nope"))];
    assert_eq!(
        wrong(&off).len(),
        2,
        "the root's key is checked, and its copy's: {:?}",
        wrong(&off)
    );
}
