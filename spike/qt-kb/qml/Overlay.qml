// The snip overlay (mock-up .snip-*): the frozen screen, dimmed outside the region; click a corner, move, click the other corner.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: over

    // The look and fonts; the region's edge width, what a screen reader is told, and the size tag with {w} and {h}.
    required property var lk
    required property var res
    required property real edge
    required property string hint
    required property string sizeText

    // True once the first corner is set; the region then follows the pointer.
    property bool picking: false
    property real ax: 0
    property real ay: 0
    // Physical pixels per Qt unit on this screen, for the size tag.
    property real dpr: 1

    readonly property var sh: lk.shape.snip
    readonly property var c: lk.common
    // The region in Qt units; empty before the first click, so the whole screen is dimmed.
    readonly property rect region: picking
        ? Qt.rect(Math.min(ax, area.mouseX), Math.min(ay, area.mouseY), Math.abs(area.mouseX - ax), Math.abs(area.mouseY - ay))
        : Qt.rect(0, 0, 0, 0)

    signal picked

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    visible: false

    // Covers the physical box `s` from its corner `at` (Qt units and scale) and shows the frozen screen from `s.url`.
    function begin(at, s) {
        over.x = at.x;
        over.y = at.y;
        over.width = s.w / at.dpr;
        over.height = s.h / at.dpr;
        dpr = at.dpr;
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

    // The size tag's text for the region now.
    function size() {
        const w = Math.round(region.width * dpr), h = Math.round(region.height * dpr);
        return sizeText.replace("{w}", w).replace("{h}", h);
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
    // Dimming above, below, left and right of the region.
    Rectangle {
        width: over.width
        height: over.region.y
        color: over.c.snip_dim
    }
    Rectangle {
        y: over.region.y + over.region.height
        width: over.width
        height: over.height - y
        color: over.c.snip_dim
    }
    Rectangle {
        y: over.region.y
        width: over.region.x
        height: over.region.height
        color: over.c.snip_dim
    }
    Rectangle {
        x: over.region.x + over.region.width
        y: over.region.y
        width: over.width - x
        height: over.region.height
        color: over.c.snip_dim
    }
    Rectangle {
        visible: over.picking
        x: over.region.x
        y: over.region.y
        width: over.region.width
        height: over.region.height
        color: "transparent"
        border.width: over.edge
        border.color: over.c.snip_edge
    }
    Rectangle {
        visible: over.picking
        x: over.region.x
        y: Math.max(0, over.region.y - over.sh.tag_gap_px)
        width: tag.implicitWidth + 2 * over.sh.tag_pad_px[1]
        height: tag.implicitHeight + 2 * over.sh.tag_pad_px[0]
        radius: over.sh.tag_radius_px
        color: over.c.snip_tag

        Text {
            id: tag
            anchors.centerIn: parent
            text: over.size()
            textFormat: Text.PlainText
            font.family: over.res.fam.latin
            font.pixelSize: over.sh.tag_px
            font.weight: over.sh.tag_weight
            font.features: { "tnum": 1 }
            color: over.c.snip_edge
            Accessible.ignored: true
        }
    }
    // The hint bar at the top centre (mock-up .snip-bar), with the theme's panel look.
    Item {
        x: (over.width - width) / 2
        y: over.sh.bar_top_px
        width: hintText.implicitWidth + 2 * over.sh.bar_pad_px[1]
        height: hintText.implicitHeight + 2 * over.sh.bar_pad_px[0]

        Repeater {
            model: over.lk.palette.window_shadow.filter(l => !l.inset)
            delegate: Shade {
                required property var modelData
                anchors.fill: parent
                shadow: modelData
                radius: over.sh.bar_radius_px
            }
        }
        // High contrast has no shadow, so the bar gets an edge instead.
        Rectangle {
            anchors.fill: parent
            radius: over.sh.bar_radius_px
            color: over.lk.palette.pop_bg
            border.width: over.lk.contrast ? over.lk.shape.line.px : 0
            border.color: over.lk.palette.pop_line
        }
        Text {
            id: hintText
            anchors.centerIn: parent
            text: over.hint
            textFormat: Text.PlainText
            font.family: over.res.fam.latin
            font.pixelSize: over.sh.hint_px
            color: over.lk.palette.pop_muted
            // The overlay's button already says the hint.
            Accessible.ignored: true
        }
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.CrossCursor
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
