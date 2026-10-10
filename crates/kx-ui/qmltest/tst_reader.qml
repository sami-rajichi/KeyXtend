// What screen readers and UI Automation get from keys and the D-pad: board ids, names, and the press and toggle actions.
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
        icons: Qt.resolvedUrl(Assets.root + "/" + Assets.assets.icons),
        ext: Assets.assets.ext,
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k,
        bar: id => En.texts["bar." + id],
        uid: (group, id) => group + "." + id,
        dpad: id => En.texts["dpad." + id],
        space: parts => parts.join(" · "),
        noteTip: (name, note) => name + "\n" + note
    })

    Component { id: keyC; Key {} }
    Component { id: padC; DPad {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "reader"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Board key `id` at size 1, with its English state.
        function makeKey(id) {
            const kv = Face.board.keys.find(k => k.id === id);
            return createTemporaryObject(keyC, root, { kv: kv, ks: Face.states.en.keys[id], lk: N.lk, lang: Face.states.en.lang, design: Face.board.lang_key, res: root.res, s: 1 });
        }
        // The first item under `item`, itself included, for which `ok` holds.
        function find(item, ok) {
            if (ok(item))
                return item;
            for (let i = 0; i < item.children.length; i++) {
                const hit = find(item.children[i], ok);
                if (hit)
                    return hit;
            }
            return null;
        }

        // UI Automation finds each key by this id, so tools aim at keys wherever the top bar puts them.
        function test_every_key_gives_ui_automation_its_board_id() {
            for (const c of Face.board.keys)
                compare(makeKey(c.id).Accessible.id, c.id);
        }

        // Screen readers press a key, or toggle a latch key such as Shift; both click it, and a dimmed key ignores both.
        function test_a_reader_press_or_toggle_clicks_a_key() {
            for (const [id, clicks] of [["shift", 2], ["power", 0]]) {
                const key = makeKey(id);
                const spy = createTemporaryObject(spyC, root, { target: key, signalName: "hit" });
                key.Accessible.pressAction();
                key.Accessible.toggleAction();
                compare(spy.count, clicks, id);
            }
        }

        function test_a_working_dpad_names_each_arrow_and_a_dimmed_one_is_one_button() {
            const box = Object.assign({}, Face.board.dpad, { dim: false, tip: null });
            const pad = createTemporaryObject(padC, root, { box: box, lk: N.lk, res: root.res, s: 1 });
            compare(pad.Accessible.role, Accessible.Grouping, "a group of arrows");
            for (const id of Object.keys(N.lk.icons.dpad).filter(id => id !== "stop"))
                verify(find(pad, o => o.Accessible.name === En.texts["dpad." + id] && !o.Accessible.ignored), id);
            // UI Automation finds each arrow and Stop by its text key, whatever the language.
            for (const id of Object.keys(N.lk.icons.dpad))
                compare(find(pad, o => o.Accessible.id === "dpad." + id).Accessible.name, En.texts["dpad." + id], id);
            compare(pad.Accessible.id, box.name, "the pad");
            const dim = createTemporaryObject(padC, root, { box: Face.board.dpad, lk: N.lk, res: root.res, s: 1 });
            compare(dim.Accessible.role, Accessible.Button);
            compare(dim.Accessible.description, En.texts["tip.later"]);
            verify(!find(dim, o => o !== dim && o.Accessible.role === Accessible.Button && !o.Accessible.ignored && o.visible), "no arrow on its own");
        }
    }
}
