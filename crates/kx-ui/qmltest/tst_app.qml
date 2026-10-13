// The app's windows over the stand-in Ui and Keyboard, as the root wires them: Close puts the keyboard away, a stopped
// keyboard hides it all, and the tray menu closes on a pick, a press on our windows or another front window.
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

        // Close puts the keyboard and its shadow away by its own signal, not Window.close(), until Show brings it back.
        function test_close_puts_the_keyboard_and_its_shadow_away() {
            const app = make();
            const spy = createTemporaryObject(spyC, root, { target: app.face, signalName: "putAway" });
            mouseClick(all(app.face.contentItem, o => o.Accessible.id === "bar.close")[0]);
            compare(spy.count, 1);
            verify(app.away, "away");
            verify(!app.face.visible && !app.kbGlow.visible, "the keyboard and its shadow hide");
            app.ui.showAsked();
            verify(app.face.visible && app.kbGlow.visible, "Show brings both back");
        }

        // The shadow shows before the keyboard and hides after it, so every guard try that finds the keyboard shown
        // finds its shadow shown too: a run never guards the keyboard without its shadow (G9).
        function test_every_guard_try_that_finds_the_keyboard_finds_its_shadow() {
            const app = make();
            const seen = [];
            app.keyboard.guarding.connect(() => seen.push({ kb: app.face.visible, glow: app.kbGlow.visible }));
            const tried = act => {
                const n = seen.length;
                act();
                tryVerify(() => seen.length > n, 2000, "a guard try");
            };
            tried(() => { app.hide(); app.ui.showAsked(); });
            app.keyboard.bubble = { x: 40, y: 50 };
            tried(() => app.minimise());
            tried(() => app.open());
            tried(() => { run(app, false); run(app, true); });
            verify(seen.filter(s => s.kb).length >= 3, "tries with the keyboard shown");
            verify(seen.every(s => !s.kb || s.glow), "its shadow showed at each");
        }

        // The keyboard module stops (no board) or runs again, as the hub tells it with new settings.
        function run(app, on) {
            app.keyboard.board = on ? Face.board : null;
            app.ui.settings = Object.assign({}, app.ui.settings);
            compare(app.face.running, on);
        }

        // A keyboard module that stops hides the keyboard, its shadow and the bubble; when it runs again, it shows.
        function test_a_stopped_keyboard_hides_everything_until_it_runs_again() {
            const app = make();
            run(app, false);
            verify(!app.face.visible && !app.kbGlow.visible, "the keyboard and its shadow hide");
            run(app, true);
            verify(app.face.visible && app.kbGlow.visible, "both show again");
            app.keyboard.bubble = { x: 40, y: 50 };
            app.minimise();
            verify(app.bubble.visible && app.bubbleGlow.visible, "minimised");
            run(app, false);
            verify(!app.bubble.visible && !app.bubbleGlow.visible, "the bubble and its shadow hide");
        }

        // A minimised keyboard whose module stops and runs again gets its bubble back, in the corner the new settings
        // name, so a one-click user can always reach it.
        function test_a_minimised_keyboard_gets_its_bubble_back_when_it_runs_again() {
            const app = make();
            app.keyboard.bubble = { x: 40, y: 50 };
            app.minimise();
            run(app, false);
            app.keyboard.calls = [];
            app.keyboard.board = Object.assign({}, Face.board, { bubble_corner: "left" });
            app.ui.settings = Object.assign({}, app.ui.settings);
            verify(app.face.running);
            verify(app.bubble.visible && app.bubbleGlow.visible, "the bubble comes back");
            verify(!app.face.visible && !app.kbGlow.visible, "the keyboard stays minimised");
            verify(app.keyboard.calls.indexOf("bubbleAt left") >= 0, "in the new corner");
            waitForRendering(app.bubble.contentItem);
            mouseClick(app.bubble.contentItem);
            verify(app.face.visible && !app.bubble.visible, "a click brings the keyboard");
        }

        // With nowhere for the bubble, the keyboard itself comes back rather than nothing.
        function test_a_minimised_keyboard_with_no_bubble_spot_comes_back_itself() {
            const app = make();
            app.keyboard.bubble = { x: 40, y: 50 };
            app.minimise();
            run(app, false);
            app.keyboard.bubble = null;
            run(app, true);
            verify(app.face.visible && app.kbGlow.visible, "the keyboard shows");
            verify(!app.bubble.visible && !app.away, "and nothing is away");
        }

        // Close stays closed when the module runs again, as before it stopped; the tray's Show brings it back.
        function test_a_closed_keyboard_stays_closed_when_it_runs_again() {
            const app = make();
            app.hide();
            run(app, false);
            run(app, true);
            verify(!app.face.visible && !app.bubble.visible, "still closed");
            app.ui.showAsked();
            verify(app.face.visible, "Show brings it back");
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
