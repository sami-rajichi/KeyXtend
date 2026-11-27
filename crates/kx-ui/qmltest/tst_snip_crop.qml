// Crop's frame in the editor over the stand-in Snip object (plan 2026-11-26 task 5): the frame and its eight named
// handles over the part kept, whole at the picture's edge where a click on a handle's outer half still counts; the held
// side following the pointer by the module's rules; the first click outlining the part to the pointer; Apply ✓ and
// Cancel ✗ on the second row only while a frame waits with no handle held; and, while one is held, a set on the plate
// or in the editor's padding.
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

    TestCase {
        name: "snipcrop"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` (or that view itself) in language `tag`, once the editor
        // shows, drawn and still, so a click lands where its pop-in ends.
        function make(name, tag) {
            const snip = createTemporaryObject(snipC, root);
            const win = createTemporaryObject(winC, root);
            const app = createTemporaryObject(appC, root, { snip: snip, lk: N.lk, res: Kit.resOf(N.lk, tag || "en"), win: win });
            snip.page = typeof name === "string" ? Kit.view(name) : name;
            const ed = app.editor;
            tryVerify(() => ed.ready, 2000, "the editor shows, drawn and still");
            return { snip: snip, app: app, c: ed.canvas, crop: Kit.named(ed.canvas, "crop"), item: ed.editor.contentItem };
        }
        // The frame waiting on view `name`.
        function waiting(name) {
            return Snip.views[name].edit.picture.frame;
        }
        // The shown handles on the canvas of `s`, clockwise from the top-left corner.
        function handles(s) {
            return Board.all(s.c, o => o.objectName === "handle" && o.visible);
        }
        // The centre of handle `h` in the canvas's units.
        function centre(s, h) {
            return h.mapToItem(s.c, h.width / 2, h.height / 2);
        }
        // Picture box `r` in the canvas's units.
        function local(s, r) {
            const [v, sh, k] = [s.c.view, s.c.shown, s.c.k];
            return { x: v.x + (r.x - sh.x) * k, y: v.y + (r.y - sh.y) * k, w: r.w * k, h: r.h * k };
        }
        function near(a, b) {
            return Math.abs(a - b) < 0.5;
        }
        // The handles lit in the ring colour.
        function lit(s) {
            return handles(s).filter(h => Qt.colorEqual(Kit.named(h, "square").color, N.lk.palette.ring)).map(h => h.key);
        }

        function test_a_waiting_frame_shows_eight_named_handles_round_its_part_over_the_part_kept() {
            for (const tag of Kit.langs) {
                const s = make("crop-framed", tag);
                const pic = Snip.views["crop-framed"].edit.picture;
                compare([s.c.shown.w, s.c.shown.h], [pic.crop.w, pic.crop.h], "the part kept shows");
                const hs = handles(s);
                compare(hs.map(h => h.Accessible.name), Snip.ids.handles.map(id => Kit.text(tag, "snip.handle-" + id)), tag);
                compare(hs.map(h => h.Accessible.id), Snip.ids.handles.map(id => "snip.handle." + id));
                const [b, at] = [local(s, pic.frame.part), hs.map(h => centre(s, h))];
                verify(near(at[0].x, b.x) && near(at[0].y, b.y), "the top-left corner");
                verify(near(at[3].x, b.x + b.w) && near(at[3].y, b.y + b.h / 2), "the right side's middle");
                verify(Kit.named(s.c, "outline").visible, "the dashed box");
                compare(hs[0].width, N.lk.shape.snip.handle_hit_px, "its click area, in window pixels at any scale");
            }
        }

        function test_on_the_pictures_edge_each_handle_shows_and_its_outer_half_takes_a_click() {
            const v = Kit.view("crop-framed");
            v.edit.picture.frame.part = v.edit.picture.crop;
            const s = make(v);
            compare(handles(s).length, 8, "each handle shows, half past the picture");
            const left = centre(s, handles(s)[7]);
            verify(near(left.x, s.c.view.x), "on the picture's left edge");
            mouseClick(s.c, left.x - N.lk.shape.snip.handle_px / 2, left.y);
            const press = s.snip.heard("press");
            compare(press.length, 1, "it reaches the module");
            verify(press[0].press[0] < v.edit.picture.crop.x * s.c.k, "left of the picture: " + JSON.stringify(press));
            const c = make("crop");
            mouseClick(c.c, c.c.view.x - N.lk.shape.snip.handle_px / 2, c.c.view.y + 10);
            compare(c.snip.heard("press"), [], "with no frame, nothing past the picture");
        }

        function test_a_held_handle_moves_its_side_by_the_pointers_travel_by_the_modules_rules() {
            const s = make("crop-held");
            const f = waiting("crop-held");
            const [g, part] = [f.grip.at, f.part];
            compare(lit(s), [s.app.names.handle.right], "the held handle is lit");
            s.c.pointer = { x: g.x + 20, y: g.y + 7 };
            compare(s.crop.framed.box, { x: part.x, y: part.y, w: part.w + 20, h: part.h }, "across only");
            s.c.pointer = { x: g.x - part.w - 10, y: g.y };
            compare(s.crop.framed.box, { x: part.x - 10, y: part.y, w: 10, h: part.h }, "edge 1: swapped");
            compare(lit(s), [s.app.names.handle.left], "lit where the moving edge now is");
            s.c.pointer = { x: g.x + 5000, y: g.y };
            compare(s.crop.framed.box.w, f.within.x + f.within.w - part.x, "edge 2: at the part kept");
            s.c.pointer = { x: g.x - part.w + 1, y: g.y };
            compare(s.crop.framed.box, { x: part.x, y: part.y, w: f.min, h: part.h }, "edge 2: its least side");
            s.c.pointer = { x: g.x, y: g.y };
            compare(s.crop.framed.box, part, "edge 3: back on its pick-up spot");
            const b = local(s, s.crop.framed.box);
            const at = handles(s).map(h => centre(s, h));
            verify(near(at[4].x, b.x + b.w) && near(at[4].y, b.y + b.h), "the handles follow");
        }

        // A light tool: with no handle held, a pointer move rebuilds no frame.
        function test_with_no_handle_held_a_pointer_move_leaves_the_frame_alone() {
            const s = make("crop-framed");
            let rebuilt = 0;
            s.crop.framedChanged.connect(() => rebuilt++);
            for (const x of [30, 40, 50])
                s.c.pointer = { x: x, y: 20 };
            compare(rebuilt, 0);
        }

        function test_after_a_readers_pick_up_the_frame_waits_for_the_pointer_to_move() {
            const s = make("crop-framed");
            s.c.pointer = { x: 2, y: 2 };
            s.snip.page = Kit.view("crop-held");
            const f = waiting("crop-held");
            tryVerify(() => lit(s).length === 1, 2000, "picked up");
            compare(s.crop.framed.box, f.part, "no jump to the pointer left at the corner");
            s.c.pointer = { x: f.grip.at.x + 20, y: f.grip.at.y };
            compare(s.crop.framed.box.w, f.part.w + 20, "it follows once the pointer moves");
        }

        function test_the_pointer_shows_how_a_handle_moves_and_a_readers_press_names_it() {
            const s = make("crop-framed");
            const area = Kit.named(s.c, "press");
            const hs = handles(s);
            const want = [Qt.SizeFDiagCursor, Qt.SizeVerCursor, Qt.SizeBDiagCursor, Qt.SizeHorCursor];
            for (const i of [0, 1, 2, 3]) {
                const p = centre(s, hs[i]);
                mouseMove(s.c, Math.round(p.x), Math.round(p.y));
                tryCompare(area, "cursorShape", want[i], 2000, hs[i].key);
            }
            const b = local(s, waiting("crop-framed").part);
            mouseMove(s.c, Math.round(b.x + b.w / 2), Math.round(b.y + b.h / 2));
            tryCompare(area, "cursorShape", Qt.ArrowCursor, 2000, "off every handle");
            hs[3].Accessible.pressAction();
            const r = Snip.views["crop-framed"].round;
            compare(s.snip.heard("act"), [{ act: s.app.names.acts.handle, id: hs[3].key, arg: 0, round: r }]);
            compare(s.snip.heard("press"), [], "no press");
        }

        function test_crops_first_click_outlines_the_part_to_the_pointer() {
            const v = Kit.view("crop");
            v.edit.picture.pending = { x: 60, y: 50 };
            const s = make(v);
            const soon = Kit.named(s.c, "soon");
            verify(!soon.visible, "nothing until the pointer comes");
            s.c.pointer = { x: 160, y: 110 };
            verify(soon.visible, "from the first click to the pointer");
            const b = local(s, { x: 60, y: 50, w: 100, h: 60 });
            const [lt, rb] = [soon.corners[0], soon.corners[2]];
            verify(near(lt.x, b.x) && near(lt.y, b.y) && near(rb.x, b.x + b.w) && near(rb.y, b.y + b.h), JSON.stringify(soon.corners));
            compare(handles(s).length, 0, "no handle before the second click");
        }

        function test_crop_ok_and_cancel_show_on_the_second_row_only_while_a_frame_waits() {
            for (const tag of Kit.langs) {
                const s = make("crop-framed", tag);
                const keys = Kit.keys(s.item, "snip.crop.");
                compare(keys.map(k => k.Accessible.name), Snip.ids.crops.map(id => Kit.text(tag, "snip.crop-" + id)), tag);
                compare(keys.map(k => k.icon), Snip.ids.crops.map(id => N.lk.icons.snip.crop[id]));
                compare(keys.map(k => k.tone), ["enter", "ghost"], "Crop is the main key");
                const tool = Kit.key(s.item, "snip.tool.crop");
                verify(keys[0].mapToItem(s.item, 0, 0).y > tool.mapToItem(s.item, 0, 0).y, "on the second row");
                compare(Kit.keys(s.item, "snip.ink.").length, 0, "the inks make room, as Crop draws none");
                verify(!Kit.key(s.item, "snip.less.thick"), "and the thickness");
            }
            const h = make("crop-held");
            compare(Kit.keys(h.item, "snip.crop.").length, 0, "hidden while a handle is held, so a set click never cuts");
            compare(Kit.keys(h.item, "snip.ink.").length, 0, "the inks stay hidden");
            const tools = Board.all(h.item, o => o.held !== undefined && o.options !== undefined)[0];
            const row = Array.from(tools.options.children).filter(o => o.visible && o.width > 0);
            compare(row[row.length - 1].text, Kit.text("en", "snip.tool-crop"), "the row ends at the tool's name");
            const s = make("crop");
            compare(Kit.keys(s.item, "snip.crop.").length, 0, "no frame waits");
            compare(Kit.keys(s.item, "snip.ink.").length, Snip.ids.inks.length);
        }

        function test_with_the_picture_filling_the_canvas_a_held_handle_takes_a_click_in_the_padding() {
            // A wide picture fills the canvas's width, its frame on the left edge; Left held at picture point `at`, if
            // any.
            const wide = at => {
                const v = Kit.view("crop-held");
                const whole = { x: 0, y: 0, w: 6000, h: 200 };
                Object.assign(v.edit, { width: whole.w, height: whole.h });
                Object.assign(v.edit.picture, { crop: whole });
                const grip = at ? { handle: Snip.ids.handles[7], at: at } : null;
                Object.assign(v.edit.picture.frame, { part: { x: 0, y: 50, w: 400, h: 100 }, within: whole, grip: grip });
                return v;
            };
            const s = make(wide(null));
            verify(near(s.c.view.x, 0) && near(s.c.view.width, s.c.width), "the picture fills the canvas's width");
            // About 8 px out, past the square's outer half, within the click area and the editor's padding.
            const at = Qt.point(-(N.lk.shape.snip.handle_px / 2 + 3), s.c.view.y + s.c.view.height / 2);
            mouseClick(s.c, at.x, at.y);
            const pick = s.snip.heard("press");
            compare(pick.length, 1, "the pick-up in the padding reaches the module");
            // The spot the module picks the handle up at, read back as the hub reads a press.
            const spot = { x: Math.round(pick[0].press[0] / pick[0].scale), y: Math.round(pick[0].press[1] / pick[0].scale) };
            verify(spot.x < 0, "left of the picture: " + JSON.stringify(spot));
            s.snip.page = wide(spot);
            tryVerify(() => !!s.c.pic.frame.grip, 2000, "Left held");
            mouseClick(s.c, at.x, at.y);
            const set = s.snip.heard("press");
            compare(set.length, 2, "the set in the padding reaches the module too");
            compare(set[1].press, pick[0].press, "on the pick-up spot, which sets it unchanged (edge 3)");
        }

        function test_while_a_handle_is_held_a_click_on_the_plate_sets_it() {
            // A tall part, so the plate shows wide on either side of the picture.
            const view = held => {
                const v = Kit.view(held ? "crop-held" : "crop-framed");
                const crop = { x: 100, y: 40, w: 60, h: 120 };
                Object.assign(v.edit.picture, { crop: crop });
                Object.assign(v.edit.picture.frame, { part: { x: 110, y: 60, w: 30, h: 40 }, within: crop });
                return v;
            };
            const s = make(view(true));
            const room = s.c.view.x;
            verify(room > N.lk.shape.snip.handle_hit_px, "the plate shows past a handle's reach: " + room);
            const at = Qt.point(room / 4, s.c.view.y + s.c.view.height / 2);
            mouseClick(s.c, at.x, at.y);
            const press = s.snip.heard("press");
            compare(press.length, 1, "the set reaches the module, which keeps the side on the picture");
            verify(press[0].press[0] < s.c.shown.x * s.c.k, "left of the picture: " + JSON.stringify(press));
            const f = make(view(false));
            mouseClick(f.c, at.x, at.y);
            compare(f.snip.heard("press"), [], "with no handle held, the plate takes no click");
        }

        function test_crop_ok_and_cancel_send_crops_clicks() {
            const s = make("crop-framed");
            mouseClick(Kit.key(s.item, "snip.crop.ok"));
            mouseClick(Kit.key(s.item, "snip.crop.cancel"));
            const [r, kind] = [Snip.views["crop-framed"].round, s.app.names.acts.crop];
            compare(s.snip.heard("act"), [{ act: kind, id: "ok", arg: 0, round: r }, { act: kind, id: "cancel", arg: 0, round: r }]);
        }
    }
}
