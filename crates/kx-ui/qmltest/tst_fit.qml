// Every key label in each theme, language and size, on the usual board and the shortcuts layer's F-key page: set in the
// theme's font and inside its cap, as the keyboard window lays it out over stand-in bridges fed from the face's golden
// files, the hub's texts and the real look; and each theme's font metrics, which the drawn check places legends by.
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
import "pointer.js" as Pointer
import "fonts.js" as Fonts

Item {
    id: root
    width: 200
    height: 100

    // Measures a theme's Latin font, as the drawn check's metrics must match it.
    FontMetrics { id: fm }

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }

    TestCase {
        name: "fit"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A keyboard window over fresh stand-ins, laid out and not shown; the real pointer passes through it.
        function make(props, kbProps) {
            const ui = createTemporaryObject(uiC, root, props || {});
            const kb = createTemporaryObject(kbC, root, kbProps || {});
            const win = createTemporaryObject(winC, root, { ui: ui, keyboard: kb });
            verify(win, "the window is made");
            Pointer.through([win]);
            return win;
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
        // Board `b` at `size`, off preset: the module scales every box by the size (kx-mod-keyboard geom.rs).
        function scaled(size, b) {
            const box = r => Object.assign({}, r, { x: r.x * size, y: r.y * size, w: r.w * size, h: r.h * size });
            return Object.assign({}, b, { size: size, preset: null, plate: box(b.plate), bar: box(b.bar), dpad: box(b.dpad), keys: b.keys.map(box) });
        }

        // One row per theme, so a theme whose Windows font this system lacks skips alone (fonts.js).
        function test_every_label_fits_its_face_in_each_language_and_size_data() {
            return [N, E, D].map(t => ({ tag: t.lk.look.id, lk: t.lk }));
        }
        function test_every_label_fits_its_face_in_each_language_and_size(row) {
            Fonts.need([row.lk.look.latin_fonts[0]], skip);
            // Every step from the smallest size to the largest, and each preset, which may sit off the steps.
            const steps = Math.round((Face.sizeMax - Face.sizeMin) / Face.sizeStep);
            const grid = Array.from({ length: steps + 1 }, (_, i) => Face.sizeMin + i * Face.sizeStep);
            const sizes = [...new Set(grid.concat(Face.board.presets))];
            for (const t of [En, Fr, Ar])
                for (const page of [null, "layer-3"])
                    labelsFit(row.lk, t, sizes, page);
        }
        // The drawn check places each legend by its theme's font metrics (kx-theme look.rs), so they are the font's own.
        function test_the_drawn_checks_font_metrics_are_the_fonts_own_data() {
            return test_every_label_fits_its_face_in_each_language_and_size_data();
        }
        function test_the_drawn_checks_font_metrics_are_the_fonts_own(row) {
            // A large size, so Qt's rounding is a small share; a capital H's top is the cap height.
            const px = 100;
            const font = row.lk.look.latin_fonts[0];
            Fonts.need([font], skip);
            make({ look: row.lk });
            tryVerify(() => Qt.fontFamilies().indexOf(font) >= 0, 5000, font + " loads");
            fm.font.family = font;
            fm.font.pixelSize = px;
            const got = [fm.ascent, fm.descent, -fm.tightBoundingRect("H").y];
            const names = ["ascent", "descent", "cap height"];
            row.lk.look.font_metrics.forEach((m, i) => fuzzyCompare(got[i] / px, m, 0.005, font + " " + names[i]));
        }
        // Where a label may run on cap `k`, left and right: the face the owner sees less the padding, or a sculpted
        // top plus what may spill onto the skirt.
        function room(k, lk, size) {
            const faces = all(k, o => o.base !== undefined && o.g !== undefined && o.visible);
            const face = faces[faces.length - 1];
            const left = face.mapToItem(k, 0, 0).x;
            const cap = lk.look.cap;
            const more = cap.kind === "skirt" ? cap.label_spill_px * size : -lk.shape.legend.label_pad_px * size;
            return [left - more, left + face.width + more];
        }
        // Each label in look `lk` and texts `t`, at every size in `sizes`, is set in the theme's font and fits; one
        // hidden window takes each size in turn, and the keyboard moves the same keys. With `page`, it shows that page of
        // the shortcuts layer.
        function labelsFit(lk, t, sizes, page) {
            const board = page ? Face.layers[page] : Face.board;
            const kb = page ? { board: board, state: Face.states[page] } : {};
            const win = make({ settings: t.settings, lang: t.lang, texts: t.texts, look: lk }, kb);
            const font = lk.look.latin_fonts[0];
            const where = font + " " + t.lang.tag + " " + (page || "") + " ";
            const keys = all(win.contentItem, o => o.kv !== undefined && o.later !== undefined && o.label.length > 0);
            verify(keys.length > 0, where + "labels");
            verify(!page || keys.some(k => k.kv.side), where + "the layer's F keys among them");
            const labels = keys.map(k => all(k, o => o.text === k.label && o.fontInfo !== undefined && o.visible)[0]);
            tryVerify(() => labels.every(l => l.fontInfo.family === font), 5000, where + "in " + labels[0].fontInfo.family);
            for (const size of sizes) {
                win.board = scaled(size, board);
                keys.forEach((k, i) => fits(k, labels[i], lk, size, t !== Fr, where + size.toFixed(2) + ": " + k.label));
            }
        }
        // Label `l` stays in cap `k`'s room, give or take float rounding, above the smallest size, and at the P1 size
        // when `p1`.
        function fits(k, l, lk, size, p1, at) {
            const slack = 0.01;
            // A row lays out once a frame; the window is hidden, so any row around the label lays out now.
            for (let p = l.parent; p && p !== k; p = p.parent)
                if (typeof p.forceLayout === "function")
                    p.forceLayout();
            const [lo, hi] = room(k, lk, size);
            const left = l.mapToItem(k, (l.width - l.contentWidth) / 2, 0).x;
            verify(left >= lo - slack && left + l.contentWidth <= hi + slack, at + " off by " + [lo - left, left + l.contentWidth - hi]);
            verify(l.fontInfo.pixelSize >= lk.shape.legend.label_min_px, at + " not below the smallest size");
            if (p1)
                compare(l.fontInfo.pixelSize, l.font.pixelSize, at + " keeps the P1 size");
        }
    }
}
