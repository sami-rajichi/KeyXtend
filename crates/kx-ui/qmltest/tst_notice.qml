// The notice banner over the stand-in Ui: it queues and counts what waits, offers Try again only when the app can
// restart what failed, answers each notice once, tells its notices again in a new keyboard language, and never takes
// focus.
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
        fam: { latin: "Segoe UI", arabic: "Segoe UI", ui: "Segoe UI" },
        icons: Qt.resolvedUrl(Assets.root + "/" + Assets.assets.icons),
        ext: Assets.assets.ext,
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k
    })
    // Where the stand-in spot puts the banner.
    readonly property point at: Qt.point(20, 30)

    Component { id: uiC; FakeUi {} }
    Component { id: noteC; Notice {} }
    Component { id: spyC; SignalSpy {} }

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

        function test_the_count_comes_in_the_ui_language() {
            const note = make([notice(1, false), notice(2, false), notice(3, false)], { more: Ar.more });
            tryVerify(() => note.visible, 2000, "it shows");
            compare(note.more, Ar.more[2]);
        }
    }
}
