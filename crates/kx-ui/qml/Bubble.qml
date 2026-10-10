// The minimised keyboard: a round button that brings it back (mock-up .kb-bubble); it never takes focus.
import QtQuick

Window {
    id: bub

    // The look, fonts, icons and texts, how long the pointer rests before its tip shows, and how its tip lifts itself
    // in front of our windows (`Tip.lift`).
    required property var lk
    required property var res
    required property int tipMs
    required property var lift

    signal clicked

    readonly property var px: lk.shape.extra.bubble_px
    readonly property string name: res.bar("bubble")

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: res.tr("window.bubble")
    color: "transparent"
    visible: false
    width: rescale.keep(px[0])
    height: rescale.keep(px[0])

    // Keeps the size above across display scale changes.
    Rescale {
        id: rescale
    }

    // Hides its tip, as a guard run starts: the guard would lift the bubble with it.
    function hideTip() {
        tip.hint("", null);
    }

    Rectangle {
        id: disc
        anchors.fill: parent
        radius: width / 2
        color: bub.lk.palette.plate
        Accessible.role: Accessible.Button
        Accessible.name: bub.name
        // The id the top bar's buttons use; the gates find the bubble by it.
        Accessible.id: bub.res.uid("bar", "bubble")
        Accessible.onPressAction: bub.clicked()
    }
    Hover {
        lk: bub.lk
        radius: width / 2
        visible: area.containsMouse
    }
    Icon {
        anchors.centerIn: parent
        icon: bub.lk.icons.bar.bubble
        base: bub.res.icons
        ext: bub.res.ext
        px: bub.px[1]
        // The disc is the plate, a window, so high contrast draws the icon in WindowText.
        tint: bub.lk.contrast ? bub.lk.palette.legend_2_plate : bub.lk.palette.legend
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: tip.hint(containsMouse ? bub.name : "", disc)
        onClicked: {
            tip.hint("", disc);
            bub.clicked();
        }
    }
    // Its own tip, which it owns, so the tip stays above it while the keyboard is away.
    Tip {
        id: tip
        transientParent: bub
        lk: bub.lk
        res: bub.res
        ms: bub.tipMs
        lift: bub.lift
    }
}
