// Snip's close question over the stand-in Snip object (design "Close"): its own window over the editor, which it dims,
// or over the part while the strip shows; its question and answers in English, French and Arabic; Save in a picture's
// editor flattening it first, a dimmed Save saying why, and the other answers going by their keys.
import QtQuick
import QtTest
import "../qml"
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
        name: "snipask"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` with `more` in language `tag`, once the question shows; aims
        // counted.
        function make(name, tag, more) {
            const snip = createTemporaryObject(snipC, root);
            const win = createTemporaryObject(winC, root);
            const aims = [];
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: N.lk, res: Kit.resOf(N.lk, tag || "en"), win: win, aim: () => aims.push(true)
            });
            snip.page = Kit.view(name, more);
            const q = app.ask.question;
            tryVerify(() => q.visible, 2000, "the question shows");
            waitForRendering(q.contentItem);
            return { snip: snip, app: app, q: q, item: q.contentItem, aims: aims };
        }
        // The centre of window `w` or rect `r`.
        function mid(r) {
            return [r.x + r.width / 2, r.y + r.height / 2];
        }
        function near(a, b) {
            return Math.abs(a[0] - b[0]) <= 1 && Math.abs(a[1] - b[1]) <= 1;
        }

        function test_over_the_editor_it_asks_in_each_language_and_dims_it() {
            for (const tag of Kit.langs) {
                const s = make("ask-edit", tag);
                compare(s.q.title, Kit.text(tag, "window.snip-ask"));
                verify(s.q.flags & Qt.WindowDoesNotAcceptFocus, "it never takes focus");
                compare(Kit.named(s.item, "card").Accessible.name, Kit.text(tag, "snip.ask"));
                const answers = Kit.keys(s.item, "snip.ask.");
                compare(answers.map(o => o.Accessible.name), Snip.ids.answers.map(id => Kit.text(tag, "snip.ask-" + id)), tag);
                const ed = s.app.editor.editor;
                verify(near(mid(s.q), mid(ed)), "centred over the editor");
                verify(s.app.editor.asking, "the editor dims under it");
            }
        }

        function test_save_in_a_pictures_editor_flattens_it_and_the_others_go_by_their_keys() {
            const s = make("ask-edit");
            mouseClick(Kit.key(s.item, "snip.ask.save"));
            tryVerify(() => s.snip.heard("flat").length === 1, 2000, "Save flattens the picture first");
            compare(s.snip.heard("flat")[0].flat, "save");
            mouseClick(Kit.key(s.item, "snip.ask.back"));
            mouseClick(Kit.key(s.item, "snip.ask.drop"));
            const r = Snip.views["ask-edit"].round;
            compare(s.snip.heard("act"), [{ act: "ask", id: "back", arg: 0, round: r }, { act: "ask", id: "drop", arg: 0, round: r }]);
            compare(s.aims.length, 3, "each press aims the letters at Snip");
        }

        function test_dont_save_and_save_let_the_picture_go() {
            for (const id of ["drop", "save"]) {
                const s = make("ask-edit");
                mouseClick(Kit.key(s.item, "snip.ask." + id));
                tryVerify(() => s.snip.heard("flat").length + s.snip.heard("act").length === 1, 2000, id);
                s.snip.page = Kit.view("idle");
                tryVerify(() => !s.app.editor.editor.visible && !s.q.visible);
                compare(String(s.app.editor.canvas.picture.source), "", id);
            }
        }

        function test_a_system_dialog_drops_it_from_the_top() {
            const s = make("ask-edit", "en", { low: true });
            verify(!(s.q.flags & Qt.WindowStaysOnTopHint), "the question leaves the top");
            verify(!(s.app.ask.flags & Qt.WindowStaysOnTopHint), "and its shadow");
        }

        function test_over_the_strip_it_sits_on_the_part_and_a_dimmed_save_says_why() {
            for (const tag of Kit.langs) {
                const s = make("ask-strip", tag);
                const v = Snip.views["ask-strip"];
                const [part, area] = [s.app.box(v.strip.part), s.app.box(v.strip.screen)];
                verify(s.q.x >= area.x && s.q.x + s.q.width <= area.x + area.width, "kept on the part's screen");
                verify(Math.abs(mid(s.q)[1] - mid(part)[1]) <= 1, "centred on the part");
                const save = Kit.key(s.item, "snip.ask.save");
                compare(save.Accessible.description, Kit.text(tag, "snip.dim-" + v.ask.save_dim), tag);
                mouseClick(save);
                compare(s.snip.heard("act"), [], "a dimmed Save never acts");
                mouseClick(Kit.key(s.item, "snip.ask.save"));
                mouseClick(Kit.key(s.item, "snip.ask.back"));
                compare(s.snip.heard("act"), [{ act: "ask", id: "back", arg: 0, round: v.round }]);
            }
        }
    }
}
