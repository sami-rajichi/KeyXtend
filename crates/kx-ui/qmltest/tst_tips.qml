// Tooltips over the stand-in Ui and Keyboard, as the owner uses them: every button with a tip shows it after the wait,
// every time, in each language, size and theme and after a quick move; and each window owns its tip, as Windows keeps
// an owned window above its owner when the guard lifts the owner (ADR-0013).
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-dolch-light.js" as Dolch
import "pointer.js" as Pointer
import "board.js" as Board

Item {
    id: root
    width: 200
    height: 100

    Component {
        id: appC
        App {
            ui: FakeUi {}
            keyboard: FakeKeyboard {}
        }
    }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "tips"
        when: windowShown

        // A short wait before a tip shows, read from the settings like the default, so a pass over every button is quick.
        readonly property int waitMs: 100

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
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
        // The one button under `item` a screen reader calls `name`.
        function named(item, name) {
            const hits = all(item, o => o.Accessible.role === Accessible.Button && o.Accessible.name === name);
            compare(hits.length, 1, name);
            return hits[0];
        }
        // The app over fresh stand-ins with the short wait, its keyboard shown. The real pointer passes through its
        // windows, so a hand on the mouse during the run cannot move the hover; the test's own moves still reach them.
        function make() {
            const app = createTemporaryObject(appC, root);
            Pointer.through([app.face, app.bubble]);
            app.ui.settings = Object.assign({}, En.settings, { tipMs: waitMs });
            tryVerify(() => app.face.visible, 2000, "the keyboard shows");
            waitForRendering(app.face.contentItem);
            compare(Board.tipOf(app.face).ms, waitMs, "the wait comes from the settings");
            compare(Board.tipOf(app.bubble).ms, waitMs, "the bubble's wait too");
            return app;
        }
        // True when a `tip`-sized box at `p` lies on screen `sc`.
        function inside(tip, p, sc) {
            const inX = p.x >= sc.virtualX && p.x + tip.width <= sc.virtualX + sc.width;
            return inX && p.y >= sc.virtualY && p.y + tip.height <= sc.virtualY + sc.height;
        }
        // The tip `tip` lies on the screen under its control.
        function onScreen(tip, at) {
            const c = tip.at.mapToGlobal(tip.at.width / 2, 0);
            const holds = sc => c.x >= sc.virtualX && c.x < sc.virtualX + sc.width && c.y >= sc.virtualY && c.y < sc.virtualY + sc.height;
            const sc = Array.from(Qt.application.screens).find(holds);
            verify(sc && inside(tip, tip, sc), at + "on its control's screen: " + [tip.x, tip.y, tip.width, tip.height]);
        }
        // The tip `tip` never takes focus, lets clicks through, and stays a tool-tip window: Qt gives a pop-up no owner.
        function isTip(tip, at) {
            verify(tip.flags & Qt.WindowDoesNotAcceptFocus, at + "no focus");
            verify(tip.flags & Qt.WindowTransparentForInput, at + "click-through");
            compare(tip.flags & Qt.WindowType_Mask, Qt.ToolTip, at + "a tool-tip window");
        }
        // The tip on screen now, of either window; null when none shows.
        function shown(app) {
            return [app.face, app.bubble].map(Board.tipOf).find(t => t && t.visible) || null;
        }
        // The face's board at `size`, off preset: the module scales every box by the size (kx-mod-keyboard geom.rs).
        function scaled(size) {
            const box = b => Object.assign({}, b, { x: b.x * size, y: b.y * size, w: b.w * size, h: b.h * size });
            const b = Face.board;
            return Object.assign({}, b, { size: size, preset: null, plate: box(b.plate), bar: box(b.bar), dpad: box(b.dpad), keys: b.keys.map(box) });
        }
        // Every keyboard control with a tip and the tip it shows in texts `t` (spec: hovering any icon button shows its
        // name): icon keys their name, dimmed keys and the D-pad their name and when they come, a key with a state tip its
        // name and that tip, and the top bar and the corner their name.
        function tipped(app, t) {
            const face = app.face.contentItem;
            const icons = app.face.lk.icons;
            const later = (name, tip) => name + "\n" + t.texts[tip];
            const drawn = k => {
                const [group, id] = k.icon ? k.icon.split(".") : [];
                return !!(group && icons[group] && icons[group][id]);
            };
            const out = [];
            for (const k of app.face.board.keys) {
                const tip = k.dim ? k.tip : app.face.st.keys[k.id].tip;
                const says = tip ? later(t.texts[k.name], tip) : drawn(k) ? t.texts[k.name] : "";
                if (says)
                    out.push({ item: named(face, t.texts[k.name]), says: says });
            }
            for (const b of all(face, o => o.part !== undefined && o.visible)) {
                const says = b.text ? t.lang.sizes.find(z => z.label === b.text).name : t.texts["bar." + b.part];
                out.push({ item: b, says: says });
            }
            out.push({ item: named(face, t.texts["bar.corner"]), says: t.texts["bar.corner"] });
            const pad = Face.board.dpad;
            out.push({ item: named(face, t.texts[pad.name]), says: later(t.texts[pad.name], pad.tip) });
            return out;
        }
        // Moves straight from control to control, as a quick hand does: each tip waits, then shows its text over its
        // control and stays on the keyboard's screen.
        function pass(app, t, where) {
            const tip = Board.tipOf(app.face);
            const list = tipped(app, t);
            verify(list.some(c => c.item.part !== undefined), where + "the top bar is in the list");
            for (const c of list) {
                const at = where + c.says.replace("\n", " ") + ": ";
                mouseMove(c.item, c.item.width / 2, c.item.height / 2);
                compare(tip.name, c.says, at + "the text");
                verify(!tip.visible, at + "it waits");
                tryVerify(() => tip.visible, 2000, at + "it shows");
                verify(tip.at === c.item, at + "over its control");
                onScreen(tip, at);
            }
        }
        // The keyboard switched to the layout in texts `t`; a tip in the last language closes.
        function speak(app, t) {
            app.ui.lang = t.lang;
            app.ui.texts = t.texts;
            const tip = Board.tipOf(app.face);
            verify(!tip.visible && tip.name === "", t.lang.tag + ": the last tip closes");
        }

        // The owner's session: English, French, the smallest size, another theme, Arabic; every tip shows each time.
        function test_every_tip_shows_after_the_wait_in_each_language_size_and_theme() {
            const app = make();
            pass(app, En, "en 1: ");
            speak(app, Fr);
            pass(app, Fr, "fr 1: ");
            // A hand rests on a button before it clicks; the keyboard then shrinks under the pointer.
            app.keyboard.board = scaled(Face.sizeMin);
            app.keyboard.reply = { board: true, state: false };
            const smaller = named(app.face.contentItem, Fr.texts["bar.smaller"]);
            mouseMove(smaller, smaller.width / 2, smaller.height / 2);
            mouseClick(smaller);
            compare(app.face.s, Face.sizeMin, "the smallest size");
            pass(app, Fr, "fr small: ");
            app.ui.look = Dolch.lk;
            speak(app, Ar);
            pass(app, Ar, "ar small dolch: ");
            // A key that prints its word has no tip, and the next icon key still shows its own.
            const tip = Board.tipOf(app.face);
            const esc = named(app.face.contentItem, Ar.texts["key.esc"]);
            mouseMove(esc, esc.width / 2, esc.height / 2);
            verify(!tip.visible && tip.at === null, "no tip on a key that prints its word");
            const bksp = named(app.face.contentItem, Ar.texts["key.bksp"]);
            mouseMove(bksp, bksp.width / 2, bksp.height / 2);
            tryVerify(() => tip.visible && tip.at === bksp, 2000, "Backspace's tip shows again");
        }

        // The guard lifts each window above all others; a tip that its window owns stays above it, one that it does not
        // own stays behind, as the hidden tip keeps its old place.
        function test_each_window_owns_its_tip() {
            const app = make();
            const face = app.face.contentItem;
            const bksp = named(face, En.texts["key.bksp"]);
            mouseMove(bksp, bksp.width / 2, bksp.height / 2);
            tryVerify(() => shown(app) !== null, 2000, "the key's tip shows");
            compare(shown(app).transientParent, app.face, "the keyboard owns its tip");
            isTip(shown(app), "the keyboard's tip: ");
            bubbleTip(app);
            compare(shown(app).transientParent, app.bubble, "the bubble owns its tip");
            isTip(shown(app), "the bubble's tip: ");
        }

        // The guard may lift a banner or the list after the keyboard, so each tip lifts itself alone in front of our
        // windows, by its title, once it shows (ADR-0013); the bubble's tip too.
        function test_a_tip_lifts_itself_in_front_once_it_shows() {
            const app = make();
            const lifts = () => app.keyboard.calls.filter(c => c === "liftAlone " + En.texts["window.tip"]).length;
            // Only a shown window is found by its title, so each lift must find its tip shown.
            const seen = [];
            app.keyboard.lifted.connect(() => seen.push(shown(app) !== null));
            const bksp = named(app.face.contentItem, En.texts["key.bksp"]);
            mouseMove(bksp, bksp.width / 2, bksp.height / 2);
            compare(lifts(), 0, "not while it waits");
            tryVerify(() => shown(app) !== null, 2000, "the key's tip shows");
            compare(lifts(), 1, "the keyboard's tip, once shown");
            bubbleTip(app);
            compare(lifts(), 2, "the bubble's tip too");
            compare(seen, [true, true], "each tip shows before it lifts");
        }

        // A guard run would lift a shown tip, and its window with it, over a pop-up; so a tip hides as a run starts, the
        // keyboard's for the banner and the bubble's for the tray menu.
        function test_a_shown_tip_hides_when_a_pop_up_shows() {
            const app = make();
            const bksp = named(app.face.contentItem, En.texts["key.bksp"]);
            mouseMove(bksp, bksp.width / 2, bksp.height / 2);
            tryVerify(() => shown(app) !== null, 2000, "the key's tip shows");
            app.ui.notices = [{ id: 1, text: "Notice 1", retry: false }];
            app.ui.noticesWaiting();
            tryVerify(() => app.notice.visible, 2000, "the banner shows");
            verify(shown(app) === null, "the keyboard's tip hides");
            bubbleTip(app);
            app.ui.trayClicked(300, 500);
            verify(app.menu.visible, "the menu opens");
            verify(shown(app) === null, "the bubble's tip hides");
        }

        // A quick hand crosses several keys without resting: only the last one's tip shows, once, after the wait.
        function test_a_quick_sweep_shows_one_tip_over_the_last_key() {
            const app = make();
            const tip = Board.tipOf(app.face);
            const keys = tipped(app, En).filter(c => c.item.part === undefined).slice(0, 4);
            const seen = createTemporaryObject(spyC, root, { target: tip, signalName: "visibleChanged" });
            for (const c of keys) {
                mouseMove(c.item, c.item.width / 2, c.item.height / 2);
                verify(!tip.visible, c.says + ": no tip while the hand moves");
            }
            const last = keys[keys.length - 1];
            tryVerify(() => tip.visible, 2000, "the last key's tip shows");
            compare(seen.count, 1, "it showed once");
            verify(tip.at === last.item, "over the last key");
            compare(tip.name, last.says, "with its text");
        }

        // A keyboard across two stand-in screens, the second a quarter lower: each tip stays on the screen under its
        // button, and a button on no screen keeps it on its window's screen.
        function test_a_tip_stays_on_the_screen_under_its_button() {
            const app = make();
            const tip = Board.tipOf(app.face);
            const a = tip.area;
            const left = { virtualX: a.x, virtualY: a.y, width: a.width, height: a.height };
            const right = { virtualX: a.x + a.width, virtualY: a.y + a.height / 4, width: a.width, height: a.height };
            const spot = (x, y) => tip.spot(Qt.point(x, y), Qt.point(x, y + a.height / 20), [left, right]);
            verify(inside(tip, spot(right.virtualX + 1, right.virtualY + 1), right), "at the right screen's top-left corner");
            verify(inside(tip, spot(right.virtualX - 1, a.y + a.height / 2), left), "at the left screen's right edge");
            verify(inside(tip, spot(a.x - 1, a.y + a.height / 2), left), "on no screen: its window's");
        }

        // A tip in the last language closes, the bubble's too, as the keyboard's language changes (ADR-0020).
        function test_a_new_language_closes_the_bubbles_tip() {
            const app = make();
            bubbleTip(app);
            app.ui.lang = Fr.lang;
            app.ui.texts = Fr.texts;
            verify(shown(app) === null, "no tip in the last language");
        }
        // Minimises the keyboard with its button and rests on the bubble until its tip shows.
        function bubbleTip(app) {
            app.keyboard.bubble = { x: 40, y: 50 };
            const away = named(app.face.contentItem, En.texts["bar.minimise"]);
            mouseMove(away, away.width / 2, away.height / 2);
            mouseClick(away);
            tryVerify(() => app.bubble.visible && !shown(app), 2000, "minimised, and the tip closed");
            waitForRendering(app.bubble.contentItem);
            const disc = app.bubble.contentItem;
            mouseMove(disc, disc.width / 2, disc.height / 2);
            tryVerify(() => shown(app) !== null, 2000, "the bubble's tip shows");
            compare(shown(app).name, En.texts["bar.bubble"]);
            onScreen(shown(app), "the bubble's tip: ");
        }
    }
}
