// The editor's picture over the stand-in Snip object (design "The editor"): each drawing where the hub says, text from
// its top left or its top right in Arabic (edge 24), a step centred, a circle in its box, an arrow's head at its tip, the
// Highlighter at its width, a step past the list's end at the last size, the caret only while the letters come to Snip,
// clicks sent in window pixels from the whole picture's corner, a pen's path on its second click, and Copy flattening
// the kept part at the picture's own pixels.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "pointer.js" as Pointer
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
    Component { id: imageC; Image {} }

    TestCase {
        name: "snipcanvas"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` (or that view itself) in language `tag`, once the editor
        // shows, drawn and still, so a click lands where its pop-in ends. The real pointer passes through the editor
        // from before it shows, so only the test's own moves reach the canvas.
        function make(name, tag) {
            const snip = createTemporaryObject(snipC, root);
            const win = createTemporaryObject(winC, root);
            const app = createTemporaryObject(appC, root, { snip: snip, lk: N.lk, res: Kit.resOf(N.lk, tag || "en"), win: win });
            Pointer.through([app.editor.editor]);
            snip.page = typeof name === "string" ? Kit.view(name) : name;
            const ed = app.editor;
            tryVerify(() => ed.ready, 2000, "the editor shows, drawn and still");
            return { snip: snip, win: win, app: app, c: ed.canvas, sent: app.sent };
        }
        // Drawing `i` of view `name`, and its mark on the canvas.
        function drawn(name, i) {
            return Snip.views[name].edit.picture.drawings[i];
        }
        function near(a, b) {
            return Math.abs(a - b) < 0.5;
        }

        function test_text_starts_at_its_corner_and_arabic_reads_from_the_right() {
            for (const tag of Kit.langs) {
                const s = make("edit", tag);
                const [ltr, rtl] = [s.c.marks.itemAt(7).words, s.c.marks.itemAt(8).words];
                const [a, b] = [drawn("edit", 7).mark, drawn("edit", 8).mark];
                compare([ltr.x, ltr.y], [a.at.x, a.at.y], "top left");
                verify(near(rtl.x + rtl.width, b.at.x) && rtl.y === b.at.y, "top right when it reads right to left");
                verify(rtl.isRightToLeft(0, rtl.length), "Arabic lays out right to left");
                compare(rtl.effectiveHorizontalAlignment, TextEdit.AlignRight);
                compare(rtl.font.family, Kit.resOf(N.lk, tag).fam.arabic);
                compare(rtl.text, b.text);
                verify(Qt.colorEqual(ltr.color, N.lk.common.snip_ink_red), "in its ink");
            }
        }

        function test_each_shape_lies_where_its_clicks_were() {
            const s = make("edit");
            const step = s.c.marks.itemAt(9);
            const at = drawn("edit", 9).mark.at;
            compare([step.disc.x + step.disc.width / 2, step.disc.y + step.disc.height / 2], [at.x, at.y], "a step is centred");
            compare(step.disc.width, s.sent.thick[0].disc);
            const circle = s.c.marks.itemAt(3);
            const c = drawn("edit", 3).mark;
            const arc = circle.arc;
            compare([arc.centerX, arc.centerY], [(c.from.x + c.to.x) / 2, (c.from.y + c.to.y) / 2], "a circle fills its box");
            compare([arc.radiusX, arc.radiusY], [Math.abs(c.to.x - c.from.x) / 2, Math.abs(c.to.y - c.from.y) / 2]);
            const arrow = s.c.marks.itemAt(4);
            const tip = drawn("edit", 4).mark.to;
            const head = arrow.lines()[1];
            const reach = s.sent.thick[drawn("edit", 4).size].head;
            verify(head.every(p => Math.hypot(p.x - tip.x, p.y - tip.y) <= reach + 0.01), "the head lies within its length of the tip");
            compare([head[1].x, head[1].y], [tip.x, tip.y]);
            const marker = s.c.marks.itemAt(1).stroke;
            compare(marker.strokeWidth, s.sent.thick[1].marker, "the Highlighter at its width");
            verify(Math.abs(marker.strokeColor.a - N.lk.shape.snip.marker_share) < 0.01, "and see-through");
            const cover = Board.all(s.c.marks.itemAt(6), o => o.visible && o.color !== undefined && o.width === 90)[0];
            verify(!!cover && Qt.colorEqual(cover.color, N.lk.common.snip_ink_black), "a cover fills its box");
        }

        function test_a_step_past_the_lists_end_takes_its_last_size() {
            const s = make("edit");
            const [text, step] = [s.c.marks.itemAt(8), s.c.marks.itemAt(10)];
            compare(text.words.font.pixelSize, s.sent.text[s.sent.text.length - 1].height);
            compare(step.disc.width, s.sent.thick[s.sent.thick.length - 1].disc);
        }

        function test_a_step_number_takes_its_inks_number_colour() {
            const s = make("edit");
            const number = i => Kit.named(s.c.marks.itemAt(i), "number");
            verify(Qt.colorEqual(number(9).color, N.lk.common.snip_num_blue), "on blue");
            verify(Qt.colorEqual(number(10).color, N.lk.common.snip_num_yellow), "on yellow");
            compare(number(10).text, "2");
        }

        function test_the_caret_shows_only_while_the_letters_come_to_snip() {
            for (const tag of Kit.langs) {
                const s = make("typing", tag);
                const t = Snip.views.typing.edit.picture.typing;
                const words = s.c.typer.words;
                verify(near(words.x + words.width, t.at.x), "Arabic being typed reads from its top right");
                verify(words.isRightToLeft(0, words.length));
                const caret = Kit.named(words, "caret");
                verify(!caret.visible, "no caret while the letters go elsewhere");
                s.win.aim = s.app.names.aim;
                verify(caret.visible, "the caret while they come to Snip");
                compare(caret.x, words.positionToRectangle(t.caret).x);
            }
        }

        function test_a_click_goes_in_window_pixels_from_the_whole_pictures_corner() {
            const s = make("cropped");
            const crop = Snip.views.cropped.edit.picture.crop;
            compare([s.c.shown.w, s.c.shown.h], [crop.w, crop.h], "the kept part shows");
            mouseMove(s.c.view, 15, 25);
            verify(!!s.c.preview && s.c.preview.mark.shape === "box", "a shape draws on to the pointer from its first click");
            mouseClick(s.c.view, 10, 20);
            const k = s.c.k;
            const press = s.snip.heard("press");
            compare(press.length, 1);
            verify(near(press[0].press[0], 10 + crop.x * k) && near(press[0].press[1], 20 + crop.y * k), JSON.stringify(press));
            compare([press[0].scale, press[0].round], [k, Snip.views.cropped.round]);

        }

        function test_crop_shows_the_part_kept_as_every_tool_does() {
            const s = make("crop");
            const crop = Snip.views.crop.edit.picture.crop;
            verify(crop.w < Snip.views.crop.edit.width, "a part was kept");
            compare([s.c.shown.x, s.c.shown.y, s.c.shown.w, s.c.shown.h], [crop.x, crop.y, crop.w, crop.h]);
        }

        function test_a_pens_second_click_sends_its_path() {
            const s = make("pending");
            mouseMove(s.c.view, 30, 30);
            mouseMove(s.c.view, 40, 35);
            verify(!!s.c.preview && s.c.preview.mark.points.length >= 4, "the path draws from the first click");
            mouseClick(s.c.view, 50, 40);
            const strokes = s.snip.heard("stroke");
            compare(strokes.length, 1);
            const pts = strokes[0].stroke;
            verify(near(pts[0][0], 30) && near(pts[1][0], 40), "each point since the first click");
            verify(near(pts[pts.length - 1][0], 50) && near(pts[pts.length - 1][1], 40), "and the last: " + JSON.stringify(pts));
            compare(strokes[0].scale, s.c.k);
            compare(s.snip.heard("press"), [], "the second click ends the path");
        }

        function test_a_click_on_the_text_being_typed_moves_its_caret() {
            const s = make("typing");
            const words = s.c.typer.words;
            const p = words.mapToItem(s.c.view, 1, words.height / 2);
            mouseClick(s.c.view, p.x, p.y);
            const acts = s.snip.heard("act");
            compare(acts.length, 1);
            compare(acts[0].act, "caret");
            compare(acts[0].arg, words.positionAt(1, words.height / 2));
        }

        // The picture view `v` flattens to on Copy: its pixel size, and the size the canvas grabbed at.
        function copied(v) {
            const s = make(v);
            let grab = null;
            s.c.flattened.connect(g => grab = g);
            mouseClick(Kit.key(s.app.editor.editor.contentItem, "snip.keep.copy"));
            tryVerify(() => s.snip.heard("flat").length === 1, 2000, "the flattened picture goes to the hub");
            const f = s.snip.heard("flat")[0];
            compare([f.flat, f.round], ["copy", v.round]);
            verify(grab !== null, "the canvas tells the flattened picture and keeps none");
            verify(!("grabbed" in s.c), "no property holds the last picture");
            const img = createTemporaryObject(imageC, root, { source: grab.url });
            tryCompare(img, "status", Image.Ready);
            verify(!s.c.flattening, "the picture shows as it was");
            return { w: img.sourceSize.width, h: img.sourceSize.height, at: s.c.grabbing };
        }

        function test_copy_flattens_the_kept_part_at_its_own_pixels() {
            for (const name of ["edit", "cropped", "crop", "typing"]) {
                const got = copied(Kit.view(name));
                const crop = Snip.views[name].edit.picture.crop;
                compare([got.w, got.h], [crop.w, crop.h], name);
                compare([got.at.width, got.at.height], [got.w, got.h], "one picture pixel to an image pixel");
            }
        }

        function test_an_uneven_part_flattens_one_to_one_and_at_most_a_pixel_over() {
            const v = Kit.view("cropped");
            v.edit.picture.crop = { x: 50, y: 40, w: 202, h: 121 };
            const got = copied(v);
            compare([got.at.width, got.at.height], [got.w, got.h], "one picture pixel to an image pixel");
            verify(got.w >= 202 && got.w <= 203 && got.h >= 121 && got.h <= 122, "the hub cuts the rest: " + [got.w, got.h]);
            if (root.Window.window.devicePixelRatio === 1)
                compare([got.w, got.h], [202, 121], "exact at 100 %");
        }

        function test_a_flatten_leaves_out_the_outline_the_caret_and_the_next_mark() {
            const s = make("typing");
            s.win.aim = s.app.names.aim;
            const [box, caret] = [Kit.named(s.c.typer, "typing"), Kit.named(s.c.typer, "caret")];
            verify(box.visible && caret.visible, "both show while typing");
            s.c.flatten("copy");
            verify(s.c.flattening && !box.visible && !caret.visible, "neither goes into the picture");
            tryVerify(() => s.snip.heard("flat").length === 1, 2000);
            verify(box.visible && caret.visible, "both come back");
            const p = make("pending");
            mouseMove(p.c.view, 30, 30);
            const next = Kit.named(p.c, "next");
            verify(next.visible, "the next mark draws to the pointer");
            p.c.flatten("copy");
            verify(!next.visible, "but not into the picture");
            tryVerify(() => p.snip.heard("flat").length === 1, 2000);
        }

        function test_a_grab_that_cannot_start_shows_the_picture_as_it_was() {
            const s = make("typing");
            s.app.editor.editor.visible = false;
            ignoreWarning(/grabToImage/);
            s.c.flatten("copy");
            verify(!s.c.flattening, "the outline and the caret come back at once");
            compare(s.snip.heard("flat").length, 0, "nothing goes to the hub");
        }

        function test_the_typing_outline_dashes_in_window_pixels() {
            const s = make("typing");
            s.app.taken = 2;
            tryVerify(() => s.c.k <= 0.5, 2000, "the picture shows at half its size");
            const outline = Kit.named(s.c.typer, "typing");
            const path = outline.path;
            const dash = path.dashPattern.map(d => d * path.strokeWidth * s.c.k);
            const want = N.lk.shape.snip.dash_px;
            verify(dash.every((d, i) => Math.abs(d - want[i]) < 0.01), "as long at any zoom: " + dash);
            const gap = -outline.anchors.margins * s.c.k;
            verify(Math.abs(gap - N.lk.shape.snip.typing_gap_px) < 0.01, "and as far from the text: " + gap);
        }
    }
}
