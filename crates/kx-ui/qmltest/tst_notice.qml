// The notice banner over the stand-in Ui: it queues and counts what waits, offers Try again only when the app can
// restart what failed and Yes and No when a notice asks, answers each notice once, tells its notices again in a new
// keyboard language, shows again what the hub still shows when made again, and never takes focus.
import QtQuick
import QtTest
import "../qml"
import "fixtures/assets.js" as Assets
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    readonly property var res: ({
        fam: { latin: "Segoe UI", arabic: "Segoe UI", mark: "Segoe UI", ui: "Segoe UI" },
        icons: Assets.assets.icons,
        ext: Assets.assets.ext,
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k
    })
    // Where the stand-in spot puts the banner.
    readonly property point at: Qt.point(20, 30)

    Component { id: uiC; FakeUi {} }
    Component { id: noteC; Notice {} }
    Component { id: spyC; SignalSpy {} }
    // Hands the keys it takes to item `to`, as a key that reaches the banner would.
    Component { id: keysC; Item { property Item to; Keys.forwardTo: [to] } }

    TestCase {
        name: "notice"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A banner over a stand-in Ui holding `waiting`, as wide as `widest` at most (the panel's width by default).
        function make(waiting, props, widest) {
            const ui = createTemporaryObject(uiC, root, Object.assign({ notices: waiting }, props || {}));
            return createTemporaryObject(noteC, root, {
                ui: ui, lk: N.lk, res: root.res, s: 1, widest: widest || N.lk.shape.panel.width_px,
                spot: (w, h) => root.at
            });
        }
        function notice(id, retry) {
            return { id: id, text: "Notice " + id, retry: retry };
        }
        // A notice that asks, with the hub's Yes and No texts in the language of fixture `t` (English by default).
        function asking(id, t) {
            const texts = (t || En).texts;
            return { id: id, text: "Ask " + id, retry: false, ask: { yes: texts["button.yes"], no: texts["button.no"] } };
        }
        // Every item under `item` for which `ok` holds.
        function all(item, ok, found) {
            found = found || [];
            if (ok(item))
                found.push(item);
            for (let i = 0; i < item.children.length; i++)
                all(item.children[i], ok, found);
            return found;
        }
        // The banner's buttons a screen reader meets, by name.
        function buttons(note) {
            return all(note.contentItem, o => o.Accessible.role === Accessible.Button && o.visible).map(o => o.Accessible.name);
        }
        function button(note, name) {
            return all(note.contentItem, o => o.Accessible.role === Accessible.Button && o.Accessible.name === name)[0];
        }

        // A banner made while the hub still shows notices, as after a device loss, shows them again before those waiting.
        function test_a_banner_made_again_shows_what_the_hub_still_shows() {
            const note = make([notice(3, false)], { retold: [notice(1, false), notice(2, true)] });
            tryVerify(() => note.visible, 2000, "it shows");
            compare(note.queue.map(n => n.id), [1, 2, 3]);
            compare(note.shown.id, 1);
        }

        function test_the_banner_queues_and_counts_what_waits() {
            const note = make([notice(1, false), notice(2, true)]);
            tryVerify(() => note.visible, 2000, "it shows");
            compare([note.shown.id, note.more], [1, En.more[1]]);
            compare([note.x, note.y], [root.at.x, root.at.y], "where the spot says");
            note.ui.notices = [notice(3, false)];
            note.ui.noticesWaiting();
            compare([note.queue.length, note.more], [3, En.more[2]], "a later notice queues");
            waitForRendering(note.contentItem);
            mouseClick(button(note, En.texts["button.ok"]));
            compare(note.ui.calls, ["dismiss 1"]);
            compare([note.shown.id, note.more], [2, En.more[1]], "the next one shows");
        }

        function test_try_again_shows_only_when_the_app_can_restart() {
            const note = make([notice(7, true), notice(8, false)]);
            tryVerify(() => note.visible, 2000, "it shows");
            compare(buttons(note), [En.texts["button.retry"], En.texts["button.ok"]]);
            waitForRendering(note.contentItem);
            mouseClick(button(note, En.texts["button.retry"]));
            compare(note.ui.calls, ["retry 7"]);
            compare(buttons(note), [En.texts["button.ok"]], "a notice with nothing to restart");
        }

        function test_the_last_answer_hides_the_banner_and_its_shadow() {
            const note = make([notice(1, false)]);
            const spy = createTemporaryObject(spyC, root, { target: note, signalName: "showing" });
            tryVerify(() => note.visible, 2000, "it shows");
            compare(note.more, "", "nothing else waits");
            waitForRendering(note.contentItem);
            mouseClick(button(note, En.texts["button.ok"]));
            verify(!note.visible && note.shown === null);
            compare([spy.count, spy.signalArguments[0][0], spy.signalArguments[1][0]], [2, true, false]);
            note.ui.noticesWaiting();
            verify(!note.visible, "nothing new to show");
        }

        function test_it_takes_no_focus_and_readers_hear_the_notice() {
            const note = make([notice(1, false), notice(2, false)]);
            tryVerify(() => note.visible, 2000, "it shows");
            verify(note.flags & Qt.WindowDoesNotAcceptFocus, "never takes focus");
            compare(note.title, En.texts["window.notice"]);
            const alert = all(note.contentItem, o => o.Accessible.role === Accessible.AlertMessage)[0];
            compare([alert.Accessible.name, alert.Accessible.description], ["Notice 1", En.more[1]]);
        }

        function test_a_long_notice_wraps_inside_the_widest_width() {
            const long = { id: 1, text: En.texts["notice.settings-locked"], retry: true };
            const note = make([long], {}, 300);
            tryVerify(() => note.visible, 2000, "it shows");
            verify(note.width <= 300, "no wider: " + note.width);
            const one = make([notice(2, false)]);
            tryVerify(() => one.visible, 2000, "the short one shows");
            verify(note.height > one.height, "a long notice takes more lines");
        }

        function test_a_double_click_answers_one_notice() {
            const note = make([notice(1, false), notice(2, false)]);
            tryVerify(() => note.visible, 2000, "it shows");
            waitForRendering(note.contentItem);
            mouseDoubleClickSequence(button(note, En.texts["button.ok"]));
            compare(note.ui.calls, ["dismiss 1"]);
            compare(note.shown.id, 2, "the next one waits to be read");
        }

        function test_the_banner_follows_its_spot_when_it_changes_size_or_is_asked() {
            let at = Qt.point(20, 30);
            const note = make([notice(1, false)]);
            note.spot = (w, h) => at;
            tryVerify(() => note.visible, 2000, "it shows");
            at = Qt.point(50, 60);
            note.follow();
            compare([note.x, note.y], [50, 60], "the keyboard moved");
            at = Qt.point(70, 80);
            note.ui.notices = [notice(2, true)];
            note.ui.noticesWaiting();
            compare(note.more, En.more[1]);
            tryCompare(note, "x", 70, 2000, "its size changed");
        }

        // Arabic flips text only (ADR-0020): the icon stays on the left and the buttons on the right.
        function test_an_arabic_notice_keeps_the_banners_places_and_reads_right_to_left() {
            for (const t of [En, Ar]) {
                const text = t.texts["notice.keys-not-sent"];
                const note = make([{ id: 1, text: text, retry: false }]);
                tryVerify(() => note.visible, 2000, "it shows");
                waitForRendering(note.contentItem);
                const icon = all(note.contentItem, o => o.tint !== undefined && o.icon === N.lk.icons.notice.icon)[0];
                const ok = button(note, En.texts["button.ok"]);
                const [i, b] = [icon, ok].map(o => o.mapToItem(note.contentItem, 0, 0).x);
                verify(i < b, "icon " + i + ", button " + b);
                const msg = all(note.contentItem, o => o.text === text && o.wrapMode !== undefined)[0];
                compare(msg.effectiveHorizontalAlignment, t === Ar ? Text.AlignRight : Text.AlignLeft, t.lang.tag);
            }
        }

        function test_a_new_keyboard_language_tells_the_queued_notices_again() {
            const note = make([notice(1, false), notice(2, true)]);
            tryVerify(() => note.visible, 2000, "it shows");
            const retold = id => ({ id: id, text: Fr.texts["notice.keys-not-sent"] + " " + id, retry: id === 2 });
            note.ui.retold = [retold(1), retold(2)];
            note.ui.texts = Fr.texts;
            compare(note.queue, [retold(1), retold(2)]);
            compare(note.shown.text, retold(1).text, "the banner reads in French at once");
            note.ui.retold = [retold(2)];
            note.ui.texts = En.texts;
            const texts = note.queue.map(n => n.text);
            compare(texts, [retold(1).text, retold(2).text], "a notice the hub does not tell again keeps its text");
        }

        function test_a_press_on_the_banner_is_seen_and_still_reaches_its_button() {
            const note = make([notice(1, false)]);
            const spy = createTemporaryObject(spyC, root, { target: note, signalName: "touched" });
            tryVerify(() => note.visible, 2000, "it shows");
            waitForRendering(note.contentItem);
            mouseClick(button(note, En.texts["button.ok"]));
            compare(spy.count, 1, "seen, so the tray menu closes");
            compare(note.ui.calls, ["dismiss 1"], "and OK still answers");
        }

        function test_an_asking_notice_offers_yes_first_and_a_plain_one_ok_as_before() {
            const note = make([asking(1), notice(2, false)]);
            tryVerify(() => note.visible, 2000, "it shows");
            const [yes, no] = [En.texts["button.yes"], En.texts["button.no"]];
            compare(buttons(note), [yes, no]);
            verify(button(note, yes).main && !button(note, no).main, "Yes is the main button");
            waitForRendering(note.contentItem);
            mouseClick(button(note, yes));
            compare(note.ui.calls, ["answer 1 true"]);
            compare(buttons(note), [En.texts["button.ok"]], "the plain notice after it");
        }

        // Only Yes and No answer a question: no OK, close or title bar beside them, so No is always a deliberate click.
        function test_an_asking_notice_offers_its_two_answers_alone() {
            const note = make([asking(1)]);
            tryVerify(() => note.visible, 2000, "it shows");
            compare(buttons(note), [En.texts["button.yes"], En.texts["button.no"]], "no OK or close");
            const clickable = all(note.contentItem, o => o.visible && o.enabled && o.acceptedButtons !== undefined);
            compare(clickable.length, 2, "nothing else takes a click");
            verify(note.flags & Qt.FramelessWindowHint, "no title bar with a close button");
        }

        function test_a_double_click_on_yes_sends_one_answer() {
            const note = make([asking(1), asking(2)]);
            tryVerify(() => note.visible, 2000, "it shows");
            waitForRendering(note.contentItem);
            mouseDoubleClickSequence(button(note, En.texts["button.yes"]));
            compare(note.ui.calls, ["answer 1 true"]);
            compare(note.shown.id, 2, "the next question waits to be read");
        }

        function test_no_and_esc_answer_no() {
            const note = make([asking(1), asking(2), asking(3)]);
            const keys = createTemporaryObject(keysC, root, { to: note.body });
            tryVerify(() => note.visible, 2000, "it shows");
            waitForRendering(note.contentItem);
            mouseClick(button(note, En.texts["button.no"]));
            keys.forceActiveFocus();
            keyClick(Qt.Key_Escape);
            compare(note.ui.calls, ["answer 1 false", "answer 2 false"]);
            compare(note.shown.id, 3);
        }

        // Esc answers the question shown with No once: the next Esc reaches the plain notice after it, then nothing.
        function test_esc_on_a_question_answers_no_once() {
            const note = make([asking(1), notice(2, false)]);
            const keys = createTemporaryObject(keysC, root, { to: note.body });
            tryVerify(() => note.visible, 2000, "it shows");
            keys.forceActiveFocus();
            keyClick(Qt.Key_Escape);
            compare(note.ui.calls, ["answer 1 false"]);
            compare(note.shown.id, 2, "the plain notice after it");
            keyClick(Qt.Key_Escape);
            keyClick(Qt.Key_Escape);
            compare(note.ui.calls, ["answer 1 false", "dismiss 2"], "each notice is answered once");
            verify(!note.visible, "nothing is left to answer");
        }

        // Arabic flips text only (ADR-0020): Yes stays left of No, and both carry the Arabic words.
        function test_an_arabic_question_names_its_answers_in_arabic() {
            const note = make([asking(1, Ar)]);
            tryVerify(() => note.visible, 2000, "it shows");
            const [yes, no] = [Ar.texts["button.yes"], Ar.texts["button.no"]];
            compare(buttons(note), [yes, no]);
            waitForRendering(note.contentItem);
            const [y, n] = [button(note, yes), button(note, no)].map(o => o.mapToItem(note.contentItem, 0, 0).x);
            verify(y < n, "yes " + y + ", no " + n);
        }

        function test_the_count_comes_in_the_keyboards_language() {
            const note = make([notice(1, false), notice(2, false), notice(3, false)], { more: Ar.more });
            tryVerify(() => note.visible, 2000, "it shows");
            compare(note.more, Ar.more[2]);
        }
    }
}
