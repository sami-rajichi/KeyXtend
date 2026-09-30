// The app's root: the Ui and Keyboard objects; the keyboard, its minimise bubble, the notice banner and the tray menu,
// each over a click-through shadow window; and the tray icon's picture. A shadow window owns its window, so Windows
// keeps the window just above it; owners show first.
pragma ComponentBehavior: Bound
import QtQuick
import KeyXtend.Ui

QtObject {
    id: root

    property Ui ui: Ui {
        onQuitAsked: {
            trayMenu.shut();
            root.ui.trayHide();
            Qt.quit();
        }
        onShowAsked: root.open()
        onTrayClicked: (x, y) => trayMenu.toggle(x, y)
        onTrayAway: trayMenu.shut()
        onLookChanged: Qt.callLater(root.trayPicture.draw)
        onSettingsChanged: Qt.callLater(root.trayPicture.draw)
    }
    property Keyboard keyboard: Keyboard {}
    // Minimise or Close put the keyboard away; it comes back from the bubble or a second copy's Show.
    property bool away: false

    property Glow kbGlow: Glow {
        lk: face.lk
        box: Qt.rect(face.at.x, face.at.y, face.drawn.width, face.drawn.height)
        radius: face.lk ? face.lk.look.window_radius_px : 0

        KbWindow {
            id: face
            ui: root.ui
            keyboard: root.keyboard
            onMinimise: root.minimise()
            onClose: root.hide()
            onRunningChanged: root.follow()
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
            onClicked: {
                trayMenu.shut();
                root.open();
            }
            onTip: (name, at) => face.hint(name, at)
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
            ui: root.ui
            lk: face.lk
            res: face.res
            s: face.s
            widest: face.running ? face.drawn.width : face.lk.shape.panel.width_px
            rtl: face.cfg.rtl
            spot: (w, h) => face.popSpot(w, h, NaN)
            onShowing: on => root.shadow(root.noticeGlow, on)
        }
    }
    // The tray icon's menu: Show keyboard and Quit; a press on the keyboard or the bubble closes it too.
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
            rtl: face.cfg.rtl
            spot: (x, y, w, h) => {
                const p = JSON.parse(root.ui.menuAt(x, y, w, h));
                return p ? face.logical(p.x, p.y) : null;
            }
            onShowing: on => root.shadow(root.menuGlow, on)
            onShowKeyboard: root.open()
            onQuitApp: root.ui.quit()
        }
    }
    // Draws the tray icon's picture in the look and hands it to the Ui object.
    property TrayPicture trayPicture: TrayPicture {
        ui: root.ui
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

    // Shows or hides a pop-up's shadow with it; a shown pop-up gets the guard.
    function shadow(glow, on) {
        glow.visible = on;
        if (on)
            face.guardRun();
    }

    // Shows the keyboard over its shadow at its place, once the keyboard module runs.
    function open() {
        away = false;
        bubble.visible = false;
        bubbleGlow.visible = false;
        if (!face.running)
            return;
        if (!face.placed)
            face.place();
        kbGlow.visible = true;
        face.visible = true;
    }

    // Hides the keyboard and shows the bubble in a bottom corner of its screen, where it always is (owner's choice).
    function minimise() {
        face.finish();
        const at = face.bubbleSpot();
        if (!at)
            return;
        bubble.x = at.x;
        bubble.y = at.y;
        hide();
        bubbleGlow.visible = true;
        bubble.visible = true;
        face.guardRun();
    }

    // Hides the keyboard until the bubble or a second copy's Show brings it back.
    function hide() {
        face.finish();
        away = true;
        face.visible = false;
        kbGlow.visible = false;
    }

    // The keyboard module started or stopped: show it, unless put away, or hide it with its bubble.
    function follow() {
        if (face.running && !away) {
            open();
        } else if (!face.running) {
            face.visible = false;
            kbGlow.visible = false;
            bubble.visible = false;
            bubbleGlow.visible = false;
        }
    }

    // After every handler here has run, so the keyboard has its place; the tray icon is drawn once the look is in.
    Component.onCompleted: {
        Qt.callLater(open);
        Qt.callLater(root.trayPicture.draw);
    }
}
