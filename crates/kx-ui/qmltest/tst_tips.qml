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
            for (const w of [app.face, app.bubble])
                w.flags |= Qt.WindowTransparentForInput;
            app.ui.settings = Object.assign({}, En.settings, { tipMs: waitMs });
            tryVerify(() => app.face.visible, 2000, "the keyboard shows");
            waitForRendering(app.face.contentItem);
            compare(tipOf(app.face).ms, waitMs, "the wait comes from the settings");
            return app;
        }
        // The tip window `win` shows, made directly or by a Loader.
        function tipOf(win) {
            return [...win.data].map(o => o && o.item !== undefined ? o.item : o).find(o => o && o.hint && o.ms !== undefined);
        }
        // The tip on screen now, of either window; null when none shows.
        function shown(app) {
            return [app.face, app.bubble].map(tipOf).find(t => t && t.visible) || null;
        }
        // The face's board at `size`, off preset: the module scales every box by the size (kx-mod-keyboard geom.rs).
        function scaled(size) {
            const box = b => Object.assign({}, b, { x: b.x * size, y: b.y * size, w: b.w * size, h: b.h * size });
            const b = Face.board;
            return Object.assign({}, b, { size: size, preset: null, plate: box(b.plate), bar: box(b.bar), dpad: box(b.dpad), keys: b.keys.map(box) });
        }
        // Every keyboard control with a tip and the tip it shows in texts `t` (spec: hovering any icon button shows its
        // name): icon keys their name, dimmed keys and the D-pad their name and when they come, the top bar and the corner
        // their name.
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
                const says = k.dim ? later(t.texts[k.name], k.tip) : drawn(k) ? t.texts[k.name] : "";
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
            const tip = tipOf(app.face);
            const list = tipped(app, t);
            verify(list.some(c => c.item.part !== undefined), where + "the top bar is in the list");
            for (const c of list) {
                const at = where + c.says.replace("\n", " ") + ": ";
                mouseMove(c.item, c.item.width / 2, c.item.height / 2);
                compare(tip.name, c.says, at + "the text");
                verify(!tip.visible, at + "it waits");
                tryVerify(() => tip.visible, 2000, at + "it shows");
                verify(tip.at === c.item, at + "over its control");
                const a = tip.area;
                const inside = tip.x >= a.x && tip.y >= a.y && tip.x + tip.width <= a.x + a.width && tip.y + tip.height <= a.y + a.height;
                verify(inside, at + "on the screen: " + [tip.x, tip.y, tip.width, tip.height]);
            }
        }
        // The keyboard switched to the layout in texts `t`; a tip in the last language closes.
        function speak(app, t) {
            app.ui.lang = t.lang;
            app.ui.texts = t.texts;
            const tip = tipOf(app.face);
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
            const tip = tipOf(app.face);
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
            bubbleTip(app);
            compare(shown(app).transientParent, app.bubble, "the bubble owns its tip");
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
        }
    }
}
