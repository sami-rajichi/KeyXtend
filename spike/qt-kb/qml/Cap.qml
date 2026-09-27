// One clickable cap: a key, a tool button or the pill's button. Readers see a button with its label.
import QtQuick

Rectangle {
    id: cap

    required property string label
    required property real fontPx
    // Edge width; drawn in the text colour, so edges keep at least 3:1 contrast.
    readonly property int edge: 1

    signal clicked

    color: area.pressed ? pal.highlight : pal.base
    border.width: edge
    border.color: pal.windowText
    Accessible.role: Accessible.Button
    Accessible.name: label
    Accessible.onPressAction: cap.clicked()

    SystemPalette { id: pal }

    Text {
        width: parent.width
        anchors.verticalCenter: parent.verticalCenter
        horizontalAlignment: Text.AlignHCenter
        elide: Text.ElideRight
        text: cap.label
        font.pixelSize: cap.fontPx
        color: area.pressed ? pal.highlightedText : pal.text
        // The cap already carries the label; hide the text so readers say it once.
        Accessible.ignored: true
    }
    MouseArea {
        id: area
        anchors.fill: parent
        onClicked: cap.clicked()
    }
}
