// The caption bar: what the mic hears, then the words it typed; it never takes focus and lets clicks pass.
import QtQuick

Window {
    id: bar

    required property real fontPx
    required property real pad
    property string text: ""
    // Edge width; drawn in the text colour, like the keys.
    readonly property int edge: 1

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
        | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    visible: false
    color: pal.button

    SystemPalette { id: pal }

    // Shows caption `r` with its top-left at `at` (Qt units); hides it after `r.hide_ms` when set.
    function say(r, at) {
        bar.text = r.text;
        bar.x = at.x;
        bar.y = at.y;
        bar.visible = true;
        if (r.hide_ms) {
            hide.interval = r.hide_ms;
            hide.restart();
        } else {
            hide.stop();
        }
    }

    Timer {
        id: hide
        onTriggered: bar.visible = false
    }
    Rectangle {
        anchors.fill: parent
        color: "transparent"
        border.width: bar.edge
        border.color: pal.windowText
    }
    Text {
        anchors.fill: parent
        anchors.margins: bar.pad
        text: bar.text
        font.pixelSize: bar.fontPx
        color: pal.buttonText
        wrapMode: Text.Wrap
        elide: Text.ElideRight
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
}
