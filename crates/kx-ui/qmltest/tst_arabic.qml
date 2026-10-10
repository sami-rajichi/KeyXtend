// The owner's Arabic (102) AZERTY keys over the face's golden states: every legend the face flags as marks sits on a
// dotted circle in the theme's mark font, in each corner and with AltGr latched, and readers hear a mark key's marks.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-ar.js" as Ar
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-dolch-light.js" as D

Item {
    id: root
    width: 420
    height: 240

    readonly property var res: ({
        fam: { latin: N.lk.look.latin_fonts[0], arabic: N.lk.look.arabic_fonts[0], mark: N.lk.look.mark_fonts[0], ui: N.lk.look.arabic_fonts[0] },
        icons: Qt.resolvedUrl(Assets.root + "/" + Assets.assets.icons),
        ext: Assets.assets.ext,
        tr: k => Ar.texts[k] !== undefined ? Ar.texts[k] : k,
        bar: id => id,
        dpad: id => id,
        uid: (group, id) => group + "." + id,
        space: parts => parts.join(" · ")
    })
    // The Arabic states: as typed, and with AltGr latched.
    readonly property var states: [Face.states.ar, Face.states["ar-altgr"]]

    Component { id: keyC; Key {} }

    TestCase {
        name: "arabic"
        when: windowShown

        // Drawn before a mark so it has a base, as the face draws it.
        readonly property string circle: "◌"

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
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
        // Character key `id` as `state` shows it, in look `lk` (Native by default) with fonts `res`.
        function makeKey(state, id, lk, res) {
            const kv = Face.board.keys.find(k => k.id === id);
            const props = { kv: kv, ks: state.keys[id], lk: lk || N.lk, lang: state.lang, design: Face.board.lang_key };
            return createTemporaryObject(keyC, root, Object.assign(props, { res: res || root.res, s: 1 }));
        }
        // The character keys the face flags as holding marks on their circle, in either state.
        function markKeys(state) {
            const parts = ["main", "second", "third"];
            return Object.keys(state.keys).filter(id => parts.some(p => state.keys[id][p + "_mark"]));
        }
        // The texts `key` shows: marks on their circle, or the rest.
        function texts(key, marks) {
            return all(key, o => o.font !== undefined && o.visible && o.text !== "" && o.text.startsWith(circle) === marks);
        }

        function test_a_mark_sits_on_a_circle_in_the_mark_font_in_every_corner() {
            compare(root.states.map(s => markKeys(s).length > 0), [true, true], "the case this test is for");
            for (const state of root.states)
                for (const id of markKeys(state)) {
                    const key = makeKey(state, id);
                    const drawn = texts(key, true);
                    verify(drawn.length > 0, id + ": a mark is drawn");
                    for (const t of drawn)
                        compare(t.font.family, root.res.fam.mark, id + " " + t.text);
                }
        }

        function test_dolch_draws_marks_in_its_mark_font_and_letters_in_rubik() {
            verify(D.lk.look.mark_fonts[0] !== D.lk.look.arabic_fonts[0], "Dolch names its own mark font");
            const fam = { latin: D.lk.look.latin_fonts[0], arabic: D.lk.look.arabic_fonts[0], mark: D.lk.look.mark_fonts[0] };
            const res = Object.assign({}, root.res, { fam: Object.assign({ ui: fam.arabic }, fam) });
            const key = makeKey(Face.states.ar, markKeys(Face.states.ar)[0], D.lk, res);
            compare(texts(key, true).map(t => t.font.family), [fam.mark], "the AltGr marks");
            compare(texts(key, false).map(t => t.font.family), [fam.arabic], "the letter");
        }

        // The owner wants every letter centred, also over an AltGr legend (2026-10-10), so ض sits as ف does.
        function test_an_arabic_letter_over_an_altgr_legend_stays_centred() {
            const state = Face.states.ar;
            const pick = ok => Object.keys(state.keys).find(id => state.keys[id].main_ar && ok(state.keys[id]));
            const offset = id => {
                const key = makeKey(state, id, D.lk);
                return all(key, o => o.font !== undefined && o.text === state.keys[id].main)[0].anchors.verticalCenterOffset;
            };
            const [over, alone] = [pick(k => k.third !== ""), pick(k => k.third === "")];
            verify(over && alone, "the case this test is for");
            compare(offset(over), offset(alone));
        }

        function test_readers_hear_a_mark_key_by_its_marks_names() {
            const state = Face.states["ar-altgr"];
            const id = Object.keys(state.keys).find(k => state.keys[k].marks.length === 2);
            const key = makeKey(state, id);
            const [shadda, vowel] = state.keys[id].marks.map(m => Ar.texts[m]);
            verify(shadda && vowel, "each mark has its Arabic name");
            compare(key.Accessible.name, shadda + Ar.texts["mark.join"] + vowel);
            const letter = makeKey(Face.states.ar, id);
            compare(letter.Accessible.name, Face.states.ar.keys[id].name, "a letter says itself");
        }
    }
}
