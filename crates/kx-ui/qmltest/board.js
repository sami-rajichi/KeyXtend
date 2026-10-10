.pragma library
.import "fixtures/assets.js" as Assets
// What the keyboard window hands its keys for a look and a UI language, and the bundled fonts, shared by the board
// tests (tst_corners, tst_order, tst_pictures) so their copies cannot drift.

// Every item under `item` for which `ok` holds.
function all(item, ok, found) {
    found = found || [];
    if (ok(item))
        found.push(item);
    for (let i = 0; i < item.children.length; i++)
        all(item.children[i], ok, found);
    return found;
}

// The space bar's text as the hub joins `parts`: each layout after the language through `lang.space` in `texts`,
// whose variables the texts test leaves as Fluent shows them, as `{$name}`.
function spaceOf(texts) {
    const at = name => "{$" + name + "}";
    const join = (lang, layout) => texts["lang.space"].replace(at("lang"), lang).replace(at("layout"), layout);
    return parts => parts.length === 0 ? "" : parts.slice(1).reduce(join, parts[0]);
}

// Fonts, icons and texts for look `lk` in UI language `ui` (a ui-*.js fixture), as the keyboard window picks them:
// each family the first Qt has, else the last, and the UI font by the language's direction.
function resOf(lk, ui) {
    const have = Qt.fontFamilies();
    const pick = list => list.find(f => have.indexOf(f) >= 0) || list[list.length - 1];
    const [latin, arabic] = [pick(lk.look.latin_fonts), pick(lk.look.arabic_fonts)];
    return {
        fam: { latin: latin, arabic: arabic, mark: pick(lk.look.mark_fonts), ui: ui.lang.rtl ? arabic : latin },
        icons: Qt.resolvedUrl(Assets.root + "/" + Assets.assets.icons),
        ext: Assets.assets.ext,
        tr: k => ui.texts[k] !== undefined ? ui.texts[k] : k,
        bar: id => id,
        dpad: id => id,
        uid: (group, id) => group + "." + id,
        space: spaceOf(ui.texts),
        noteTip: (name, note) => name + "\n" + note
    };
}

// Loads every bundled font, from where the texts test lays them out, under `parent`; see `loaded`.
function loadFonts(parent) {
    const qml = "import QtQuick\nFontLoader { readonly property bool ready: status === FontLoader.Ready }";
    return Assets.assets.fonts.map(f => {
        const loader = Qt.createQmlObject(qml, parent, "board.js");
        loader.source = Qt.resolvedUrl(Assets.root + "/" + f);
        return loader;
    });
}

// Whether every font `loadFonts` gave is ready.
function loaded(fonts) {
    return fonts.every(f => f.ready);
}
