// The selection pill: one Copy button next to selected text; it never takes focus.
import QtQuick

Window {
    id: pill

    required property string label
    required property real fontPx

    signal copyClicked

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    visible: false

    // Moves to point `at`, in Qt units.
    function place(at) {
        pill.x = at.x;
        pill.y = at.y;
    }

    Cap {
        anchors.fill: parent
        label: pill.label
        fontPx: pill.fontPx
        onClicked: pill.copyClicked()
    }
}
