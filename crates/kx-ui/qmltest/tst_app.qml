// The app's windows over the stand-in Ui and Keyboard, as the root wires them: the tray menu closes on a pick, a second
// click on the icon, a press on any of our windows, and a new front window after the grace; Quit removes the icon.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr

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
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "app"
        when: windowShown

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
        function named(item, role, name) {
            return all(item, o => o.Accessible.role === role && o.Accessible.name === name)[0];
        }
        // The app over fresh stand-ins, once its keyboard shows.
        function make() {
            const app = createTemporaryObject(appC, root);
            tryVerify(() => app.face.visible, 2000, "the keyboard shows");
            waitForRendering(app.face.contentItem);
            return app;
        }
        // A click on the tray icon, which opens the menu.
        function open(app) {
            app.ui.trayClicked(300, 500);
            verify(app.menu.visible, "the menu opens");
            waitForRendering(app.menu.contentItem);
        }
        function item(app, key) {
            return named(app.menu.contentItem, Accessible.MenuItem, En.texts[key]);
        }
        // Opens the List design's language list over the keyboard; answers the list once it shows.
        function openList(app) {
            app.keyboard.board = Object.assign({}, Face.board, { lang_key: "list" });
            app.keyboard.state = Face.states.list;
            app.face.redraw(JSON.stringify({ board: true, state: true }));
            const panel = () => [...app.face.data].find(o => o.item && o.item.win === app.face);
            tryVerify(() => panel() && panel().item.visible, 2000, "the list shows");
            const list = [...panel().item.data].find(o => o.view !== undefined);
            tryVerify(() => list.visible, 2000);
            return list;
        }

        function test_a_pick_closes_the_menu_then_does_it() {
            const app = make();
            app.hide();
            open(app);
            mouseClick(item(app, "tray.show"));
            verify(!app.menu.visible, "closed");
            verify(app.face.visible, "the keyboard shows again");
            open(app);
            mouseClick(item(app, "tray.quit"));
            verify(!app.menu.visible);
            compare(app.ui.calls.filter(c => c === "quit"), ["quit"], "Quit asks the app");
        }

        function test_a_second_click_on_the_icon_closes_it() {
            const app = make();
            open(app);
            app.ui.trayClicked(300, 500);
            verify(!app.menu.visible);
        }

        function test_a_press_on_the_keyboard_or_its_bubble_closes_it() {
            const app = make();
            app.keyboard.bubble = { x: 40, y: 50 };
            open(app);
            mouseClick(named(app.face.contentItem, Accessible.Button, "q"));
            verify(!app.menu.visible, "the keyboard");
            app.minimise();
            verify(app.bubble.visible, "minimised");
            open(app);
            waitForRendering(app.bubble.contentItem);
            mouseClick(app.bubble.contentItem);
            verify(!app.menu.visible, "the bubble");
            verify(app.face.visible, "which brings the keyboard back");
        }

        function test_a_press_on_the_banner_closes_it() {
            const app = make();
            app.ui.notices = [{ id: 1, text: "Notice 1", retry: false }];
            app.ui.noticesWaiting();
            tryVerify(() => app.notice.visible, 2000, "the banner shows");
            open(app);
            waitForRendering(app.notice.contentItem);
            mouseClick(named(app.notice.contentItem, Accessible.Button, En.texts["button.ok"]));
            verify(!app.menu.visible);
            compare(app.ui.calls.filter(c => c.startsWith("dismiss")), ["dismiss 1"]);
        }

        function test_a_press_on_the_language_list_closes_it() {
            const app = make();
            const list = openList(app);
            open(app);
            waitForRendering(list.contentItem);
            mouseClick(all(list.contentItem, o => o.Accessible.role === Accessible.ListItem && !o.Accessible.ignored)[0]);
            verify(!app.menu.visible);
        }

        // A hidden pop-up shows again at its old place, maybe under the keyboard, so its guard run names it to land in
        // front (kx-platform `guard_own`); the keyboard's own run names none.
        function test_a_shown_pop_up_lands_in_front() {
            const app = make();
            const ran = top => app.keyboard.calls.indexOf(top ? "guard " + top : "guard") >= 0;
            tryVerify(() => ran(""), 2000, "the keyboard's run");
            app.ui.notices = [{ id: 1, text: "Notice 1", retry: false }];
            app.ui.noticesWaiting();
            tryVerify(() => ran(En.texts["window.notice"]), 2000, "the banner's run");
            open(app);
            tryVerify(() => ran(En.texts["window.menu"]), 2000, "the menu's run");
        }

        // The banner's guard run, its first try answering "try again", once the keyboard's own run has ended.
        function bannerRun(app) {
            tryVerify(() => app.keyboard.calls.indexOf("guard") >= 0, 2000, "the keyboard's run");
            app.keyboard.again = 1;
            app.keyboard.calls = [];
            app.ui.notices = [{ id: 1, text: "Notice 1", retry: false }];
            app.ui.noticesWaiting();
            tryVerify(() => app.notice.visible, 2000, "the banner shows");
        }
        function tries(app) {
            return app.keyboard.calls.filter(c => c.startsWith("guard"));
        }

        // A run started before the banner's ends keeps naming the banner until the run ends; the next run names none.
        function test_the_pop_up_named_in_a_run_stays_named_until_it_ends() {
            const app = make();
            bannerRun(app);
            app.face.guardRun();
            tryVerify(() => tries(app).length === 2, 2000, "two tries");
            const banner = "guard " + En.texts["window.notice"];
            compare(tries(app), [banner, banner], "the banner stays named");
            app.face.guardRun();
            tryVerify(() => tries(app).length === 3, 2000);
            compare(tries(app)[2], "guard", "the next run names none");
        }

        // Each try reads the pop-up's title, so a new keyboard language within the guard's delay still finds it.
        function test_a_new_language_during_a_run_still_names_the_pop_up() {
            const app = make();
            bannerRun(app);
            app.ui.lang = Fr.lang;
            app.ui.texts = Fr.texts;
            compare(app.notice.title, Fr.texts["window.notice"], "the banner's title follows");
            tryVerify(() => tries(app).indexOf("guard " + Fr.texts["window.notice"]) >= 0, 2000, "a try names it in French");
        }

        function test_a_new_front_window_closes_it_after_the_grace() {
            let t = 1000;
            const app = make();
            app.now = () => t;
            open(app);
            t += En.settings.trayGraceMs - 1;
            app.ui.trayAway();
            verify(app.menu.visible, "the hidden icons' pop-up closing");
            t += 1;
            app.ui.trayAway();
            verify(!app.menu.visible, "another app");
        }

        function test_quit_closes_the_menu_and_removes_the_icon_before_the_end() {
            const app = make();
            const done = createTemporaryObject(spyC, root, { target: app, signalName: "done" });
            open(app);
            app.ui.quitAsked();
            verify(!app.menu.visible);
            compare(app.ui.calls.filter(c => c === "trayHide"), ["trayHide"]);
            compare(done.count, 1, "then the root quits");
        }
    }
}
