// Moving and resizing: one click starts, the window follows the pointer, and the next click anywhere ends it.
// Fill the window with it, last, so its own catcher lies over every control while it runs.
pragma ComponentBehavior: Bound
import QtQuick
import QtQml.Models

Item {
    id: mv

    // Nothing, following the pointer to move, or following it to resize.
    enum Mode { Idle, Moving, Resizing }

    // The window's place, the keyboard bridge, Qt units for a physical point, the poll time, and the texts.
    required property point at
    required property var keyboard
    required property var logical
    required property int pollMs
    required property var res

    // What runs now.
    property int mode: Mover.Idle
    // Where the pointer holds the window while moving.
    property point grab: Qt.point(0, 0)
    // True between the two clicks.
    readonly property bool busy: mode !== Mover.Idle
    // The catchers' text key, also their UI Automation id in every language, and what readers call them.
    readonly property string uid: "catch"
    readonly property string name: res.tr(uid)
    // The catchers, one per screen.
    readonly property alias covers: covers

    // The window's new place while moving, in Qt units.
    signal moved(point to)
    // A resize step answered what to redraw, as the bridge's JSON.
    signal redraw(string json)
    // The second click ended a move (true) or a resize.
    signal ended(bool wasMove)

    // The pointer in Qt units, or null when Windows could not say.
    function pointer() {
        const c = JSON.parse(keyboard.cursor());
        return c ? logical(c.x, c.y) : null;
    }

    // The first click on the grip: the window follows the pointer, held where it was grabbed.
    function startMove() {
        const p = pointer();
        if (!p)
            return;
        grab = Qt.point(p.x - at.x, p.y - at.y);
        mode = Mover.Moving;
    }

    // The first click on the corner: the size follows the pointer.
    function startResize() {
        if (!pointer())
            return;
        redraw(keyboard.resizeBegin());
        mode = Mover.Resizing;
    }

    // One poll: moves or resizes to where the pointer is now.
    function follow() {
        const p = pointer();
        if (!p)
            return;
        if (mode === Mover.Moving)
            moved(Qt.point(p.x - grab.x, p.y - grab.y));
        else if (mode === Mover.Resizing)
            redraw(keyboard.resizeAt());
    }

    // The second click: the window stays where it is, at the size it reached.
    function finish() {
        const was = mode;
        if (was === Mover.Idle)
            return;
        mode = Mover.Idle;
        if (was === Mover.Resizing)
            redraw(keyboard.resizeEnd());
        ended(was === Mover.Moving);
    }

    // A click on the keyboard itself ends it too, whatever lies below, at let-go wherever the pointer went (ADR-0024).
    // The second click of a double click on the grip or corner is ignored, so a shaky start never ends at once.
    Act {
        anchors.fill: parent
        visible: mv.busy
        hoverEnabled: true
        doubles: false
        Accessible.role: Accessible.Button
        Accessible.name: mv.name
        Accessible.id: mv.uid
        Accessible.onPressAction: mv.finish()
        onActed: mv.finish()
    }
    // One catcher per screen, since screens may have different scales.
    Instantiator {
        id: covers
        model: Qt.application.screens
        delegate: Catch {
            required property var modelData
            area: modelData
            uid: mv.uid
            name: mv.name
            visible: mv.busy
            onPicked: mv.finish()
        }
    }
    Timer {
        interval: mv.pollMs
        running: mv.busy
        repeat: true
        onTriggered: mv.follow()
    }
}
