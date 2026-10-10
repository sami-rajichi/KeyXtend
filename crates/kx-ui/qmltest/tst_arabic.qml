// The owner's Arabic (102) AZERTY keys over the face's golden states: every legend the face flags as marks sits on a
// dotted circle in the theme's mark font, in each corner and with AltGr latched; readers hear a mark key's marks; letters
// stay centred; corner legends keep one margin in from each theme's face; and a key that types nothing shows no letter.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-ar.js" as Ar
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-dolch-light.js" as D
import "fonts.js" as Fonts

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
    // The bundled fonts loaded so far, from where the texts test lays them out.
    property int fontsIn: 0

    Instantiator {
        model: Assets.assets.fonts
        delegate: FontLoader {
            required property string modelData
            source: Qt.resolvedUrl(Assets.root + "/" + modelData)
            onStatusChanged: if (status === FontLoader.Ready) root.fontsIn++
        }
    }

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

        // Look `lk`'s own first fonts, as the keyboard window picks them when the system has them.
        function resOf(lk) {
            const l = lk.look;
            const fam = { latin: l.latin_fonts[0], arabic: l.arabic_fonts[0], mark: l.mark_fonts[0], ui: l.latin_fonts[0] };
            return Object.assign({}, root.res, { fam: fam });
        }
        // How far key `id`'s corner legends sit in from the face its look draws, at size 1: [side, baseline, legend] for
        // the top left and then the bottom right, undefined for a corner with no legend.
        function margins(state, id, lk) {
            const key = makeKey(state, id, lk, resOf(lk));
            // The cap's own face, the first one drawn; the inner shading draws it again, moved.
            const cap = all(key, o => o.sculpted !== undefined && o.grad !== undefined)[0];
            const fills = all(cap, o => o.base !== undefined && o.g !== undefined && o.visible);
            const f = fills[fills.length - 1];
            const c = f.mapToItem(key, 0, 0);
            const shown = s => all(key, o => o.font !== undefined && o.visible && s !== "" && o.text === s)[0];
            const [lt, rb] = [shown(state.keys[id].second), shown(state.keys[id].third)];
            const at = (l, x) => l.mapToItem(key, x, l.baselineOffset);
            const top = lt && [at(lt, 0).x - c.x, at(lt, 0).y - c.y, lt];
            const bottom = rb && [c.x + f.width - at(rb, rb.width).x, c.y + f.height - at(rb, 0).y, rb];
            return [top, bottom];
        }

        // Every theme leaves one margin between its face and a corner legend, Dolch's lit top too (owner 2026-10-10);
        // a mark in a taller font sits on the Latin baseline, as a Latin legend in its corner does.
        function test_corner_legends_sit_the_same_way_in_from_each_face_data() {
            return [N, E, D].map(t => ({ tag: t.lk.look.id, lk: t.lk }));
        }
        function test_corner_legends_sit_the_same_way_in_from_each_face(row) {
            const [lk, state] = [row.lk, Face.states.ar];
            Fonts.need([lk.look.latin_fonts[0], lk.look.mark_fonts[0]], skip);
            tryCompare(root, "fontsIn", Assets.assets.fonts.length, 5000, "the bundle is laid out by the texts test");
            const [lg, fm] = [lk.shape.legend, lk.look.font_metrics];
            const find = ok => Object.keys(state.keys).find(id => ok(state.keys[id]));
            // A Latin key with both corners, and a letter with marks bottom right.
            const ids = [find(k => k.second !== "" && k.third !== "" && !k.third_mark), find(k => k.third_mark)];
            verify(ids.every(id => !!id), "the cases this test is for");
            for (const id of ids) {
                const [top, bottom] = margins(state, id, lk);
                if (top) {
                    fuzzyCompare(top[0], lg.corner_px[1], 0.5, id + " top left, side");
                    fuzzyCompare(top[1], lg.corner_px[0] + fm[0] * top[2].font.pixelSize, 0.5, id + " top left, baseline");
                }
                fuzzyCompare(bottom[0], lg.corner_px[1], 0.5, id + " bottom right, side");
                fuzzyCompare(bottom[1], lg.corner_px[0] + fm[1] * bottom[2].font.pixelSize, 0.5, id + " bottom right, baseline");
            }
        }

        // A key that types nothing with Shift (س) or AltGr (ف) shows no letter, as Windows' keyboard does, and readers hear
        // its empty name; only an AltGr corner may stay.
        function test_an_empty_key_shows_no_letter_and_readers_hear_it_is_empty() {
            for (const name of ["ar-shift", "ar-altgr"]) {
                const state = Face.states[name];
                const ids = Object.keys(state.keys).filter(id => state.keys[id].blank);
                verify(ids.length > 0, name + ": the case this test is for");
                for (const id of ids) {
                    const ks = state.keys[id];
                    const key = makeKey(state, id);
                    const shown = all(key, o => o.font !== undefined && o.visible && o.text !== "").map(o => o.text);
                    compare(shown, ks.third ? [ks.third] : [], name + " " + id);
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
