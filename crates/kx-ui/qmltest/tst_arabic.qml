// The owner's Arabic (102) AZERTY keys over the face's golden states: every legend the face flags as marks sits on a
// dotted circle in the theme's mark font, in each corner and with AltGr latched; readers hear a mark key's marks; and
// wide letters rise clear of the AltGr legend below them.
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
        // The texts in look `lk`'s own fonts.
        function resOf(lk) {
            const l = lk.look;
            const fam = { latin: l.latin_fonts[0], arabic: l.arabic_fonts[0], mark: l.mark_fonts[0], ui: l.arabic_fonts[0] };
            return Object.assign({}, root.res, { fam: fam });
        }
        // The text `key` shows as `text`, once its legends have risen in.
        function line(key, text) {
            const found = all(key, o => o.font !== undefined && o.visible && o.text === text)[0];
            tryVerify(() => !found.parent.parent.clip, 2000, "the legends rose in");
            return found;
        }
        // The pixels of `shot` that differ from `bare` by half the strongest difference or more, so faint edges count
        // as blank.
        function ink(shot, bare) {
            const px = [];
            for (let y = 0; y < shot.height; y++)
                for (let x = 0; x < shot.width; x++) {
                    const p = shot.pixel(x, y), q = bare.pixel(x, y);
                    px.push([x, y, Math.abs(p.r - q.r) + Math.abs(p.g - q.g) + Math.abs(p.b - q.b)]);
                }
            const most = Math.max(...px.map(p => p[2]));
            return px.filter(p => p[2] >= most / 2);
        }
        // The inks of texts `a` and `b` on `key`, each drawn alone over the bare cap, and the device pixels per px.
        function inks(key, a, b) {
            a.opacity = b.opacity = 0;
            const bare = grabImage(key);
            a.opacity = 1;
            const inkA = ink(grabImage(key), bare);
            [a.opacity, b.opacity] = [0, 1];
            const inkB = ink(grabImage(key), bare);
            a.opacity = 1;
            return [inkA, inkB, bare.width / key.width];
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
            const res = resOf(D.lk);
            const key = makeKey(Face.states.ar, markKeys(Face.states.ar)[0], D.lk, res);
            compare(texts(key, true).map(t => t.font.family), [res.fam.mark], "the AltGr marks");
            compare(texts(key, false).map(t => t.font.family), [res.fam.arabic], "the letter");
        }

        // One row per theme, so a theme whose Windows font this system lacks skips alone (fonts.js).
        function test_a_wide_letter_keeps_a_clear_gap_over_its_altgr_legend_data() {
            return [N, E, D].map(t => ({ tag: t.lk.look.id, lk: t.lk }));
        }
        // The owner saw AltGr marks and the riyal sign touch letters such as ض and س; a clear gap is 1 px of blank cap.
        function test_a_wide_letter_keeps_a_clear_gap_over_its_altgr_legend(row) {
            tryCompare(root, "fontsIn", Assets.assets.fonts.length, 5000, "the bundle is laid out by cargo test -p kx-ui --test texts");
            const look = row.lk.look;
            Fonts.need([look.arabic_fonts[0], look.mark_fonts[0]], skip);
            const state = Face.states.ar;
            const ids = Object.keys(state.keys).filter(id => state.keys[id].main_ar && state.keys[id].third !== "");
            verify(ids.length > 0, "the case this test is for");
            for (const id of ids) {
                const ks = state.keys[id];
                const key = makeKey(state, id, row.lk, resOf(row.lk));
                const [a, b, k] = inks(key, line(key, ks.main), line(key, ks.third));
                let gap = Infinity;
                for (const p of a)
                    for (const q of b)
                        gap = Math.min(gap, Math.hypot(p[0] - q[0], p[1] - q[1]) / k);
                verify(gap >= 1, ks.main + " " + ks.third + ": " + gap.toFixed(2) + " px apart");
                verify(a.every(p => p[1] > 0), ks.main + " stays on its cap");
            }
        }

        function test_only_an_arabic_letter_over_an_altgr_legend_rises() {
            const up = N.lk.shape.legend.arabic_lift_px;
            verify(up > 0, "the token");
            const offset = (state, id) => line(makeKey(state, id), state.keys[id].main).anchors.verticalCenterOffset;
            const [ar, fr] = [Face.states.ar, Face.states.fr];
            const pick = (st, ok) => Object.keys(st.keys).find(id => st.keys[id].main !== undefined && ok(st.keys[id]));
            compare(offset(ar, pick(ar, k => k.main_ar && k.third !== "")), -up, "an Arabic letter over an AltGr legend");
            compare(offset(ar, pick(ar, k => k.main_ar && k.third === "")), 0, "nothing below it");
            compare(offset(fr, pick(fr, k => !k.main_ar && k.third !== "")), 0, "a Latin legend keeps its place");
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
