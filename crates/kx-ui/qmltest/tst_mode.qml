// The keyboard's mode over the stand-in Ui and Keyboard: shown, away or behind its bubble, kept in the hub, so windows
// made again, as after a device loss, leave the keyboard as the user left it.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face

Item {
    id: root
    width: 200
    height: 100

    // What the hub keeps and the keyboard answers when a test makes the app: the mode, and the bubble's spot.
    property var mode: null
    property var spot: null
    // Where the stand-in keyboard keeps the window, in physical pixels.
    readonly property var kept: ({ x: 100, y: 200 })

    // The app as the root makes it, over stand-ins whose keyboard keeps its place.
    Component {
        id: appC
        App {
            ui: FakeUi {
                mode: root.mode
            }
            keyboard: FakeKeyboard {
                state: Object.assign({}, Face.states.en, { at: root.kept })
                bubble: root.spot
            }
        }
    }

    TestCase {
        name: "mode"
        when: windowShown

        readonly property var shown: ({ away: false, minimised: false })
        readonly property var away: ({ away: true, minimised: false })
        readonly property var minimised: ({ away: true, minimised: true })

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The app over a hub that keeps `mode` and a keyboard with a bubble spot, or none, once it handed over its tray
        // picture, which it draws after it restored the mode.
        function make(mode, spot) {
            root.mode = mode;
            root.spot = spot === undefined ? { x: 40, y: 50 } : spot;
            const app = createTemporaryObject(appC, root);
            tryVerify(() => app.ui.calls.indexOf("trayShow") >= 0, 2000, "the tray picture");
            return app;
        }
        function same(a, b, why) {
            compare(JSON.stringify(a), JSON.stringify(b), why);
        }

        function test_each_change_of_mode_reaches_the_hub() {
            const app = make(shown);
            verify(app.face.visible, "a new hub shows the keyboard");
            same(app.ui.mode, shown);
            app.hide();
            same(app.ui.mode, away, "Close");
            app.minimise();
            same(app.ui.mode, minimised, "Minimise");
            app.open();
            same(app.ui.mode, shown, "the bubble or Show");
        }

        function test_a_keyboard_made_again_while_away_stays_away() {
            const app = make(away);
            verify(app.away && !app.minimised);
            verify(!app.face.visible && !app.kbGlow.visible && !app.bubble.visible, "nothing shows");
            same(app.ui.mode, away, "the hub keeps it");
            app.ui.showAsked();
            verify(app.face.visible, "Show brings it back");
            same(app.ui.mode, shown);
        }

        function test_a_keyboard_made_again_while_minimised_gets_its_bubble_from_its_place() {
            const app = make(minimised);
            tryVerify(() => app.bubble.visible && app.bubbleGlow.visible, 2000, "the bubble shows");
            verify(!app.face.visible && app.minimised, "the keyboard stays minimised");
            compare([app.keyboard.placedAt.x, app.keyboard.placedAt.y], [root.kept.x, root.kept.y], "placed where it was");
            const f = app.face;
            const c = f.spots.physical(f.at.x + f.drawn.width / 2, f.at.y + f.drawn.height / 2);
            same(app.keyboard.bubbleFrom, { x: c.x, y: c.y }, "the bubble's corner is found from there");
            same(app.ui.mode, minimised);
            waitForRendering(app.bubble.contentItem);
            mouseClick(app.bubble.contentItem);
            verify(app.face.visible, "a click brings the keyboard");
        }

        // With nowhere for the bubble, the keyboard itself comes back rather than nothing.
        function test_a_keyboard_made_again_minimised_with_no_bubble_spot_shows() {
            const app = make(minimised, null);
            verify(app.face.visible, "the keyboard shows");
            verify(!app.bubble.visible && !app.away);
            same(app.ui.mode, shown);
        }
    }
}
