// Each key makes only the part its kind shows: character legends, the space bar text, a named key's icon or label, or
// the language key in its design. A part swaps when the kind or the design changes, and still plays its motion.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 420
    height: 240

    readonly property var res: ({
        fam: { latin: N.lk.look.latin_fonts[0], arabic: N.lk.look.arabic_fonts[0], mark: N.lk.look.mark_fonts[0], ui: N.lk.look.latin_fonts[0] },
        icons: Assets.assets.icons,
        ext: Assets.assets.ext,
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k,
        space: parts => parts.join(" · "),
        noteTip: (name, note) => name + "\n" + note
    })
    // The parts a key may hold, by type.
    readonly property var parts: ["CharLegends", "SpaceText", "NamedFace", "LangFace"]
    // Arabic, reached by a step forward, so the carousel turns.
    readonly property var stepped: Object.assign({}, Face.states.ar.lang, { turn: 1 })

    Component { id: keyC; Key {} }
    Component { id: faceC; KeyText {} }

    TestCase {
        name: "keyparts"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Board key `id` in the English state, with the language key in `design` (the board's when not given).
        function makeKey(id, design) {
            const kv = Face.board.keys.find(k => k.id === id);
            return createTemporaryObject(keyC, root, { kv: kv, ks: Face.states.en.keys[id], lk: N.lk, lang: Face.states.en.lang, design: design || Face.board.lang_key, res: root.res, s: 1 });
        }
        // Every item under `item` for which `ok` holds.
        function all(item, ok, found) {
            found = found || [];
            if (ok(item))
                found.push(item);
            for (let i = 0; i < item.children.length; i++)
                all(item.children[i], ok, found);
            return found;
        }
        // The QML type of `o`, such as `Icon`.
        function type(o) {
            return String(o).replace(/(_QML.*)?\(.*$/, "");
        }
        // The types under `item` that are key parts or icons.
        function made(item) {
            const wanted = root.parts.concat(["Icon", "LangCycle"]);
            return all(item, o => wanted.indexOf(type(o)) >= 0).map(type);
        }
        // The texts under `item` that have something to show.
        function texts(item) {
            return all(item, o => o.font !== undefined && o.text !== undefined && o.text !== "").map(o => o.text);
        }

        function test_a_character_key_holds_only_its_legends() {
            const k = makeKey("c-0x0002");
            compare(made(k), ["CharLegends"], "no icon, space text or language row");
            compare(texts(k).sort(), ["!", "1"], "its main and Shift legends");
        }

        function test_each_kind_holds_only_its_own_part() {
            compare(made(makeKey("space")), ["SpaceText"]);
            compare(texts(makeKey("space")), [root.res.space(Face.states.en.lang.space)]);
            compare(made(makeKey("bksp")), ["NamedFace", "Icon"], "an icon key");
            compare(texts(makeKey("bksp")), [], "and no label");
            compare(made(makeKey("esc")), ["NamedFace"], "a label key makes no icon");
            compare(texts(makeKey("esc")), [En.texts["label.esc"]]);
            compare(made(makeKey("lang", "carousel")), ["LangFace"]);
            compare(made(makeKey("lang", "cycle")), ["LangFace", "LangCycle", "Icon"]);
            compare(made(makeKey("lang", "list")), ["LangFace", "Icon"]);
        }

        // A named face given an icon and a label shows both side by side, the label fitted to the room the icon leaves.
        function test_a_named_face_can_show_an_icon_and_a_label() {
            const key = makeKey("bksp");
            const props = { kv: key.kv, ks: key.ks, lk: N.lk, lang: key.lang, design: key.design, res: root.res, s: 1 };
            const f = createTemporaryObject(faceC, root, Object.assign(props, { ink: key.ink, icon: key.icon, label: "Back", width: key.width, height: key.height }));
            compare(made(f), ["NamedFace", "Icon"]);
            const icon = all(f, o => type(o) === "Icon")[0];
            const legend = all(f, o => o.text === "Back")[0];
            verify(legend.mapToItem(f, 0, 0).x >= icon.mapToItem(f, 0, 0).x + icon.width, "the label sits right of the icon");
            fuzzyCompare(legend.width, f.item.labelRoom - icon.width - N.lk.shape.legend.gap_px, 1e-9, "the label's room");
        }

        // Edge cases 3 and 4: the board can turn a key into another kind, and the language key can change its design.
        function test_a_key_that_changes_kind_or_design_shows_the_new_part() {
            const k = makeKey("c-0x0002");
            k.ks = Face.states.en.keys.bksp;
            k.kv = Face.board.keys.find(c => c.id === "bksp");
            compare(made(k), ["NamedFace", "Icon"], "the character key became Backspace");
            const l = makeKey("lang", "carousel");
            const cur = Face.states.en.lang.cur.short;
            compare(texts(l).filter(t => t === cur).length, 1, "the carousel names the current layout");
            l.design = "cycle";
            compare(made(l), ["LangFace", "LangCycle", "Icon"], "the cycle");
            l.design = "list";
            compare(made(l), ["LangFace", "Icon"], "the list");
            compare(texts(l), [cur]);
            l.design = "carousel";
            compare(made(l), ["LangFace"], "the carousel again");
            compare(texts(l).length, 3, "previous, current and next");
        }

        // Edge case 12: after a language change, the main legend and the space bar text rise in, and the carousel turns.
        function test_a_language_change_raises_the_legends_and_turns_the_carousel() {
            for (const id of ["c-0x0002", "space"]) {
                const k = makeKey(id);
                const face = all(k, o => o.isChar !== undefined)[0];
                const legend = all(k, o => type(o) === "Rise")[0];
                k.lang = root.stepped;
                verify(face.clip, id + " clips while its legend rises");
                verify(legend.opacity < 1, id + " fades its legend in");
                tryVerify(() => !face.clip && legend.opacity === 1, 1000, id + " rose in");
            }
            const l = makeKey("lang", "carousel");
            const turn = () => all(l, o => o.transform !== undefined && o.transform.length > 0 && !!o.transform[0].x);
            l.lang = root.stepped;
            verify(turn().length > 0, "the carousel turns");
            tryVerify(() => turn().length === 0, 1000, "and comes to rest");
        }
    }
}
