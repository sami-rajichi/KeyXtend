// Covers one screen while the keyboard moves or resizes, so the second click never reaches another app.
import QtQuick

Window {
    id: c

    // The screen it covers, from `Qt.application.screens`, its UI Automation id, and what a screen reader hears.
    required property var area
    required property string uid
    required property string name

    // The second click.
    signal picked

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: c.name
    color: "transparent"
    visible: false
    screen: area
    // One pixel short at the top only: a window covering a whole screen looks full-screen, and Windows drops the taskbar.
    x: area.virtualX
    y: area.virtualY + 1
    width: area.width
    height: area.height - 1

    // It acts at let-go, wherever the pointer went (ADR-0024); a double click's second click ends nothing more.
    Act {
        anchors.fill: parent
        doubles: false
        Accessible.role: Accessible.Button
        Accessible.name: c.name
        Accessible.id: c.uid
        Accessible.onPressAction: c.picked()
        onActed: c.picked()
    }
}
