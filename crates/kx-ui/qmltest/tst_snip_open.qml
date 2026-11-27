// Snip's windows as they open, over the stand-in Snip object: no shadow draws before the window it owns has drawn, and
// each window shows in the turn its shadow does; the editor shows in the turn its picture is ready, and it, the
// question and the panel come from windows made once before their first show; the capture bar's and the editor's
// opens are timed.
import QtQuick
import QtTest
import "../qml"
import "snipkit.js" as Kit
import "fixtures/snip.js" as Snip
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-reduced.js" as R

Item {
    id: root
    width: 200
    height: 100

    Component { id: snipC; FakeSnip {} }
    Component { id: winC; FakeWin {} }
    Component { id: appC; SnipApp {} }

    TestCase {
        name: "snipopen"
        when: windowShown

        // The view that shows each shadow with its window, in snipkit's order: editor, question, panel, capture bar,
        // strip, record bar.
        readonly property var views: ["edit", "ask-edit", "panel-capture", "region", "strip", "record"]
        // A picture served at once, as the hub's provider serves the editor's; the tests' data URL loads later.
        readonly property string served: Assets.assets.icons + "/000000/" + N.lk.icons.snip.edit.close + "." + Assets.assets.ext

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in with no view yet, in look `lk` (the full-motion one by default).
        function make(lk) {
            const look = lk || N.lk;
            const snip = createTemporaryObject(snipC, root);
            const win = createTemporaryObject(winC, root);
            const app = createTemporaryObject(appC, root, { snip: snip, lk: look, res: Kit.resOf(look, "en"), win: win });
            return { snip: snip, app: app };
        }
        // The editor's view, its picture at `src` when given.
        function edit(src) {
            const v = Kit.view("edit");
            if (src)
                v.edit.src = src;
            return v;
        }
        // The open times the stand-in heard, once there is one.
        function timed(s) {
            tryVerify(() => s.snip.heard("opened").length === 1, 2000, "the open is timed once");
            return s.snip.heard("opened")[0];
        }

        function test_no_shadow_draws_before_its_window_and_each_shows_with_it() {
            views.forEach((name, i) => {
                const s = make();
                const [glow, w] = Kit.owned(s.app)[i];
                let drew = false;
                const early = [];
                w.frameSwapped.connect(() => drew = true);
                glow.frameSwapped.connect(() => {
                    if (!w.visible)
                        early.push("a frame while its window hides");
                    else if (!drew && glow.shade.visible && glow.shade.opacity > 0)
                        early.push("its shade before its window drew");
                });
                s.snip.page = Kit.view(name);
                tryVerify(() => drew && glow.shade.visible, 2000, name + ": both draw");
                compare(early, [], w.title);
            });
        }

        // With reduced motion no pop hides the shade at first, so only the frame gate keeps it from drawing first.
        function test_each_shade_waits_for_its_windows_first_frame_with_full_and_reduced_motion() {
            for (const lk of [N.lk, R.lk]) {
                const pop = lk.motion.moves.snip_pop.ms;
                views.forEach((name, i) => {
                    const s = make(lk);
                    const [glow, w] = Kit.owned(s.app)[i];
                    let frames = 0;
                    const shown = [];
                    w.frameSwapped.connect(() => frames++);
                    glow.visibleChanged.connect(() => shown.push([w.visible, glow.drawn, glow.shade.visible]));
                    const v = Kit.view(name);
                    if (v.edit)
                        v.edit.src = served;
                    s.snip.page = v;
                    const tag = name + " at a " + pop + " ms pop: ";
                    tryVerify(() => shown.length > 0, 2000, tag + "it shows");
                    compare(shown[0], [true, false, false], tag + "its window shows in its turn, and the shade waits");
                    tryVerify(() => glow.shade.visible, 2000, tag + "the shade shows");
                    verify(frames > 0 && glow.drawn, tag + "after the window drew");
                });
            }
        }

        function test_the_editor_shows_in_the_turn_its_view_comes_when_its_picture_is_served_at_once() {
            for (const before of ["idle", "strip"]) {
                const s = make();
                s.snip.page = Kit.view(before);
                tryCompare(s.app, "shown", before);
                s.snip.page = edit(served);
                const ed = s.app.editor;
                compare(ed.canvas.picture.status, Image.Ready, "served at once");
                verify(ed.visible && ed.editor.visible, "after " + before + ", no turn waits");
            }
        }

        // The tests' data URL goes through Qt's pixmap reader, uncached, so it is still loading in the turn its view
        // comes.
        function test_the_editor_stays_hidden_while_its_picture_loads_then_shows_as_it_is_ready() {
            const s = make();
            const ed = s.app.editor;
            const at = [];
            ed.editor.visibleChanged.connect(() => at.push(ed.canvas.picture.status));
            s.snip.page = edit();
            compare(ed.canvas.picture.status, Image.Loading, "the picture is still loading in the view's turn");
            verify(!ed.visible && !ed.editor.visible, "the editor and its shadow wait for it");
            tryVerify(() => ed.editor.visible, 2000);
            compare(at, [Image.Ready], "it shows once its picture is ready");
        }

        // The editor, the close question and the panel, which no capture's shield makes.
        function test_the_shadowed_windows_are_made_once_before_their_first_show() {
            const s = make();
            const made = ["window.snip-editor", "window.snip-ask", "window.snip-panel"].map(k => Kit.text("en", k)).sort();
            const heard = () => s.snip.heard("make").map(c => c.make).sort();
            compare(heard(), made, "as Snip's windows start");
            for (const name of ["edit", "ask-edit", "idle", "panel-capture", "edit"]) {
                s.snip.page = Kit.view(name);
                tryCompare(s.app, "shown", Snip.views[name].stage);
            }
            tryVerify(() => s.app.editor.editor.visible);
            compare(heard(), made, "never again");
        }

        function test_the_editors_open_is_timed_from_the_edit_click() {
            const s = make();
            s.snip.page = Kit.view("strip");
            tryCompare(s.app, "shown", "strip");
            s.app.act(s.app.names.acts.strip, "edit");
            s.snip.page = edit(served);
            const t = timed(s);
            compare(t.opened, s.app.names.stages.edit);
            verify(t.click >= 0 && t.shown >= 0 && t.shown <= t.drawn && t.drawn <= t.ready, JSON.stringify(t));
            verify(s.app.editor.ready, "ready once drawn and its pop ended");
            verify([t.click, t.shown, t.drawn, t.ready].every(Number.isInteger), "whole ms");
        }

        function test_the_bars_open_is_timed_from_its_view_as_no_snip_click_led_to_it() {
            const s = make();
            s.snip.page = Kit.view("region");
            const t = timed(s);
            compare([t.opened, t.click], [s.app.names.stages.pick, -1]);
            verify(t.shown >= 0 && t.shown <= t.drawn && t.drawn <= t.ready, JSON.stringify(t));
            verify(s.app.bar.ready);
        }
    }
}
