// Each Snip window over its one shadow, over the stand-in Snip object: the panel and the close question are made with
// the editor before they first show, so each window lies over its shadow from its first show, and a Save dialog, a
// maximise and a restore move neither window of the editor's pair.
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
        name: "snipstack"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in, with their stack as Qt would make and lift them.
        function make() {
            const snip = createTemporaryObject(snipC, root);
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: N.lk, res: Kit.resOf(N.lk, "en"), win: createTemporaryObject(winC, root), aim: () => {}
            });
            const pairs = Kit.owned(app);
            const z = Kit.stack([].concat(...pairs.map(([glow, w]) => [w, glow])), snip);
            return { snip: snip, app: app, ed: app.editor, pairs: pairs, z: z };
        }
        // Shows view `name` as round `round`, once its stage shows.
        function go(s, name, round) {
            s.snip.page = Kit.view(name, { round: round });
            tryCompare(s.app, "shown", Snip.views[name].stage);
        }
        // Clicks the editor's Maximise or Restore, then hands it the hub's new view, as a kept place brings one.
        function full(s, round) {
            mouseClick(Kit.key(s.ed.editor.contentItem, "snip.edit.full"));
            go(s, "edit", round);
        }

        function test_the_panel_and_the_question_are_made_with_their_shadow_before_they_show() {
            const s = make();
            const made = s.snip.heard("make").map(c => c.make);
            for (const w of [s.ed.editor, s.app.ask.question, s.app.panel.pane])
                verify(made.indexOf(w.title) >= 0 && !w.visible, w.title + " is made hidden");
        }

        // The owner's check: the panel looked dark, as its shadow lay over it from its first show.
        function test_each_window_lies_over_its_shadow_from_its_first_show() {
            for (const [name, i] of [["edit", 0], ["ask-edit", 1], ["panel-capture", 2], ["region", 3], ["strip", 4], ["record", 5]]) {
                const s = make();
                const [glow, w] = s.pairs[i];
                s.snip.page = Kit.view(name);
                tryVerify(() => w.visible, 2000, name + " shows " + w.title);
                verify(s.z.above(w, glow), w.title + " lies over its shadow at its first show");
            }
        }

        // The close question lies over the editor from its first show, so the editor's dim never takes its clicks.
        function test_the_close_question_lies_over_the_editor_from_its_first_show() {
            const s = make();
            go(s, "ask-edit", 1);
            tryVerify(() => s.app.ask.question.visible && s.ed.editor.visible, 2000, "the question shows over the editor");
            verify(s.z.above(s.app.ask, s.ed.editor), "its shadow lies over the editor");
            verify(s.z.above(s.app.ask.question, s.ed.editor), "and the question too");
        }

        // The owner's check: after a Save dialog, a maximise and a restore, the editor's shadow lay over it.
        function test_a_dialog_then_a_maximise_and_a_restore_keep_the_editor_over_its_shadow() {
            for (const dialog of [true, false]) {
                const s = make();
                go(s, "edit", 1);
                tryVerify(() => s.ed.ready, 2000, "the editor shows, drawn and still");
                if (dialog) {
                    go(s, "edit-low", 2);
                    go(s, "edit", 3);
                }
                verify(s.z.above(s.ed.editor, s.ed), "over its shadow before the maximise");
                let moved = 0;
                for (const w of [s.ed, s.ed.editor]) {
                    w.flagsChanged.connect(() => moved++);
                    w.visibleChanged.connect(() => moved++);
                }
                full(s, 4);
                verify(s.ed.full && s.z.above(s.ed.editor, s.ed), "maximised, still over its shadow");
                full(s, 5);
                verify(!s.ed.full && s.z.above(s.ed.editor, s.ed), "restored, still over its shadow");
                compare(moved, 0, "neither window changes its flags or shows again");
                verify(s.ed.editor.visible && s.ed.visible && s.ed.drawn, "both still show, the shadow drawn");
            }
        }
    }
}
