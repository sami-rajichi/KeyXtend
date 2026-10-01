// The keyboard's language over the stand-in Ui and Keyboard, as the root wires the app (ADR-0020): a new layout
// language switches every text on and around the keyboard at once and moves nothing, Arabic keeps the English key
// words and flips text only, and the tray keeps the system's language.
import QtQuick
import QtTest
import "../qml"
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    // The app as the root makes it, with its objects declared in place, over the stand-ins.
    Component {
        id: appC
        App {
            ui: FakeUi {}
            keyboard: FakeKeyboard {}
        }
    }

    TestCase {
        name: "texts"
        when: windowShown

        // The notice the banner shows while the keyboard's language changes.
        readonly property string noticeKey: "notice.keys-not-sent"

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
        // The control under `item` a screen reader hears as `name` in `role`.
        function named(item, role, name) {
            return all(item, o => o.Accessible.role === role && o.Accessible.name === name)[0];
        }
        function button(app, name) {
            return named(app.face.contentItem, Accessible.Button, name);
        }
        // The app over fresh stand-ins, its keyboard shown under an English banner; the real pointer passes through.
        function make() {
            const app = createTemporaryObject(appC, root);
            Pointer.through([app.face]);
            tryVerify(() => app.face.visible, 2000, "the keyboard shows");
            waitForRendering(app.face.contentItem);
            app.ui.notices = [{ id: 1, text: En.texts[noticeKey], retry: false }];
            app.ui.noticesWaiting();
            tryVerify(() => app.notice.visible, 2000, "the banner shows");
            return app;
        }
        // The app in front switched to a layout in language `t` on a system in `sys`: the hub tells the banner again,
        // then the texts change.
        function speak(app, t, sys) {
            app.ui.retold = [{ id: 1, text: t.texts[noticeKey], retry: false }];
            app.ui.trayTexts = sys.texts;
            app.ui.lang = Object.assign({}, t.lang, { tray: sys.lang.tray });
            app.ui.texts = t.texts;
        }
        // The keyboard window's tooltip.
        function tipOf(app) {
            return [...app.face.data].find(o => o.item && o.item.hint).item;
        }
        // The keys that show a word.
        function labelled(app) {
            return all(app.face.contentItem, o => o.kv !== undefined && o.later !== undefined && o.label.length > 0);
        }
        // The top bar's size buttons, in order.
        function sizeButtons(app) {
            return all(app.face.contentItem, o => o.part !== undefined && o.text.length > 0);
        }
        // Where the keyboard window and each item of `items` are.
        function places(app, items) {
            const spot = o => {
                const p = o.mapToItem(null, 0, 0);
                return [p.x, p.y];
            };
            return [[app.face.x, app.face.y, app.face.width, app.face.height]].concat(items.map(spot));
        }
        // Every word on keys, the readers' names, the size letters, the tip, the banner and the window title in `t`.
        function speaks(app, t) {
            const where = t.lang.tag + ": ";
            compare(labelled(app).map(k => k.label), labelled(app).map(k => t.texts[k.kv.label]), where + "the words");
            verify(button(app, t.texts["key.esc"]), where + "readers hear the keys in it");
            compare(sizeButtons(app).map(b => b.text), t.lang.sizes.map(z => z.label), where + "the size letters");
            compare(tipOf(app).name, "", where + "a tip in the last language closes");
            const mic = button(app, t.texts["key.mic"]);
            mouseMove(app.face.contentItem, 1, 1);
            mouseMove(mic, mic.width / 2, mic.height / 2);
            compare(tipOf(app).name, t.texts["key.mic"] + "\n" + t.texts["tip.later"], where + "the tip");
            compare(app.notice.shown.text, t.texts[noticeKey], where + "the banner");
            verify(named(app.notice.contentItem, Accessible.Button, t.texts["button.ok"]), where + "its button");
            compare(app.face.title, t.texts["window.keyboard"], where + "the window's name");
        }

        // The owner's case: an English layout in front of a Windows in another language, then French, then English.
        function test_the_layouts_language_switches_the_keyboards_texts_at_once_and_not_the_trays() {
            const app = make();
            const board = app.face.board;
            const keys = labelled(app).concat([button(app, En.texts["bar.grip"])]);
            const was = places(app, keys);
            const fam = app.face.res.fam;
            verify(keys.some(k => k.kv && En.texts[k.kv.label] !== Fr.texts[k.kv.label]), "the case this test is for");
            verify(fam.arabic !== fam.latin, "the fonts this test tells apart");
            for (const t of [Fr, En]) {
                speak(app, t, Ar);
                speaks(app, t);
                compare(places(app, keys), was, t.lang.tag + ": nothing moves");
                verify(app.face.board === board, t.lang.tag + ": the board is not read again");
                compare(app.face.res.fam.ui, fam.latin, t.lang.tag + ": the keyboard's font");
                app.ui.trayClicked(300, 500);
                const show = Ar.texts["tray.show"];
                verify(named(app.menu.contentItem, Accessible.MenuItem, show), "the tray keeps its language");
                compare(app.menu.title, Ar.texts["window.menu"]);
                const label = all(app.menu.contentItem, o => o.text === show && o.font !== undefined)[0];
                compare(label.font.family, fam.arabic, "the tray keeps its language's font");
                app.menu.shut();
            }
        }

        // Arabic keyboards print the English key words; the tips and the banner read right to left, and nothing moves.
        function test_an_arabic_layout_keeps_english_key_words_and_flips_text_only() {
            const app = make();
            const keys = labelled(app).concat([button(app, En.texts["bar.grip"])]);
            const was = places(app, keys);
            speak(app, Ar, En);
            speaks(app, Ar);
            compare(button(app, Ar.texts["key.esc"]).label, En.texts["label.esc"], "the English word");
            compare(app.face.res.fam.ui, app.face.res.fam.arabic, "Arabic texts in the Arabic font");
            compare(places(app, keys), was, "nothing moves");
            const msg = all(app.notice.contentItem, o => o.text === Ar.texts[noticeKey] && o.wrapMode !== undefined)[0];
            compare(msg.effectiveHorizontalAlignment, Text.AlignRight, "the banner reads right to left");
        }
    }
}
