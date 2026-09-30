// The app's root: the Ui and Keyboard objects, and the keyboard and its minimise bubble, each over a click-through
// shadow window. A shadow window owns its window, so Windows keeps the window just above it; owners show first.
pragma ComponentBehavior: Bound
import QtQuick
import KeyXtend.Ui

QtObject {
    id: root

    property Ui ui: Ui {
        onQuitAsked: Qt.quit()
        onShowAsked: root.open()
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
            onClicked: root.open()
            onTip: (name, at) => face.hint(name, at)
        }
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

    // After every handler here has run, so the keyboard has its place.
    Component.onCompleted: Qt.callLater(open)
}
