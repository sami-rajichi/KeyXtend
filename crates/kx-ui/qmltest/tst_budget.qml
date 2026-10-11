// Each key makes only what it draws: every kind of key makes exactly today's items in each look, high contrast and dark
// too, and each shadow layer is one drawn item, not a hidden twin.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-dolch-light.js" as D
import "../../kx-theme/fixtures/look-native-light-contrast.js" as H
import "../../kx-theme/fixtures/look-native-dark.js" as Nd

Item {
    id: root
    width: 420
    height: 240

    readonly property var res: Board.resOf(N.lk, En)
    // The looks, by name.
    readonly property var looks: ({ native: N.lk, et66: E.lk, dolch: D.lk, contrast: H.lk, dark: Nd.lk })
    // Today's items per key (2026-10-11), by look and kind. Any change fails, so a hidden part cannot creep back and a
    // change is a conscious update here.
    readonly property var budget: ({
        native: { char: 26, space: 24, icon: 24, label: 24, lock: 24, lit: 29, carousel: 28, cycle: 34, list: 26 },
        et66: { char: 29, space: 27, icon: 27, label: 27, lock: 27, lit: 29, carousel: 31, cycle: 37, list: 29 },
        dolch: { char: 21, space: 19, icon: 19, label: 19, lock: 19, lit: 26, carousel: 23, cycle: 29, list: 21 },
        contrast: { char: 27, space: 25, icon: 25, label: 25, lock: 25, lit: 25, carousel: 29, cycle: 35, list: 27 },
        dark: { char: 26, space: 24, icon: 24, label: 24, lock: 24, lit: 29, carousel: 28, cycle: 34, list: 26 }
    })
    // Each kind's board key, the language key's design and the state, where they matter.
    readonly property var kinds: ({
        char: { id: "c-0x0002" }, space: { id: "space" }, icon: { id: "bksp" }, label: { id: "esc" },
        lock: { id: "caps" }, lit: { id: "caps", st: "en-caps" },
        carousel: { id: "lang", design: "carousel" }, cycle: { id: "lang", design: "cycle" }, list: { id: "lang", design: "list" }
    })

    Component { id: keyC; Key {} }

    TestCase {
        name: "budget"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Board key `id` over look `lk`, with the language key in `design` and in state `st` (English when not given).
        function makeKey(id, lk, design, st) {
            const kv = Face.board.keys.find(k => k.id === id);
            const ks = Face.states[st || "en"].keys[id];
            return createTemporaryObject(keyC, root, { kv: kv, ks: ks, lk: lk, lang: Face.states.en.lang, design: design || Face.board.lang_key, res: root.res, s: 1 });
        }
        // How many items `item` makes, itself included.
        function count(item) {
            return Board.all(item, () => true).length;
        }
        // The QML type of `o`, such as `Shade`.
        function type(o) {
            return String(o).replace(/(_QML.*)?\(.*$/, "");
        }

        function test_each_key_makes_its_budget() {
            for (const look in root.looks)
                for (const kind in root.kinds) {
                    const k = root.kinds[kind];
                    const n = count(makeKey(k.id, root.looks[look], k.design, k.st));
                    compare(n, root.budget[look][kind], look + " " + kind);
                }
        }

        // A sharp layer is one box and a soft one one shadow, never both with one hidden; high contrast has only an
        // inner line, so no shade.
        function test_each_shadow_layer_is_one_drawn_item() {
            for (const look in root.looks) {
                const k = makeKey("c-0x0002", root.looks[look]);
                const shades = Board.all(k, o => type(o) === "Shade");
                const outer = k.p.key_shadow.filter(l => !l.inset).length;
                compare(shades.length, outer, look + ": one shade per outer layer");
                for (const s of shades)
                    verify(s.children.length === 1 && s.children[0].visible, look + ": " + s.children.map(type));
            }
        }
    }
}
