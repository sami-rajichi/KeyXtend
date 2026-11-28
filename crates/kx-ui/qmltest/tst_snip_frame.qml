// A region's frame on a frozen screen over the stand-in Snip object (plan 2026-11-26 task 4): a dashed box with eight
// handles named in each language, a held handle that the frame follows by the module's rules while the strip hides,
// clicks on the frozen screen sent only where a frame takes them, and a reader's press on a handle by name.
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
    Component { id: appC; SnipApp {} }

    TestCase {
        name: "snipframe"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` in language `tag`, once the strip shows, then view `then`.
        function make(name, tag, then) {
            const snip = createTemporaryObject(snipC, root);
            const app = createTemporaryObject(appC, root, { snip: snip, lk: N.lk, res: Kit.resOf(N.lk, tag || "en") });
            snip.page = Kit.view(name);
            tryVerify(() => app.strip.strip.visible, 2000, "the strip shows");
            const ov = app.overlays.objectAt(0);
            tryVerify(() => ov.visible);
            if (then)
                snip.page = Kit.view(then);
            return { snip: snip, app: app, ov: ov, frame: Kit.named(ov.contentItem, "frame") };
        }
        // Make's windows with the right handle held, the pointer last at `p` (physical) before the pick-up.
        function held(p) {
            const s = make("framed");
            if (p)
                s.app.pointer = p;
            s.snip.page = Kit.view("held");
            return s;
        }
        // The framed view with its part `dh` taller, as once a handle was set.
        function grown(dh) {
            const v = Kit.view("framed");
            v.strip.part.h += dh;
            return v;
        }
        // The shown handles of overlay `ov`, clockwise from the top-left corner.
        function handles(ov) {
            return Board.all(ov.contentItem, o => o.objectName === "handle" && o.visible);
        }
        // The size tag's words for `w` by `h` in English.
        function sized(w, h) {
            return Kit.text("en", "snip.size").replace("%1", w).replace("%2", h);
        }

        function test_a_regions_strip_draws_a_dashed_frame_and_eight_named_handles_in_each_language() {
            for (const tag of Kit.langs) {
                const s = make("framed", tag);
                const hs = handles(s.ov);
                compare(hs.map(h => h.Accessible.name), Snip.ids.handles.map(id => Kit.text(tag, "snip.handle-" + id)), tag);
                compare(hs.map(h => h.Accessible.id), Snip.ids.handles.map(id => "snip.handle." + id));
                const b = s.ov.local(Snip.views.framed.strip.part);
                const at = hs.map(h => [h.x + h.width / 2, h.y + h.height / 2]);
                compare(at[0], [b.x, b.y], "the top-left corner");
                compare(at[3], [b.x + b.width, b.y + b.height / 2], "the right side's middle");
                verify(Kit.named(s.ov.contentItem, "outline").visible, "the dashed box");
                verify(!Kit.named(s.ov.contentItem, "edge").visible, "instead of the hole's edge");
                compare(hs[0].width, N.lk.shape.snip.handle_hit_px, "the click area");
            }
        }

        function test_a_strip_without_a_frame_shows_no_handle_and_takes_no_click() {
            const s = make("strip");
            compare(handles(s.ov).length, 0);
            verify(Kit.named(s.ov.contentItem, "edge").visible, "the hole's edge as before");
            mouseClick(s.ov.contentItem, 20, 20);
            compare(s.snip.heard("spot").length, 0);
        }

        function test_a_click_on_a_framed_strips_frozen_screen_is_a_spot() {
            const s = make("framed");
            mouseClick(s.ov.contentItem, 20, 20);
            const sent = s.snip.heard("spot");
            compare(sent.length, 1);
            compare([sent[0].spot, sent[0].x, sent[0].y, sent[0].round], [0, 20, 20, s.app.round]);
        }

        // A light tool: with no handle held, a pointer move rebuilds no frame.
        function test_with_no_handle_held_a_pointer_move_leaves_the_frame_alone() {
            const s = make("framed");
            let rebuilt = 0;
            s.ov.partChanged.connect(() => rebuilt++);
            for (const x of [30, 40, 50])
                s.app.pointer = Qt.point(x, 20);
            compare(rebuilt, 0);
        }

        // The handles lit in the ring colour.
        function lit(ov) {
            return handles(ov).filter(h => Qt.colorEqual(Kit.named(h, "square").color, N.lk.palette.ring)).map(h => h.key);
        }

        function test_the_held_handle_moves_its_side_by_the_pointers_travel_while_the_strip_hides() {
            const s = held();
            const [f, part] = [Snip.views.held.strip.frame, Snip.views.held.strip.part];
            tryVerify(() => !s.app.strip.strip.visible, 2000, "no strip while held");
            s.app.pointer = Qt.point(f.grip.at.x + 50, f.grip.at.y + 30);
            compare(s.ov.part.box, { x: part.x, y: part.y, w: part.w + 50, h: part.h }, "across only");
            compare(Kit.texts(Kit.named(s.ov.contentItem, "tag"), sized(part.w + 50, part.h)).length, 1, "its size");
            compare(lit(s.ov), [f.grip.handle], "the held handle is lit");
            const st = s.app.strip.strip;
            mouseClick(s.ov.contentItem, st.x - s.ov.x + st.width / 2, st.y - s.ov.y + st.height / 2);
            compare(s.snip.heard("spot").length, 1, "where the strip was, a click sets the handle");
        }

        function test_once_set_the_strip_shows_again_at_the_new_part_and_once_dropped_at_the_old_one() {
            const s = make("framed");
            const y = s.app.strip.strip.y;
            s.snip.page = Kit.view("held");
            tryVerify(() => !s.app.strip.strip.visible, 2000, "hidden while held");
            s.snip.page = grown(30);
            tryVerify(() => s.app.strip.strip.visible, 2000, "set");
            fuzzyCompare(s.app.strip.strip.y - y, 30 / s.ov.dpr, 1, "under the new part");
            s.snip.page = Kit.view("held");
            tryVerify(() => !s.app.strip.strip.visible);
            s.snip.page = Kit.view("framed");
            tryVerify(() => s.app.strip.strip.visible, 2000, "dropped");
            compare(s.app.strip.strip.y, y, "under the old part");
        }

        function test_minor_1_past_the_far_side_the_handle_lit_is_the_one_the_moving_edge_now_holds() {
            const s = held();
            const [f, part] = [Snip.views.held.strip.frame, Snip.views.held.strip.part];
            s.app.pointer = Qt.point(f.grip.at.x - part.w - 50, f.grip.at.y);
            compare(lit(s.ov), [s.app.names.handle.left], "the right handle swapped to the left");
            s.app.pointer = Qt.point(f.grip.at.x + 10, f.grip.at.y);
            compare(lit(s.ov), [s.app.names.handle.right], "back on the right");
        }

        function test_after_a_readers_pick_up_the_frame_waits_for_the_pointer_to_move() {
            const s = held(Qt.point(2, 2));
            const [f, part] = [Snip.views.held.strip.frame, Snip.views.held.strip.part];
            compare(s.ov.part.box, part, "no jump to the pointer left at the corner");
            s.app.pointer = Qt.point(f.grip.at.x + 20, f.grip.at.y);
            compare(s.ov.part.box.w, part.w + 20, "it follows once the pointer moves");
        }

        function test_edges_1_and_2_the_held_handle_swaps_past_the_far_side_stops_at_the_screen_and_keeps_its_least() {
            const s = held();
            const [f, part] = [Snip.views.held.strip.frame, Snip.views.held.strip.part];
            const left = part.x;
            s.app.pointer = Qt.point(f.grip.at.x - part.w - 50, f.grip.at.y);
            compare(s.ov.part.box, { x: left - 50, y: part.y, w: 50, h: part.h }, "swapped");
            s.app.pointer = Qt.point(f.grip.at.x + 5000, f.grip.at.y);
            compare(s.ov.part.box.w, f.within.x + f.within.w - left, "at its screen's edge");
            s.app.pointer = Qt.point(f.grip.at.x - part.w + 1, f.grip.at.y);
            compare(s.ov.part.box, { x: left, y: part.y, w: f.min, h: part.h }, "its least side");
            s.app.pointer = Qt.point(f.grip.at.x, f.grip.at.y);
            compare(s.ov.part.box, part, "edge 3: back on its pick-up spot");
        }

        function test_the_pointer_over_a_handle_is_still_followed_and_shows_how_the_handle_moves() {
            const s = make("framed");
            const area = Kit.named(s.ov.contentItem, "spots");
            const hs = handles(s.ov);
            const want = [Qt.SizeFDiagCursor, Qt.SizeVerCursor, Qt.SizeBDiagCursor, Qt.SizeHorCursor];
            for (const i of [0, 1, 2, 3, 4]) {
                const [x, y] = [Math.round(hs[i].x + hs[i].width / 2), Math.round(hs[i].y + hs[i].height / 2)];
                mouseMove(s.ov.contentItem, x, y);
                const p = Qt.point(s.ov.screenInfo.rect.x + x * s.ov.dpr, s.ov.screenInfo.rect.y + y * s.ov.dpr);
                tryVerify(() => s.app.pointer.x === p.x && s.app.pointer.y === p.y, 2000, "" + s.app.pointer);
                compare(area.cursorShape, want[i % want.length], hs[i].key);
            }
            mouseMove(s.ov.contentItem, 2, 2);
            tryCompare(area, "cursorShape", Qt.CrossCursor, 2000, "off every handle");
        }

        function test_minor_2_a_readers_press_on_a_handle_names_it_and_sends_no_spot() {
            for (const v of ["framed", "held"]) {
                const s = make("framed", "en", v);
                const h = handles(s.ov)[7];
                h.Accessible.pressAction();
                const sent = s.snip.heard("act");
                compare(sent.length, 1, v);
                compare([sent[0].act, sent[0].id, sent[0].round], [s.app.names.acts.handle, h.key, s.app.round], v);
                compare(s.snip.heard("spot").length, 0, v);
            }
        }
    }
}
