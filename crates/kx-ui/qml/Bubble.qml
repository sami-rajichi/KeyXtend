// The minimised keyboard: a round button that brings it back (mock-up .kb-bubble); it never takes focus.
import QtQuick

Window {
    id: bub

    // The look, fonts, icons and texts, how long the pointer rests before its tip shows, and its screen in Qt units.
    required property var lk
    required property var res
    required property int tipMs
    required property rect screenBox

    signal clicked

    readonly property var px: lk.shape.extra.bubble_px
    readonly property string name: res.bar("bubble")

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: res.tr("window.bubble")
    color: "transparent"
    visible: false
    width: px[0]
    height: px[0]

    Rectangle {
        id: disc
        anchors.fill: parent
        radius: width / 2
        color: bub.lk.palette.plate
        Accessible.role: Accessible.Button
        Accessible.name: bub.name
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
        tint: bub.lk.palette.legend
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
        area: bub.screenBox
    }
}
