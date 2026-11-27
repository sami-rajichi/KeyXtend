// A frozen screen over the stand-in Snip object (design/snip-parts.css .hole, .tag, .hover): a region from its corner
// to the pointer holds both, a video region stops at its screen, the size tag speaks each language, and a ring marks
// the window or screen a click takes, with the hint for each way. Each stage shown and each move asks for the frozen
// screens to go under our other windows.
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
        name: "snipoverlay"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` in language `tag`; its first frozen screen and its shade.
        function make(name, tag) {
            const snip = createTemporaryObject(snipC, root);
            const app = createTemporaryObject(appC, root, { snip: snip, lk: N.lk, res: Kit.resOf(N.lk, tag || "en") });
            snip.page = Kit.view(name);
            tryVerify(() => app.shown === app.stage, 2000, "the stage shows");
            const ov = app.overlays.objectAt(0);
            tryVerify(() => ov.visible);
            return { snip: snip, app: app, ov: ov, shade: Kit.named(ov.contentItem, "shade") };
        }
        // The size tag's words for `w` by `h` in language `tag`.
        function sized(tag, w, h) {
            return Kit.text(tag, "snip.size").replace("%1", w).replace("%2", h);
        }

        function test_each_stage_shown_lays_the_frozen_screens_under_our_windows_by_their_title() {
            for (const tag of Kit.langs) {
                const s = make("corner", tag);
                const first = s.snip.heard("order").length;
                verify(first >= 1, "the stage that showed them");
                s.snip.page = Kit.view("window");
                tryVerify(() => s.snip.heard("order").length > first, 2000, "the next stage");
                const asks = s.snip.heard("order").map(c => c.order);
                verify(asks.every(t => t === s.ov.title), tag + ": " + asks);
                compare(s.ov.title, Kit.text(tag, "window.snip-screen"), tag);
            }
        }

        function test_a_frozen_screen_that_moves_or_resizes_goes_under_our_windows_again_once() {
            const s = make("corner");
            tryVerify(() => s.snip.heard("order").length >= 1);
            const before = s.snip.heard("order").length;
            s.ov.x += 10;
            s.ov.y += 10;
            tryVerify(() => s.snip.heard("order").length === before + 1, 2000, "one ask for both moves");
            s.ov.width += 10;
            s.ov.height += 10;
            tryVerify(() => s.snip.heard("order").length === before + 2, 2000, "one ask for a new size, as at a new scale");
        }

        function test_the_pointer_over_a_frozen_screen_is_followed_in_physical_pixels() {
            const s = make("corner");
            mouseMove(s.ov.contentItem, 30, 20);
            const want = Qt.point(s.ov.screenInfo.rect.x + 30 * s.ov.dpr, s.ov.screenInfo.rect.y + 20 * s.ov.dpr);
            tryVerify(() => s.app.pointer.x === want.x && s.app.pointer.y === want.y, 2000, "" + s.app.pointer);
        }

        function test_a_region_from_its_corner_to_the_pointer_holds_both_and_tags_its_size_in_each_language() {
            for (const tag of Kit.langs) {
                const s = make("corner", tag);
                const c = Snip.views.corner.pick.corner;
                s.app.pointer = Qt.point(c.x + 99, c.y + 49);
                compare(s.ov.part, { box: { x: c.x, y: c.y, w: 100, h: 50 }, ring: false }, tag);
                s.app.pointer = Qt.point(c.x - 20, c.y - 10);
                compare(s.ov.part.box, { x: c.x - 20, y: c.y - 10, w: 21, h: 11 }, "both corners in, either way");
                const chip = Kit.named(s.ov.contentItem, "tag");
                verify(chip.visible && !Kit.named(s.ov.contentItem, "ring").visible, "a hole with its tag, no ring");
                compare(Kit.texts(chip, sized(tag, 21, 11)).length, 1, "its size in physical pixels");
                compare(chip.y, s.shade.hole.y - N.lk.shape.snip.tag_gap_px, "above the hole while there is room");
            }
        }

        function test_the_tag_moves_inside_a_hole_with_no_room_above() {
            const s = make("corner");
            const c = Snip.views.corner.pick.corner;
            s.app.pointer = Qt.point(c.x + 40, 0);
            const chip = Kit.named(s.ov.contentItem, "tag");
            const hole = s.shade.hole;
            verify(chip.y >= hole.y && chip.x >= hole.x, "inside its top left: " + [chip.x, chip.y]);
            compare(chip.y - hole.y, N.lk.shape.snip.tag_pad_px[0]);
        }

        function test_a_video_regions_corner_stops_at_its_first_corners_screen() {
            const s = make("video-corner");
            const r = Snip.views["video-corner"].pick.within;
            const c = Snip.views["video-corner"].pick.corner;
            s.app.pointer = Qt.point(r.x + r.w + 50, r.y + r.h + 50);
            const kept = { x: c.x, y: c.y, w: r.x + r.w - c.x, h: r.y + r.h - c.y };
            compare(s.ov.part.box, kept, "the moving corner stays on it");
            const still = make("corner");
            still.app.pointer = Qt.point(r.x + r.w + 50, r.y + r.h + 50);
            compare(still.ov.part.box.w, r.x + r.w + 50 - c.x + 1, "a picture's region may cross screens");
        }

        function test_the_window_way_rings_the_window_under_the_pointer_and_none_off_every_window() {
            for (const tag of Kit.langs) {
                const s = make("window", tag);
                const w = Snip.views.window.pick.windows[1];
                s.app.pointer = Qt.point(w.x + 1, w.y + 1);
                compare(s.ov.part, { box: w, ring: true }, tag);
                const ring = Kit.named(s.ov.contentItem, "ring");
                verify(ring.visible && !Kit.named(s.ov.contentItem, "tag").visible, "a ring and no tag");
                compare([ring.x, ring.y, ring.width, ring.height], [w.x, w.y, w.w, w.h].map(v => v / s.ov.dpr));
                s.app.pointer = Qt.point(5, 5);
                compare(s.ov.part, null, "no window under the pointer");
                verify(!ring.visible);
                compare(s.app.hint.said, Kit.text(tag, "snip.hint-window"));
            }
        }

        function test_a_whole_screen_way_rings_the_screen_under_the_pointer() {
            for (const tag of Kit.langs) {
                const s = make("full", tag);
                s.app.pointer = Qt.point(10, 10);
                compare(s.ov.part, { box: s.ov.screenInfo.rect, ring: true }, tag);
                verify(Kit.named(s.ov.contentItem, "ring").visible);
                compare(s.app.hint.said, Kit.text(tag, "snip.hint-full"));
            }
        }
    }
}
