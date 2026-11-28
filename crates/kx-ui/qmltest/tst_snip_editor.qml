// Snip's editor over the stand-in Snip object (design "The editor"): its ids are the hub's, it opens where the hub keeps
// it with the theme's sizes sent again at its part's scale, names its keys in English, French and Arabic, stops at its
// least size, keeps its place for next time (edge 19), says why a key waits, and leaves the top under a system dialog,
// still over its one shadow and under the close question after it (edge 10). Maximising is in tst_snip_maximise.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "snipkit.js" as Kit
import "fixtures/snip.js" as Snip
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    Component { id: snipC; FakeSnip {} }
    Component { id: winC; FakeWin {} }
    Component { id: appC; SnipApp {} }
    Component { id: namesC; SnipNames {} }
    Component { id: metricsC; FontMetrics {} }

    TestCase {
        name: "snipeditor"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in in language `tag`, aims counted, showing view `name` once the editor is drawn
        // and still (after `before` if given), or no view for a null `name`.
        function make(name, tag, before) {
            const snip = createTemporaryObject(snipC, root);
            const win = createTemporaryObject(winC, root);
            const aims = [];
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: N.lk, res: Kit.resOf(N.lk, tag || "en"), win: win, aim: () => aims.push(true)
            });
            if (before) {
                snip.page = Kit.view(before);
                tryCompare(app, "shown", Snip.views[before].stage);
            }
            const ed = app.editor;
            if (name) {
                snip.page = Kit.view(name);
                tryVerify(() => ed.ready, 2000, "the editor shows, drawn and still");
            }
            return { snip: snip, win: win, app: app, aims: aims, ed: ed, item: ed.editor.contentItem };
        }

        function test_the_editors_ids_are_the_hubs() {
            const names = createTemporaryObject(namesC, root);
            for (const list of Object.keys(Snip.ids).filter(k => k !== "shapes"))
                compare(names.ids[list], Snip.ids[list], list);
            compare(Object.values(names.shapes), Snip.ids.shapes);
        }

        function test_it_opens_where_the_hub_keeps_it_and_names_its_keys_in_each_language() {
            for (const tag of Kit.langs) {
                const s = make("edit", tag);
                const w = s.ed.editor;
                compare(w.title, Kit.text(tag, "window.snip-editor"));
                verify(w.flags & Qt.WindowDoesNotAcceptFocus && w.flags & Qt.WindowStaysOnTopHint, "it never takes focus");
                compare(Kit.box(w), Kit.shown(s.app, s.snip.place), "where the hub keeps it");
                const tools = Kit.keys(s.item, "snip.tool.");
                compare(tools.map(o => o.Accessible.name), Snip.ids.tools.map(id => Kit.text(tag, "snip.tool-" + id)), tag);
                verify(Kit.key(s.item, "snip.tool.pen").Accessible.checked, "the picked tool is on");
                verify(tools.every(o => o.Accessible.role === Accessible.RadioButton), "one tool of many");
                const inks = Kit.keys(s.item, "snip.ink.");
                compare(inks.map(o => o.Accessible.name), Snip.ids.inks.map(id => Kit.text(tag, "snip.ink-" + id)));
                verify(Kit.key(s.item, "snip.ink.red").Accessible.checked, "the picked ink is on");
                const own = ["snip.edit.move", "snip.edit.full", "snip.edit.close", "snip.edit.size", "snip.edit.undo", "snip.edit.redo"];
                verify(own.every(id => !!Kit.key(s.item, id)), "the editor's own keys show");
                compare(Kit.key(s.item, "snip.edit.full").Accessible.name, Kit.text(tag, "snip.edit-full"));
                compare(Kit.key(s.item, "snip.keep.save").Accessible.name, Kit.text(tag, "snip.strip-save"));
                compare(Kit.texts(s.item, Kit.text(tag, "snip.editor")).length, 1, "its name in the title bar");
                compare(Kit.key(s.item, "snip.edit.close").says, Kit.text(tag, "snip.edit-close"), "an icon alone names itself");
            }
        }

        function test_an_editor_that_opens_gets_the_sizes_again_at_its_parts_scale() {
            const s = make("edit", "en", "strip");
            const fits = s.snip.heard("fit");
            compare(fits.length, 2, "a snip's start, then the editor");
            const [sh, d] = [N.lk.shape.snip, root.Screen.devicePixelRatio];
            compare(fits[1].fit.thick.map(t => t.pen), sh.pen_px.map(p => Math.ceil(p * d)));
            compare(fits[1].fit.text.map(t => t.height), sh.text_px.map(p => Math.ceil(p * d)));
            compare(s.app.sent, fits[1].fit, "the editor draws by what it sent");
        }

        function test_a_resize_stops_at_the_least_size_and_is_kept_for_next_time() {
            const s = make("edit");
            const at = s.snip.place;
            s.win.pointer = Qt.point(at.x + at.w, at.y + at.h);
            mouseClick(Kit.key(s.item, "snip.edit.size"));
            verify(s.ed.mover.busy, "the first click starts");
            s.win.pointer = Qt.point(at.x + 10, at.y + 10);
            s.ed.mover.follow();
            compare([s.ed.editor.place.w, s.ed.editor.place.h], [s.snip.least.w, s.snip.least.h], "it stops at its least");
            mouseClick(s.ed.editor.contentItem);
            verify(!s.ed.mover.busy, "the next click ends it");
            const placed = s.snip.heard("placed");
            compare(placed.length, 1);
            compare(placed[0].placed, s.ed.editor.place);
            s.snip.page = Kit.view("idle");
            tryVerify(() => !s.ed.editor.visible);
            s.snip.page = Kit.view("edit");
            tryVerify(() => s.ed.editor.visible);
            compare(s.ed.editor.place, placed[0].placed, "it opens where it was left");
        }

        function test_a_move_follows_the_pointer_held_where_it_was_grabbed() {
            const s = make("edit");
            const at = s.snip.place;
            s.win.pointer = Qt.point(at.x + 40, at.y + 10);
            mouseClick(Kit.key(s.item, "snip.edit.move"));
            s.win.pointer = Qt.point(at.x + 140, at.y + 60);
            s.ed.mover.follow();
            compare([s.ed.editor.place.x, s.ed.editor.place.y], [at.x + 100, at.y + 50]);
            s.ed.mover.finish();
            compare(s.snip.heard("placed").length, 1);
        }

        function test_a_key_that_waits_says_why_and_never_acts() {
            for (const tag of Kit.langs) {
                const s = make("ends", tag);
                const says = id => Kit.key(s.item, id).Accessible.description;
                compare(says("snip.edit.undo"), Kit.text(tag, "snip.why-undo"), tag);
                compare(says("snip.edit.redo"), Kit.text(tag, "snip.why-redo"));
                compare(says("snip.more.thick"), Kit.text(tag, "snip.why-most"));
                compare(says("snip.less.thick"), "");
                compare(says("snip.keep.save"), Kit.text(tag, "snip.dim-hidden"));
                const copy = Kit.key(s.item, "snip.keep.copy");
                compare(copy.Accessible.name, Kit.text(tag, "snip.done-copy"));
                compare(copy.icon, N.lk.icons.snip.tick);
                mouseClick(Kit.key(s.item, "snip.edit.undo"));
                mouseClick(Kit.key(s.item, "snip.keep.save"));
                compare(s.snip.heard("act"), [], "a dimmed key never acts");
                compare(s.snip.heard("flat"), []);
            }
            const t = make("typing");
            compare(Kit.key(t.item, "snip.less.text").Accessible.description, Kit.text("en", "snip.why-least"));
        }

        function test_a_click_aims_the_letters_and_acts() {
            const s = make("edit");
            mouseClick(Kit.key(s.item, "snip.tool.box"));
            mouseClick(Kit.key(s.item, "snip.ink.blue"));
            mouseClick(Kit.key(s.item, "snip.more.thick"));
            mouseClick(Kit.key(s.item, "snip.edit.close"));
            const r = Snip.views.edit.round;
            compare(s.snip.heard("act"), [
                { act: "tool", id: "box", arg: 0, round: r },
                { act: "ink", id: "blue", arg: 0, round: r },
                { act: "size", id: "", arg: 1, round: r },
                { act: "edit", id: "close", arg: 0, round: r }
            ]);
            compare(s.aims.length, 4, "each press aims the letters at Snip");
        }

        function test_each_way_out_lets_the_picture_go() {
            for (const way of [["snip.edit.close"], ["snip.keep.copy", "snip.edit.close"]]) {
                const s = make("edit");
                for (const id of way)
                    mouseClick(Kit.key(s.item, id));
                tryVerify(() => s.snip.heard("flat").length === way.length - 1, 2000);
                verify(String(s.ed.canvas.picture.source) !== "", "it shows the picture until the snip ends");
                s.snip.page = Kit.view("idle");
                tryVerify(() => !s.ed.editor.visible);
                compare(String(s.ed.canvas.picture.source), "", way.join(" then "));
                compare(s.ed.ed, null, "nor keeps its view");
            }
        }

        function test_the_steppers_show_window_sizes_at_any_capture_scale() {
            const sh = N.lk.shape.snip;
            for (const name of ["edit", "typing"]) {
                const s = make(name);
                s.app.taken = 2;
                s.app.send(2);
                const [st, i] = [s.ed.tools.stepper, Snip.views[name].edit.picture.size];
                if (name === "edit")
                    compare([st.dot, st.value], [sh.pen_px[i], String(sh.pen_px[i])], "a line as wide as it shows");
                else
                    compare(st.value, String(sh.text_px[i]), "a text size as it shows");
            }
        }

        function test_a_letter_is_as_wide_as_the_marks_own_font_draws_it() {
            const s = make("edit");
            const [sh, fam] = [N.lk.shape.snip, s.app.res.fam];
            const wide = (family, px) => createTemporaryObject(metricsC, root, {
                font: Qt.font({ family: family, pixelSize: px, weight: sh.mark_weight })
            }).averageCharacterWidth;
            const want = sh.text_px.map(px => Math.max(1, Math.round(Math.max(wide(fam.latin, px), wide(fam.arabic, px)))));
            compare(s.app.sent.text.map(t => t.letter), want, "the wider of the Latin and Arabic mark fonts");
        }

        function test_a_system_dialog_drops_it_from_the_top() {
            const s = make("edit-low");
            verify(!(s.ed.editor.flags & Qt.WindowStaysOnTopHint), "the editor leaves the top");
            verify(!(s.ed.flags & Qt.WindowStaysOnTopHint), "and its shadow");
        }

        // Edge 10: each Snip window lies over its one shadow from its first show, and still after the close question, a
        // Save dialog, a close and a reopen, though Qt lifts each window alone on new flags; nothing dims the editor.
        function test_a_dialog_and_a_reopen_leave_each_window_over_its_one_shadow() {
            const s = make(null);
            const pairs = Kit.owned(s.app);
            const z = Kit.stack([].concat(...pairs.map(([glow, w]) => [w, glow])), s.snip);
            for (const [name, i] of [["region", 3], ["strip", 4], ["record", 5], ["panel-capture", 2], ["ask-edit", 1]]) {
                s.snip.page = Kit.view(name);
                tryVerify(() => pairs[i][1].visible, 2000, name + " shows its window");
            }
            compare(Kit.dims(s.item, N.lk).length, 1, "the question dims it");
            for (const name of ["edit-low", "edit", "idle", "edit"]) {
                s.snip.page = Kit.view(name);
                tryCompare(s.app, "shown", Snip.views[name].stage);
            }
            tryVerify(() => s.ed.editor.visible, 2000, "it opens again");
            verify(pairs.every(([glow]) => glow.visible === (glow === s.ed)), "one shadow shows, the editor's");
            for (const [glow, w] of pairs) {
                verify(z.above(w, glow), w.title + " lies over its shadow");
                verify(w.flags & Qt.WindowDoesNotAcceptFocus && !(w.flags & Qt.WindowTransparentForInput), w.title + " takes clicks only");
            }
            verify(z.above(s.app.ask, s.ed.editor), "the close question stays over the editor");
            tryCompare(s.ed.follow, "opacity", 1);
            compare(s.ed.shade.opacity, 1, "the shadow as at first open");
            compare(Kit.dims(s.item, N.lk).length, 0, "no dim left over the editor");
        }
    }
}
