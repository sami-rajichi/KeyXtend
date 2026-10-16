// The D-pad's Stop: it pops in while an arrow scrolls and out after (mock-up .dp-stop), a spring curve overshooting a
// little, with one dot per speed under its square, the first lit for slow and both for fast (spec §5.2).
pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: stop

    // The pad it stops, whose look, size, way and speed it shows.
    required property Item pad

    readonly property var sh: pad.sh
    readonly property real icon: sh.stop_icon_px * pad.s
    readonly property real dot: sh.dot_px * pad.s
    readonly property real gap: sh.dot_gap_px * pad.s
    // The square and the dots under it, centred together.
    readonly property real head: (height - icon - gap - dot) / 2
    readonly property real row: pad.speeds.length * dot + (pad.speeds.length - 1) * gap

    scale: pad.dir !== "" ? 1 : 0
    visible: scale > 0
    anchors.fill: parent
    anchors.margins: pad.width * sh.stop
    radius: width / 2
    color: pad.d.stop
    Accessible.role: Accessible.Button
    Accessible.name: pad.res.dpad(pad.stopId)
    Accessible.id: pad.res.uid("dpad", pad.stopId)
    Accessible.description: pad.speed
    Accessible.onPressAction: pad.tapped("")
    Behavior on scale {
        Tween {
            move: stop.pad.mv.stop
        }
    }

    Icon {
        id: square
        x: (stop.width - width) / 2
        y: stop.head
        icon: stop.pad.lk.icons.dpad[stop.pad.stopId]
        base: stop.pad.res.icons
        ext: stop.pad.res.ext
        px: stop.icon
        tint: stop.pad.d.stop_ink
    }
    Repeater {
        model: stop.pad.speeds
        delegate: Rectangle {
            required property int index
            // Lit up to the speed it scrolls at; an unlit dot keeps its place, faded.
            readonly property bool lit: index <= stop.pad.pace
            x: (stop.width - stop.row) / 2 + index * (stop.dot + stop.gap)
            y: stop.head + stop.icon + stop.gap
            width: stop.dot
            height: stop.dot
            radius: stop.dot / 2
            color: stop.pad.d.stop_ink
            opacity: lit ? 1 : stop.sh.dot_off
        }
    }
    Act {
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: stop.pad.tip(containsMouse ? stop.Accessible.name : "", stop)
        onActed: {
            stop.pad.tip("", null);
            stop.pad.tapped("");
        }
    }
}
