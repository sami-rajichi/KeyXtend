// The snip overlay: the frozen screen over everything; click a corner, move, click the other corner.
import QtQuick

Window {
    id: over

    // True once the first corner is set; the box then follows the pointer.
    property bool picking: false
    property real ax: 0
    property real ay: 0
    // Width of the region's two-tone edge, and what a screen reader is told.
    required property real edge
    required property string hint

    signal picked

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    visible: false

    // Covers the physical box `s` from its corner `at` (Qt units and scale) and shows the frozen screen from `s.url`.
    function begin(at, s) {
        over.x = at.x;
        over.y = at.y;
        over.width = s.w / at.dpr;
        over.height = s.h / at.dpr;
        picking = false;
        frozen.source = "";
        frozen.source = s.url;
        visible = true;
    }

    // Hides the overlay and drops the frozen screen, which is private.
    function end() {
        visible = false;
        frozen.source = "";
    }

    Image {
        id: frozen
        anchors.fill: parent
        cache: false
        asynchronous: false
        smooth: false
        fillMode: Image.Stretch
        Accessible.ignored: true
    }
    SystemPalette { id: pal }
    Rectangle {
        visible: over.picking
        x: Math.min(over.ax, area.mouseX)
        y: Math.min(over.ay, area.mouseY)
        width: Math.abs(area.mouseX - over.ax)
        height: Math.abs(area.mouseY - over.ay)
        // Two tones, so the edge shows on any screenshot.
        color: "transparent"
        border.width: over.edge
        border.color: pal.text
        Rectangle {
            anchors.fill: parent
            anchors.margins: over.edge
            color: "transparent"
            border.width: over.edge
            border.color: pal.base
        }
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        Accessible.role: Accessible.Button
        Accessible.name: over.hint
        onClicked: mouse => {
            if (!over.picking) {
                over.ax = mouse.x;
                over.ay = mouse.y;
            }
            over.picking = !over.picking;
            over.picked();
        }
    }
}
