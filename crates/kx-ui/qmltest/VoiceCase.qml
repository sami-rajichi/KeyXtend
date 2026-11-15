// What the Voice panel's tests share: the panel over stand-in Ui and Voice objects in a language and a look, and its
// parts by gate id, by what readers hear, as rows and as the footer's buttons. Any warning fails a test.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N

TestCase {
    id: kase

    // The item the panels are made under.
    required property Item host

    readonly property var langs: ({ en: En, fr: Fr, ar: Ar })
    property Component uiC: Component {
        FakeUi {}
    }
    property Component voiceC: Component {
        FakeVoice {}
    }
    property Component boxC: Component {
        VoiceBox {}
    }

    when: windowShown

    function init() {
        failOnWarning(/./);
    }
    // The panel showing panel view `v` (a whole view from Kit.view) in language `tag` and look `lk` (Native light by
    // default), laid out and shown.
    function show(v, tag, lk) {
        const t = tag || "en";
        const look = lk || N.lk;
        const ui = createTemporaryObject(uiC, host, { texts: langs[t].texts, lang: langs[t].lang, look: look });
        const box = createTemporaryObject(boxC, host, {
            ui: ui, voice: createTemporaryObject(voiceC, host), lk: look, res: Board.resOf(look, langs[t]), s: 1,
            view: v.panel, round: v.round, rows: En.settings.pageRows, ar: t === "ar"
        });
        Pointer.through([box]);
        box.open();
        verify(waitForPolish(box), "laid out");
        return box;
    }
    // The panel showing the hub's view `name`.
    function make(name, tag, lk) {
        return show(Kit.view(tag || "en", name), tag, lk);
    }
    // The one shown part under `item` (the panel's content by default) whose gate id is `id`.
    function one(box, id, item) {
        const hit = Kit.only(Board.all(item || box.contentItem, o => o.visible && o.Accessible.id === id));
        verify(hit, id + " shows once");
        return hit;
    }
    // The shown parts under `item` readers hear as `name`.
    function heard(item, name) {
        return Board.all(item, o => o.visible && !o.Accessible.ignored && o.Accessible.name === name);
    }
    // The rows readers find, top to bottom.
    function rows(box) {
        return Board.all(box.contentItem, o => o.visible && o.Accessible.role === Accessible.ListItem && !o.Accessible.ignored);
    }
    // The shown buttons under `item`, left to right on screen.
    function buttons(item) {
        const all = Board.all(item, o => o.visible && o.Accessible.role === Accessible.Button);
        return all.sort((a, b) => a.mapToItem(null, 0, 0).x - b.mapToItem(null, 0, 0).x);
    }
    // The marks a row shows, left to right.
    function marks(row) {
        const all = Board.all(row, o => o.visible && o.markId !== undefined);
        return all.sort((a, b) => a.mapToItem(null, 0, 0).x - b.mapToItem(null, 0, 0).x);
    }
    // The clicks sent to the Voice object, each as `act kind id arg round`.
    function acts(box) {
        return box.voice.calls;
    }
}
