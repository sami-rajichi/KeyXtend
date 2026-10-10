// Corner legends in every theme, in the bundled fonts: one margin in from the face each theme draws, on the Latin
// baseline whatever font they draw in, and a clear gap from the main legend's ink in each language and latch state.
pragma ComponentBehavior: Bound
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-dolch-light.js" as D
import "board.js" as Board
import "fonts.js" as Fonts

Item {
    id: root
    width: Face.board.plate.w
    height: Face.board.plate.h

    // The states checked, by golden name, each with its layout's language: its texts and direction.
    readonly property var states: [
        { name: "en", ui: En },
        { name: "fr", ui: Fr },
        { name: "ar", ui: Ar },
        { name: "ar-shift", ui: Ar },
        { name: "ar-altgr", ui: Ar }
    ]
    // The bundled fonts, from where the texts test lays them out.
    property var bundled: []

    Component { id: keyC; Key {} }
    // Every key of the board at size `s`, in look `lk` and state `st`.
    Component {
        id: boardC
        Item {
            id: board
            required property var lk
            required property var st
            required property var res
            required property real s
            width: root.width
            height: root.height

            // Cell `kv` at size `s`.
            function scaled(kv) {
                return Object.assign({}, kv, { x: kv.x * s, y: kv.y * s, w: kv.w * s, h: kv.h * s });
            }

            Repeater {
                model: Face.board.keys
                delegate: Key {
                    required property var modelData
                    kv: board.scaled(modelData)
                    ks: board.st.keys[modelData.id]
                    lk: board.lk
                    lang: board.st.lang
                    design: Face.board.lang_key
                    res: board.res
                    s: board.s
                }
            }
        }
    }

    TestCase {
        name: "corners"
        when: windowShown

        // Drawn before a mark so it has a base, as the face draws it.
        readonly property string circle: "◌"
        // The blank cap, in px at size 1, a corner legend's ink keeps from the main legend's ink; it scales with the
        // board, like every size.
        readonly property real clearPx: 1

        // The bundled fonts load once, for every row.
        function initTestCase() {
            root.bundled = Board.loadFonts(root);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // One row per theme, so a theme whose Windows font this system lacks skips alone (fonts.js).
        function themes() {
            return [N, E, D].map(t => ({ tag: t.lk.look.id, lk: t.lk }));
        }
        // Skips without look `lk`'s Windows fonts, and waits for the bundled ones.
        function fonts(lk) {
            Fonts.need([lk.look.latin_fonts[0], lk.look.arabic_fonts[0], lk.look.mark_fonts[0]], skip);
            tryVerify(() => Board.loaded(root.bundled), 5000, "the bundle is laid out by the texts test");
        }
        // Waits until the legends under `item` have risen in.
        function risen(item) {
            const texts = Board.all(item, o => o.isChar === true);
            tryVerify(() => texts.every(o => !o.clip), 2000, "the legends rose in");
        }
        // The line `key` shows as `text`.
        function line(key, text) {
            return Board.all(key, o => o.font !== undefined && o.visible && o.text === text)[0];
        }

        // How far `key`'s corner legends `lt` (top left) and `rb` (bottom right) sit in from the face its look draws,
        // at size 1: [side, baseline] for each.
        function margins(key, lt, rb) {
            // The cap's own face, the first one drawn; the inner shading draws it again, moved.
            const cap = Board.all(key, o => o.sculpted !== undefined && o.grad !== undefined)[0];
            const fills = Board.all(cap, o => o.base !== undefined && o.g !== undefined && o.visible);
            const f = fills[fills.length - 1];
            const c = f.mapToItem(key, 0, 0);
            const at = (l, x) => l.mapToItem(key, x, l.baselineOffset);
            const top = [at(lt, 0).x - c.x, at(lt, 0).y - c.y];
            const bottom = [c.x + f.width - at(rb, rb.width).x, c.y + f.height - at(rb, 0).y];
            return [top, bottom];
        }

        // Every theme leaves one margin between its face and a corner legend, Dolch's lit top too (owner 2026-10-10);
        // a mark in a taller font sits on the Latin baseline, as a Latin legend in its corner does.
        function test_corner_legends_sit_the_same_way_in_from_each_face_data() {
            return themes();
        }
        function test_corner_legends_sit_the_same_way_in_from_each_face(row) {
            const [lk, state] = [row.lk, Face.states.ar];
            const [lg, fm] = [lk.shape.legend, lk.look.font_metrics];
            fonts(lk);
            const id = Object.keys(state.keys).find(k => state.keys[k].second !== "" && state.keys[k].third !== "");
            verify(id, "a key with both corners");
            // The same key with marks in its AltGr corner, as a key with no Arabic letter may show them.
            const marked = Object.assign({}, state.keys[id], { third: circle + "ّ", third_ar: true, third_mark: true });
            for (const ks of [state.keys[id], marked]) {
                const kv = Face.board.keys.find(k => k.id === id);
                const props = { kv: kv, ks: ks, lk: lk, lang: state.lang, design: Face.board.lang_key, s: 1 };
                const key = createTemporaryObject(keyC, root, Object.assign(props, { res: Board.resOf(lk, Ar) }));
                const [lt, rb] = [line(key, ks.second), line(key, ks.third)];
                const [top, bottom] = margins(key, lt, rb);
                const at = ks.third + ": ";
                const [above, below, side] = lg.corner_px;
                fuzzyCompare(top[0], side, 0.5, at + "top left, side");
                fuzzyCompare(top[1], above + fm[0] * lt.font.pixelSize, 0.5, at + "top left, baseline");
                fuzzyCompare(bottom[0], side, 0.5, at + "bottom right, side");
                fuzzyCompare(bottom[1], below + fm[1] * rb.font.pixelSize, 0.5, at + "bottom right, baseline");
            }
        }

        // The pixels of `shot` inside `box` (device px) that differ from `bare` by half the strongest difference or
        // more, so faint edges count as blank.
        function ink(shot, bare, box) {
            const px = [];
            for (let y = box[1]; y < box[3]; y++)
                for (let x = box[0]; x < box[2]; x++) {
                    const p = shot.pixel(x, y), q = bare.pixel(x, y);
                    px.push([x, y, Math.abs(p.r - q.r) + Math.abs(p.g - q.g) + Math.abs(p.b - q.b)]);
                }
            const most = Math.max(...px.map(p => p[2]));
            return px.filter(p => p[2] > 0 && p[2] >= most / 2);
        }
        // Where line `l` of `key` may draw on its board, in device px at `k` per px: its box, one font size wider all
        // round for marks and overhangs, inside the key so a neighbour's ink stays out.
        function box(key, l, k) {
            const m = l.font.pixelSize;
            const r = l.mapToItem(key.parent, -m, -m, l.width + 2 * m, l.height + 2 * m);
            const clamp = (v, lo, hi) => Math.round(Math.max(lo, Math.min(hi, v)) * k);
            const [x1, y1] = [key.x + key.width, key.y + key.height];
            const [w, h] = [r.x + r.width, r.y + r.height];
            return [clamp(r.x, key.x, x1), clamp(r.y, key.y, y1), clamp(w, key.x, x1), clamp(h, key.y, y1)];
        }
        // The blank cap between inks `a` and `b`, in px on a screen of `k` device pixels per px: their nearest pixel
        // centres, less the one pixel that centres of touching inks are apart.
        function blank(a, b, k) {
            let gap = Infinity;
            for (const p of a)
                for (const q of b)
                    gap = Math.min(gap, Math.hypot(p[0] - q[0], p[1] - q[1]));
            return (gap - 1) / k;
        }
        // `board` grabbed with only line `n` of each key in `keys` drawn (0 main, 1 second, 2 third; -1 none).
        function only(board, keys, n) {
            for (const [, lines] of keys)
                lines.forEach((l, i) => l && (l.opacity = i === n ? 1 : 0));
            return grabImage(board);
        }
        // Each corner legend of every key on `board` in `state` with a main legend, with the blank cap between its ink
        // and the main legend's ink in px; each line is drawn alone over the bare board.
        function gaps(board, state) {
            const keys = board.children.filter(k => k.kv && k.kv.kind === "char").map(k => {
                const ks = state.keys[k.kv.id];
                return [k, [ks.main, ks.second, ks.third].map(t => t !== "" && line(k, t))];
            }).filter(([, l]) => l[0] && (l[1] || l[2]));
            const bare = only(board, keys, -1);
            const k = bare.width / board.width;
            const shots = [0, 1, 2].map(n => only(board, keys, n));
            const found = [];
            for (const [key, lines] of keys) {
                const main = ink(shots[0], bare, box(key, lines[0], k));
                for (const n of [1, 2])
                    if (lines[n])
                        found.push([lines[0].text, lines[n].text, blank(main, ink(shots[n], bare, box(key, lines[n], k)), k)]);
            }
            return found;
        }

        // Inks in neighbouring pixels touch at any screen scale, even corner to corner; at 100 %, one blank pixel
        // between them is the 1 px of clear cap the check below asks for.
        function test_inks_in_neighbouring_pixels_touch_at_any_scale() {
            for (const k of [1, 1.25, 2])
                compare(blank([[0, 0]], [[1, 0]], k), 0, "side by side at " + k);
            verify(blank([[0, 0]], [[1, 1]], 1) < clearPx, "corner to corner");
            compare(blank([[0, 0]], [[2, 0]], 1), clearPx, "one blank pixel at 100 %");
        }

        // The owner saw corner legends touch letters such as ض and س, and Dolch's @ touch its 2 (2026-10-10): every
        // key with a corner legend keeps `clearPx` of blank cap between the inks, at the default and smallest sizes.
        function test_no_corner_legend_touches_the_main_one_data() {
            const sizes = [1, Face.sizeMin];
            const rows = themes().map(t => sizes.map(s => ({ tag: t.tag + " at " + s, lk: t.lk, s: s })));
            return [].concat(...rows);
        }
        function test_no_corner_legend_touches_the_main_one(row) {
            fonts(row.lk);
            const [touching, seen] = [[], []];
            for (const st of root.states) {
                const props = { lk: row.lk, st: Face.states[st.name], res: Board.resOf(row.lk, st.ui), s: row.s };
                const board = createTemporaryObject(boardC, root, props);
                risen(board);
                for (const [main, corner, gap] of gaps(board, props.st)) {
                    const at = st.name + " " + main + " " + corner + ": " + gap.toFixed(2) + " px";
                    (gap < clearPx * row.s ? touching : seen).push(at);
                }
                board.visible = false;
                board.destroy();
            }
            verify(seen.length > 0, "the case this test is for");
            compare(touching, [], "a corner legend touches the main one");
        }
    }
}
