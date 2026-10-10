// Corner legends in every theme, in the bundled fonts: one margin in from the face each theme draws, on the Latin
// baseline whatever font they draw in.
pragma ComponentBehavior: Bound
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
    width: Face.board.plate.w
    height: Face.board.plate.h

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
        name: "corners"
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
        // One row per theme, so a theme whose Windows font this system lacks skips alone (fonts.js).
        function themes() {
            return [N, E, D].map(t => ({ tag: t.lk.look.id, lk: t.lk }));
        }
        // Skips without look `lk`'s Windows fonts, and waits for the bundled ones.
        function fonts(lk) {
            Fonts.need([lk.look.latin_fonts[0], lk.look.arabic_fonts[0], lk.look.mark_fonts[0]], skip);
            tryCompare(root, "fontsIn", Assets.assets.fonts.length, 5000, "the bundle is laid out by the texts test");
        }
        // Look `lk`'s fonts and texts `texts`, as the keyboard window picks them.
        function resOf(lk, texts) {
            const l = lk.look;
            return {
                fam: { latin: l.latin_fonts[0], arabic: l.arabic_fonts[0], mark: l.mark_fonts[0], ui: l.latin_fonts[0] },
                icons: Qt.resolvedUrl(Assets.root + "/" + Assets.assets.icons),
                ext: Assets.assets.ext,
                tr: k => texts[k] !== undefined ? texts[k] : k,
                bar: id => id,
                dpad: id => id,
                uid: (group, id) => group + "." + id,
                space: parts => parts.join(" · ")
            };
        }
        // The line `key` shows as `text`.
        function line(key, text) {
            return all(key, o => o.font !== undefined && o.visible && o.text === text)[0];
        }

        // How far `key`'s corner legends `lt` (top left) and `rb` (bottom right) sit in from the face its look draws,
        // at size 1: [side, baseline] for each.
        function margins(key, lt, rb) {
            // The cap's own face, the first one drawn; the inner shading draws it again, moved.
            const cap = all(key, o => o.sculpted !== undefined && o.grad !== undefined)[0];
            const fills = all(cap, o => o.base !== undefined && o.g !== undefined && o.visible);
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
                const key = createTemporaryObject(keyC, root, Object.assign(props, { res: resOf(lk, Ar.texts) }));
                const [lt, rb] = [line(key, ks.second), line(key, ks.third)];
                const [top, bottom] = margins(key, lt, rb);
                const at = ks.third + ": ";
                fuzzyCompare(top[0], lg.corner_px[1], 0.5, at + "top left, side");
                fuzzyCompare(top[1], lg.corner_px[0] + fm[0] * lt.font.pixelSize, 0.5, at + "top left, baseline");
                fuzzyCompare(bottom[0], lg.corner_px[1], 0.5, at + "bottom right, side");
                fuzzyCompare(bottom[1], lg.corner_px[0] + fm[1] * rb.font.pixelSize, 0.5, at + "bottom right, baseline");
            }
        }
    }
}
