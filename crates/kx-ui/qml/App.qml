// The app's windows over the Ui and Keyboard objects: the keyboard, its minimise bubble, the notice banner and the tray
// menu, each over a click-through shadow window, and the tray icon's picture. A shadow window owns its window, so
// Windows keeps the window just above it; owners show first.
pragma ComponentBehavior: Bound
import QtQuick

QtObject {
    id: app

    // The hub's Ui and Keyboard objects, or the tests' stand-ins.
    required property QtObject ui
    required property QtObject keyboard
    // The tray menu's clock in ms; tests set their own.
    property var now: () => Date.now()
    // Minimise or Close put the keyboard away; it comes back from the bubble or a second copy's Show.
    property bool away: false
    // Minimise put it away behind the bubble, which comes back when the keyboard module runs again.
    property bool minimised: false

    // The UI ended and the tray icon is gone, so the root quits.
    signal done

    // The windows, for the tests.
    readonly property alias face: face
    readonly property alias bubble: bubble
    readonly property alias notice: notice
    readonly property alias menu: trayMenu

    property Connections hub: Connections {
        target: app.ui
        function onQuitAsked() {
            trayMenu.shut();
            app.ui.trayHide();
            app.done();
        }
        function onShowAsked() {
            app.open();
        }
        function onTrayClicked(x, y) {
            trayMenu.toggle(x, y);
        }
        function onTrayAway() {
            trayMenu.away();
        }
        function onLookChanged() {
            Qt.callLater(app.trayPicture.draw);
        }
        function onSettingsChanged() {
            Qt.callLater(app.trayPicture.draw);
        }
    }

    property Glow kbGlow: Glow {
        lk: face.lk
        box: Qt.rect(face.at.x, face.at.y, face.drawn.width, face.drawn.height)
        radius: face.lk ? face.lk.look.window_radius_px : 0

        KbWindow {
            id: face
            ui: app.ui
            keyboard: app.keyboard
            onMinimise: app.minimise()
            onPutAway: app.hide()
            onRunningChanged: app.follow()
            onTouched: trayMenu.shut()
        }
    }
    // Not owned by the keyboard's shadow, so it can show while the keyboard is hidden.
    property Glow bubbleGlow: Glow {
        transientParent: null
        lk: face.lk
        box: Qt.rect(bubble.x, bubble.y, bubble.width, bubble.height)
        radius: bubble.width / 2

        Bubble {
            id: bubble
            lk: face.lk
            res: face.res
            tipMs: face.cfg.tipMs
            lift: t => app.keyboard.liftAlone(t)
            onRescaled: Qt.callLater(app.rebubble)
            onClicked: {
                trayMenu.shut();
                app.open();
            }
        }
    }

    // Notices show above the keyboard, or at the bottom of the main screen while it is away.
    property Glow noticeGlow: Glow {
        transientParent: null
        lk: face.lk
        box: Qt.rect(notice.x, notice.y, notice.width, notice.height)
        radius: notice.radius
        follow: notice.body

        Notice {
            id: notice
            ui: app.ui
            lk: face.lk
            res: face.res
            s: face.s
            widest: face.running ? face.drawn.width : face.lk.shape.panel.width_px
            spot: (w, h) => face.spots.popSpot(w, h, NaN)
            onShowing: on => app.shadow(app.noticeGlow, on, notice)
            onTouched: trayMenu.shut()
        }
    }
    // The tray icon's menu: Show keyboard and Quit; a press on any of our windows closes it too.
    property Glow menuGlow: Glow {
        transientParent: null
        lk: face.lk
        box: Qt.rect(trayMenu.x, trayMenu.y, trayMenu.width, trayMenu.height)
        radius: trayMenu.sh.radius_px
        follow: trayMenu.body

        TrayMenu {
            id: trayMenu
            lk: face.lk
            res: face.res
            graceMs: face.cfg.trayGraceMs
            now: app.now
            spot: (x, y, w, h) => {
                const p = JSON.parse(app.ui.menuAt(x, y, w, h));
                return p ? face.spots.logical(p.x, p.y) : null;
            }
            onShowing: on => app.shadow(app.menuGlow, on, trayMenu)
            onShowKeyboard: app.open()
            onQuitApp: app.ui.quit()
        }
    }
    // Draws the tray icon's picture in the look and hands it to the Ui object.
    property TrayPicture trayPicture: TrayPicture {
        ui: app.ui
        lk: face.lk
        res: face.res
    }

    // The banner follows the keyboard as it shows, hides, moves or changes size.
    property Connections noticeFollows: Connections {
        target: face
        function onAtChanged() {
            notice.follow();
        }
        function onVisibleChanged() {
            notice.follow();
        }
        function onDrawnChanged() {
            notice.follow();
        }
    }

    // Shows or hides a pop-up's shadow with it. Pop-up window `top` shows again where it was, so its guard run puts it
    // in front; the bubble's tip hides first, as the keyboard's does.
    function shadow(glow, on, top) {
        glow.visible = on;
        if (!on)
            return;
        bubble.hideTip();
        face.guardRun(top);
    }

    // Shows the keyboard over its shadow at its place, once the keyboard module runs.
    function open() {
        away = false;
        minimised = false;
        bubble.visible = false;
        bubbleGlow.visible = false;
        if (!face.running)
            return;
        if (!face.placed)
            face.place();
        kbGlow.visible = true;
        face.visible = true;
    }

    // Hides the keyboard and shows the bubble; false when there is nowhere for the bubble, so the keyboard stays.
    function minimise() {
        face.finish();
        if (!bubbleTo())
            return false;
        hide();
        minimised = true;
        bubbleGlow.visible = true;
        bubble.visible = true;
        face.guardRun();
        return true;
    }

    // Puts the bubble in a bottom corner of the keyboard's screen, where it always is (owner's choice); false when there
    // is nowhere for it.
    function bubbleTo() {
        const at = face.bubbleSpot();
        if (!at)
            return false;
        bubble.x = at.x;
        bubble.y = at.y;
        return true;
    }
    // A new display scale left the shown bubble at its old top-left, so it goes back to its corner.
    function rebubble() {
        if (bubble.visible)
            bubbleTo();
    }

    // Hides the keyboard until the bubble or a second copy's Show brings it back.
    function hide() {
        face.finish();
        away = true;
        minimised = false;
        face.visible = false;
        kbGlow.visible = false;
    }

    // The keyboard module stopped: everything hides. It runs again: the keyboard shows, or its bubble when minimised
    // (the keyboard if the bubble has nowhere to go), and a closed keyboard stays closed.
    function follow() {
        if (!face.running) {
            face.visible = false;
            kbGlow.visible = false;
            bubble.visible = false;
            bubbleGlow.visible = false;
        } else if (!away || (minimised && !minimise())) {
            open();
        }
    }

    // After every handler here has run, so the keyboard has its place; the tray icon is drawn once the look is in.
    Component.onCompleted: {
        Qt.callLater(open);
        Qt.callLater(app.trayPicture.draw);
    }
}
