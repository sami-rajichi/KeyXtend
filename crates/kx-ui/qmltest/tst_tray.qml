// The tray icon's menu and picture over the stand-in Ui: the menu offers Show and Quit, opens by the icon, closes on
// a pick or another click, and the picture is drawn in the theme and handed to the Ui.
import QtQuick
import QtTest
import "../qml"
import "fixtures/assets.js" as Assets
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-dark.js" as Dark

Item {
    id: root
    width: 200
    height: 100

    // The tray menu speaks the system's language, through `tray`.
    readonly property var res: ({
        fam: { latin: "Segoe UI", arabic: "Segoe UI", ui: "Segoe UI", tray: "Segoe UI" },
        icons: Qt.resolvedUrl(Assets.root + "/" + Assets.assets.icons),
        ext: Assets.assets.ext,
        tray: k => En.texts[k] !== undefined ? En.texts[k] : k
    })

    Component { id: uiC; FakeUi {} }
    Component { id: menuC; TrayMenu {} }
    Component { id: picC; TrayPicture {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "tray"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A menu whose spot asks a stand-in Ui, as the root does, and that Ui; `res` for other texts, and `more` for
        // other properties, such as a clock.
        function makeMenu(res, more) {
            const ui = createTemporaryObject(uiC, root);
            const spot = (x, y, w, h) => {
                const p = JSON.parse(ui.menuAt(x, y, w, h));
                return Qt.point(p.x, p.y);
            };
            const props = { lk: N.lk, res: res || root.res, spot: spot, graceMs: En.settings.trayGraceMs };
            return [createTemporaryObject(menuC, root, Object.assign(props, more || {})), ui];
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
        function items(menu) {
            return all(menu.contentItem, o => o.Accessible.role === Accessible.MenuItem);
        }

        function test_the_menu_offers_show_and_quit_and_takes_no_focus() {
            const [menu] = makeMenu();
            compare(items(menu).map(o => o.Accessible.name), [En.texts["tray.show"], En.texts["tray.quit"]]);
            compare(items(menu).map(o => o.part), ["show", "quit"], "icons.toml [tray] items");
            compare(menu.title, En.texts["window.menu"]);
            verify(menu.flags & Qt.WindowDoesNotAcceptFocus, "never takes focus");
            const popup = all(menu.contentItem, o => o.Accessible.role === Accessible.PopupMenu)[0];
            compare(popup.Accessible.name, En.texts["window.menu"]);
        }

        // Arabic flips text only (ADR-0020): each icon stays on the left of its text.
        function test_an_arabic_system_keeps_each_icon_left_of_its_text() {
            const ar = Object.assign({}, root.res, { tray: k => Ar.texts[k] !== undefined ? Ar.texts[k] : k });
            const [menu] = makeMenu(ar);
            compare(menu.title, Ar.texts["window.menu"]);
            const item = items(menu)[0];
            compare(item.Accessible.name, Ar.texts["tray.show"]);
            const icon = all(item, o => o.tint !== undefined && o.icon !== undefined)[0];
            const text = all(item, o => o.text === Ar.texts["tray.show"] && o.font !== undefined)[0];
            const [i, t] = [icon, text].map(o => o.mapToItem(item, 0, 0).x);
            verify(i < t, "icon " + i + ", text " + t);
        }

        function test_a_click_on_the_icon_opens_it_above_the_click_and_another_closes_it() {
            const [menu, ui] = makeMenu();
            const spy = createTemporaryObject(spyC, root, { target: menu, signalName: "showing" });
            menu.toggle(300, 500);
            verify(menu.visible);
            compare([menu.x, menu.y], [300, 500 - menu.height]);
            compare(ui.calls, ["menuAt 300 500"]);
            menu.toggle(300, 500);
            verify(!menu.visible, "a second click closes it");
            compare([spy.count, spy.signalArguments[0][0], spy.signalArguments[1][0]], [2, true, false]);
        }

        function test_a_pick_closes_it_then_asks() {
            const [menu] = makeMenu();
            const asked = [];
            menu.showKeyboard.connect(() => asked.push(["show", menu.visible]));
            menu.quitApp.connect(() => asked.push(["quit", menu.visible]));
            for (const at of [0, 1]) {
                menu.toggle(300, 500);
                waitForRendering(menu.contentItem);
                mouseClick(items(menu)[at]);
                verify(!menu.visible, "closed on a pick");
            }
            compare(asked, [["show", false], ["quit", false]], "closed before it asks");
        }

        function test_a_long_translation_widens_the_menu() {
            const ui = createTemporaryObject(uiC, root);
            const long = "Show the keyboard again, please";
            const res = Object.assign({}, root.res, { tray: k => k === "tray.show" ? long : root.res.tray(k) });
            const spot = () => Qt.point(0, 0);
            const menu = createTemporaryObject(menuC, root, { lk: N.lk, res: res, spot: spot, graceMs: 0 });
            const item = items(menu)[0];
            verify(menu.width > N.lk.shape.menu.width_px, "wider than the smallest menu: " + menu.width);
            verify(menu.width >= item.implicitWidth + 2 * N.lk.shape.menu.pad_px, "the item fits");
        }

        function test_shut_closes_it_and_twice_is_harmless() {
            const [menu] = makeMenu();
            menu.toggle(300, 500);
            menu.shut();
            verify(!menu.visible);
            menu.shut();
            verify(!menu.visible, "closing twice is harmless");
        }

        // The hidden icons' pop-up can bring the last app to the front just after the menu opens.
        function test_a_new_front_window_closes_it_only_after_the_grace() {
            let t = 1000;
            const grace = En.settings.trayGraceMs;
            const [menu] = makeMenu(null, { now: () => t });
            menu.toggle(300, 500);
            t += grace - 1;
            menu.away();
            verify(menu.visible, "inside the grace");
            t += 1;
            menu.away();
            verify(!menu.visible, "after it");
            menu.toggle(300, 500);
            t -= 10 * grace;
            menu.away();
            verify(!menu.visible, "a clock set back counts as after");
            menu.graceMs = 0;
            menu.toggle(300, 500);
            menu.away();
            verify(!menu.visible, "0 turns the grace off");
        }

        function test_the_picture_is_the_theme_badge_and_goes_to_the_ui() {
            for (const lk of [N.lk, Dark.lk]) {
                const ui = createTemporaryObject(uiC, root);
                const pic = createTemporaryObject(picC, root, { ui: ui, lk: lk, res: root.res });
                verify(pic.flags & Qt.WindowDoesNotAcceptFocus && pic.flags & Qt.WindowTransparentForInput);
                compare(pic.width, Math.ceil(ui.trayPixels / Screen.devicePixelRatio));
                const badge = all(pic.contentItem, o => o.radius !== undefined && o.color !== undefined)[0];
                compare(badge.color, Qt.color(lk.palette.badge_bg));
                const glyph = all(pic.contentItem, o => o.tint !== undefined && o.icon !== undefined)[0];
                compare([glyph.icon, glyph.tint], [lk.icons.tray.glyph, Qt.color(lk.palette.badge_ink)]);
                pic.draw();
                tryVerify(() => ui.calls.indexOf("trayShow") >= 0, 5000, "handed over");
                tryVerify(() => !pic.visible, 2000, "hidden again");
            }
        }

        function test_a_new_screen_scale_draws_the_picture_again() {
            const ui = createTemporaryObject(uiC, root);
            const pic = createTemporaryObject(picC, root, { ui: ui, lk: N.lk, res: root.res });
            pic.dpr = pic.dpr * 2;
            tryVerify(() => ui.calls.indexOf("trayShow") >= 0, 5000, "drawn again for the new scale");
            tryVerify(() => !pic.visible, 2000, "hidden again");
        }

        function test_a_draw_asked_during_a_grab_follows_it() {
            const ui = createTemporaryObject(uiC, root);
            const pic = createTemporaryObject(picC, root, { ui: ui, lk: N.lk, res: root.res });
            pic.draw();
            pic.draw();
            tryVerify(() => ui.calls.filter(c => c === "trayShow").length === 2, 5000, "both handed over: " + ui.calls);
            tryVerify(() => !pic.visible, 2000, "hidden again");
        }
    }
}
