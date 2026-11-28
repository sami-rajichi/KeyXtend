// Snip's editor maximised, over the stand-in Snip object (design "The editor"): it fills its screen's work area in each
// language and a restore brings back its box, each kept by the hub; it opens maximised next time, over that box; and
// with no screen to fit on, a restore keeps it where it shows.
import QtQuick
import QtTest
import "../qml"
import "snipkit.js" as Kit
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    Component { id: snipC; FakeSnip {} }
    Component { id: winC; FakeWin {} }
    Component { id: appC; SnipApp {} }

    TestCase {
        name: "snipmaximise"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in in language `tag`, once the editor shows, drawn and still.
        function make(tag) {
            const snip = createTemporaryObject(snipC, root);
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: N.lk, res: Kit.resOf(N.lk, tag || "en"), win: createTemporaryObject(winC, root), aim: () => {}
            });
            snip.page = Kit.view("edit");
            const ed = app.editor;
            tryVerify(() => ed.ready, 2000, "the editor shows, drawn and still");
            return { snip: snip, app: app, ed: ed, item: ed.editor.contentItem };
        }
        // The editor goes with its snip, then the next snip's Edit opens it again.
        function reopen(s) {
            s.snip.page = Kit.view("idle");
            tryVerify(() => !s.ed.editor.visible);
            s.snip.page = Kit.view("edit");
            tryVerify(() => s.ed.editor.visible);
        }

        function test_maximise_fills_the_work_area_and_restore_brings_back_the_box() {
            for (const tag of Kit.langs) {
                const s = make(tag);
                const was = s.ed.editor.place;
                compare(Kit.key(s.item, "snip.edit.full").icon, N.lk.icons.snip.edit.full, "Maximise's icon");
                mouseClick(Kit.key(s.item, "snip.edit.full"));
                compare(s.ed.editor.place, s.snip.work, tag);
                tryCompare(s.ed.editor, "width", Kit.shown(s.app, s.snip.work)[2]);
                const back = Kit.key(s.item, "snip.edit.full");
                compare(back.Accessible.name, Kit.text(tag, "snip.edit-back"));
                compare(back.icon, N.lk.icons.snip.edit.back, "Restore's own icon");
                verify(!Kit.key(s.item, "snip.edit.size") && !Kit.key(s.item, "snip.edit.move"), "a maximised window neither moves nor resizes");
                mouseClick(back);
                compare(s.ed.editor.place, was, "the box it had");
                compare(s.snip.heard("full"), [{ full: s.snip.work, from: was }], "the hub keeps the maximise");
                compare(s.snip.heard("placed"), [{ placed: was }], "and the restore");
            }
        }

        // The owner's check: maximised, closed, and opened by the next snip's Edit, it comes back maximised.
        function test_a_maximised_editor_opens_maximised_over_its_box_next_time() {
            const s = make();
            const was = s.ed.editor.place;
            mouseClick(Kit.key(s.item, "snip.edit.full"));
            reopen(s);
            verify(s.ed.full, "it opens maximised");
            compare([s.ed.editor.place, s.ed.editor.before], [s.snip.work, was], "over its box");
            tryCompare(s.ed.editor, "width", Kit.shown(s.app, s.snip.work)[2]);
            verify(!Kit.key(s.item, "snip.edit.size"), "with no resize corner");
            mouseClick(Kit.key(s.item, "snip.edit.full"));
            compare(s.ed.editor.place, was, "a restore brings back its box");
            reopen(s);
            verify(!s.ed.full, "and the next one opens unmaximised");
            compare(s.ed.editor.place, was);
        }

        function test_restore_with_no_screen_to_fit_on_keeps_the_editor_where_it_shows() {
            const s = make();
            mouseClick(Kit.key(s.item, "snip.edit.full"));
            const work = s.snip.work;
            s.snip.work = null;
            mouseClick(Kit.key(s.item, "snip.edit.full"));
            compare(s.ed.editor.place, work, "never hidden with no way back");
            verify(s.ed.full, "and a later click may still restore it");
            compare(s.snip.heard("placed"), [], "nor kept");
        }
    }
}
