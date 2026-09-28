// The selection pill (mock-up .selpill): one Copy button next to selected text, in the theme's badge colours; it never takes focus.
import QtQuick

Window {
    id: pill

    // The look and fonts, the button's text, and the narrowest the pill may be (Qt units).
    required property var lk
    required property var res
    required property string label
    required property real least

    signal copyClicked

    readonly property var sh: lk.shape.pill

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    color: "transparent"
    visible: false
    width: Math.max(least, text.implicitWidth + 2 * (sh.button_side_px + sh.pad_px))

    // Moves to point `at`, in Qt units.
    function place(at) {
        pill.x = at.x;
        pill.y = at.y;
    }

    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: pill.lk.palette.badge_bg
    }
    Rectangle {
        anchors.fill: parent
        anchors.margins: pill.sh.pad_px
        radius: height / 2
        color: area.containsMouse ? pill.lk.common.pill_hover : "transparent"
        Accessible.role: Accessible.Button
        Accessible.name: pill.label
        Accessible.onPressAction: pill.copyClicked()

        Text {
            id: text
            anchors.centerIn: parent
            text: pill.label
            textFormat: Text.PlainText
            font.family: pill.res.fam.latin
            font.pixelSize: pill.sh.font_px
            font.weight: pill.sh.weight
            color: pill.lk.palette.badge_ink
            // The button already carries the label; hide the text so readers say it once.
            Accessible.ignored: true
        }
        MouseArea {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onClicked: pill.copyClicked()
        }
    }
}
