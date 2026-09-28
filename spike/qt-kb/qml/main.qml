// Qt face of the P1 spike, stage 2: the keyboard and its minimise bubble, each over a click-through shadow window.
// A shadow window owns its window, so Windows keeps the window just above it; owners are shown first.
pragma ComponentBehavior: Bound
import QtQuick
import KeyXtend.Spike

Glow {
    id: kbGlow

    lk: face.lk
    box: Qt.rect(face.at.x, face.at.y, face.drawn.width, face.drawn.height)
    radius: face.lk.look.window_radius_px

    // Shows the keyboard over its shadow, then cuts it now that Windows shows it.
    function open() {
        kbGlow.visible = true;
        face.visible = true;
        face.reshape();
    }

    // Hides the keyboard and shows the bubble in a bottom corner of its screen, where it always is (owner's choice).
    function minimise() {
        face.finish();
        const at = face.bubbleSpot();
        if (!at)
            return;
        bubble.x = at.x;
        bubble.y = at.y;
        face.visible = false;
        kbGlow.visible = false;
        bubbleGlow.visible = true;
        bubble.visible = true;
        face.guardTop(bubble.title);
    }

    // Hides the bubble and brings the keyboard back.
    function restore() {
        bubble.visible = false;
        bubbleGlow.visible = false;
        open();
        face.guardTop(face.title);
    }

    // After every handler here has run, so the keyboard has its place.
    Component.onCompleted: Qt.callLater(open)

    KbWindow {
        id: face
        onMinimise: kbGlow.minimise()
    }
    // Not owned by the keyboard's shadow, so it can show while the keyboard is hidden.
    Glow {
        id: bubbleGlow
        transientParent: null
        lk: face.lk
        box: Qt.rect(bubble.x, bubble.y, bubble.width, bubble.height)
        radius: bubble.width / 2

        Bubble {
            id: bubble
            title: face.bar.bar.bubble_title
            lk: face.lk
            res: face.res
            button: face.bar.bar.bubble
            onClicked: kbGlow.restore()
            onTip: (name, at) => face.hint(name, at)
        }
    }
}
