// The owner's Arabic (102) AZERTY keys over the face's golden states: every legend the face flags as marks sits on a
// dotted circle in the theme's mark font, with Shift and with AltGr latched; readers hear a mark key's marks; a letter
// stands alone and centred; and a key that types nothing shows nothing.
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
    // The Arabic states that show marks: with Shift and with AltGr latched.
    readonly property var states: [Face.states["ar-shift"], Face.states["ar-altgr"]]

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

        function test_a_mark_sits_on_a_circle_in_the_mark_font_with_shift_and_with_altgr() {
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
            const id = markKeys(Face.states["ar-shift"])[0];
            const marks = makeKey(Face.states["ar-shift"], id, D.lk, res);
            compare(texts(marks, true).map(t => t.font.family), [fam.mark], "the Shift marks");
            const letter = makeKey(Face.states.ar, id, D.lk, res);
            compare(texts(letter, false).map(t => t.font.family), [fam.arabic], "the letter");
        }

        // The owner wants each Arabic letter alone and centred (2026-10-10): no corner legend crowds it, plain or with
        // Shift, and every letter sits at the same height, as ف does.
        function test_an_arabic_letter_stands_alone_and_centred() {
            for (const name of ["ar", "ar-shift"]) {
                const state = Face.states[name];
                const ids = Object.keys(state.keys).filter(id => state.keys[id].main_ar && !state.keys[id].main_mark);
                verify(ids.length > 0, name + ": the case this test is for");
                const offsets = ids.map(id => {
                    const shown = all(makeKey(state, id, D.lk), o => o.font !== undefined && o.visible && o.text !== "");
                    compare(shown.map(o => o.text), [state.keys[id].main], name + " " + id);
                    return shown[0].anchors.verticalCenterOffset;
                });
                verify(offsets.every(o => o === offsets[0]), name + ": " + offsets);
            }
        }

        // A key that types nothing with Shift (س) or AltGr (ف) shows nothing, as Windows' keyboard does, and readers
        // hear its empty name.
        function test_an_empty_key_shows_nothing_and_readers_hear_it_is_empty() {
            for (const name of ["ar-shift", "ar-altgr"]) {
                const state = Face.states[name];
                const ids = Object.keys(state.keys).filter(id => state.keys[id].blank);
                verify(ids.length > 0, name + ": the case this test is for");
                for (const id of ids) {
                    const ks = state.keys[id];
                    const key = makeKey(state, id);
                    const shown = all(key, o => o.font !== undefined && o.visible && o.text !== "").map(o => o.text);
                    compare(shown, [], name + " " + id);
                    compare(key.Accessible.name, Ar.texts[ks.blank], name + " " + id);
                }
            }
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
