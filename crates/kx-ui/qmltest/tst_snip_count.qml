// The countdown over the stand-in Snip object (design/snip-parts.css .count), as the owner met it after a Cancel: its
// hide lets go of a press Windows never ends, the frozen screens hide in the turn the count comes, it draws once its
// pop rests and at each later second, and each second tells the debug log its frames and whether Cancel is held.
import QtQuick
import QtTest
import "../qml"
import "snipkit.js" as Kit
import "fixtures/snip.js" as Snip
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-reduced.js" as R

Item {
    id: root
    width: 200
    height: 100

    Component { id: snipC; FakeSnip {} }
    Component { id: appC; SnipApp {} }

    TestCase {
        name: "snipcount"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` in look `lk`.
        function make(name, lk) {
            const snip = createTemporaryObject(snipC, root);
            const look = lk || N.lk;
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: look, res: Kit.resOf(look, "en"), aim: () => {}
            });
            snip.page = Kit.view(name);
            tryVerify(() => app.shown === app.stage, 2000, "the stage shows");
            return { snip: snip, app: app };
        }
        // View `name` as round `round`, once its stage shows.
        function go(s, name, round) {
            s.snip.page = Kit.view(name, { round: round });
            tryCompare(s.app, "shown", Snip.views[name].stage);
        }
        // The countdown's second `left` as round `round`.
        function second(s, left, round) {
            s.snip.page = Kit.view("count", { round: round, count: { part: Snip.views.count.count.part, left_s: left } });
        }

        // The owner's check: Cancel ends a countdown at its press, and once the countdown hid Windows sends its let-go
        // elsewhere or never; the next one shows its disc and Cancel, at rest and not held, for its whole count.
        function test_a_countdown_after_a_cancel_shows_its_disc_for_the_whole_count_data() {
            return [{ tag: "let go on the hidden countdown", letGo: true }, { tag: "never let go", letGo: false }];
        }
        function test_a_countdown_after_a_cancel_shows_its_disc_for_the_whole_count(data) {
            const s = make("count");
            const win = s.app.count;
            tryVerify(() => win.visible, 2000, "the first countdown");
            const cancel = Kit.buttons(win)[0];
            mousePress(cancel);
            verify(cancel.down, "held");
            s.snip.page = Kit.view("idle", { round: 30 });
            tryVerify(() => !win.visible, 2000, "it ends at the press");
            verify(!cancel.down, "its hide lets go of the press");
            if (data.letGo)
                mouseRelease(cancel);
            go(s, "region", 31);
            go(s, "ready", 32);
            const body = win.contentItem.children[0];
            let hides = 0;
            win.visibleChanged.connect(() => hides += win.visible ? 0 : 1);
            for (const left of [3, 2, 1]) {
                second(s, left, 36 - left);
                tryVerify(() => win.visible && body.opacity === 1 && body.scale === 1, 2000, left + " s left, at rest");
                verify(Kit.texts(win.contentItem, String(left)).length === 1, "its seconds");
                verify(!Kit.buttons(win)[0].down, "its Cancel shows, not held");
            }
            compare(hides, 0, "it shows for the whole count");
            compare(s.snip.heard("act").length, 1, "the cut press acted once");
        }

        // Record's countdown drops the frozen screens: they hide in that turn, not once Qt frees their windows later.
        function test_the_frozen_screens_hide_in_the_turn_the_count_drops_them() {
            const s = make("ready");
            const frozen = s.app.overlays.objectAt(0);
            verify(frozen.visible, "the frozen screen shows");
            second(s, 3, 40);
            verify(!frozen.visible, "hidden in the same turn");
            verify(s.app.count.visible, "the countdown shows");
        }

        // Its pop rests on time whether frames came or not, then it asks to draw again: at the pop's end, or at once
        // with reduced motion.
        function test_the_countdown_rests_and_draws_again_once_its_pop_ends_data() {
            return [{ tag: "moving", lk: N.lk }, { tag: "reduced", lk: R.lk }];
        }
        function test_the_countdown_rests_and_draws_again_once_its_pop_ends(data) {
            const s = make("count", data.lk);
            const win = s.app.count;
            const body = win.contentItem.children[0];
            tryVerify(() => s.snip.heard("draw").length === 1, 2000, "it asks to draw again");
            compare(s.snip.heard("draw")[0].draw, win.title);
            compare([body.opacity, body.scale], [1, 1], "at rest");
        }

        // Each later second asks to draw in its turn once its digit shows, so a lost update request never leaves an old
        // second up and the frame forced holds the new one.
        function test_each_later_second_draws_once_its_digit_shows() {
            const s = make("ready");
            const win = s.app.count;
            second(s, 3, 37);
            tryVerify(() => win.visible && s.snip.heard("draw").length === 1, 2000, "the pop's rest draws 3 s");
            const digit = Kit.texts(win.contentItem, "3")[0];
            const drawn = [];
            s.snip.callsChanged.connect(() => {
                const call = s.snip.calls[s.snip.calls.length - 1];
                if (call.draw !== undefined)
                    drawn.push(digit.text);
            });
            for (const left of [2, 1])
                second(s, left, 40 - left);
            compare(drawn, ["2", "1"], "each draw asked in its turn holds its new second");
            verify(s.snip.heard("draw").every(d => d.draw === win.title), "the countdown's own window");
        }

        // Each second tells the debug log its draws since it showed and whether Cancel is held, with the frozen
        // screens' title, which the hub counts those still up by; its hide tells it once more, as 0 s.
        function test_each_count_second_tells_its_draws_since_it_showed() {
            const s = make("ready");
            const win = s.app.count;
            for (const left of [3, 2, 1]) {
                second(s, left, 40 - left);
                tryVerify(() => win.visible, 2000, left + " s left");
                waitForRendering(win.contentItem);
            }
            go(s, "idle", 40);
            const told = s.snip.heard("counted");
            compare(told.map(c => c.counted), [3, 2, 1, 0], "each second, then its hide");
            verify(told.every(c => c.held === false && c.screens === s.app.screenTitle), JSON.stringify(told));
            compare(told[0].draws, 0, "none yet as it shows");
            verify(told[1].draws >= 1 && told[2].draws > told[1].draws, "each second's own draw counts");
            const drawn = s.snip.heard("draw").filter(d => d.draw === win.title).length;
            compare(told[3].draws, drawn, "every draw since it showed, its pop's rest too");
        }
    }
}
