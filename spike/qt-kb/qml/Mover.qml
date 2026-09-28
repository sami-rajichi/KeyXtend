// Moving and resizing: one click starts, the window follows the pointer, and the next click anywhere ends it.
// Fill the window with it, last, so its own catcher lies over every control while it runs.
pragma ComponentBehavior: Bound
import QtQuick
import QtQml.Models
import KeyXtend.Spike

Item {
    id: mv

    // Nothing, following the pointer to move, or following it to resize.
    enum Mode { Idle, Moving, Resizing }

    // The window's place and the board that sizes it; Qt units for a physical point; what screen readers call a catcher.
    required property point at
    required property var board
    required property var logical
    required property string name

    // What runs now.
    property int mode: Mover.Idle
    // Where the pointer holds the window while moving.
    property point grab: Qt.point(0, 0)
    // True between the two clicks.
    readonly property bool busy: mode !== Mover.Idle

    // The window's new place while moving, in Qt units.
    signal moved(point to)
    // The size changed while resizing.
    signal resized

    // The pointer in Qt units, or null when Windows could not say.
    function pointer() {
        const c = JSON.parse(board.cursor());
        return c.note ? null : logical(c.x, c.y);
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
        const p = pointer();
        if (!p)
            return;
        board.resizeBegin(p.x, p.y);
        mode = Mover.Resizing;
    }

    // One poll: moves or resizes to where the pointer is now.
    function follow() {
        const p = pointer();
        if (!p)
            return;
        if (mode === Mover.Moving)
            moved(Qt.point(p.x - grab.x, p.y - grab.y));
        else if (mode === Mover.Resizing && board.resizeAt(p.x, p.y))
            resized();
    }

    // The second click: the window stays where it is, at the size it reached.
    function finish() {
        if (mode === Mover.Resizing)
            board.resizeEnd();
        mode = Mover.Idle;
    }

    // A click on the keyboard itself ends it too, whatever lies below.
    MouseArea {
        anchors.fill: parent
        visible: mv.busy
        hoverEnabled: true
        Accessible.role: Accessible.Button
        Accessible.name: mv.name
        Accessible.onPressAction: mv.finish()
        onClicked: mv.finish()
    }
    // One catcher per screen, since screens may have different scales.
    Instantiator {
        model: Qt.application.screens
        delegate: Catch {
            required property var modelData
            area: modelData
            name: mv.name
            visible: mv.busy
            onPicked: mv.finish()
        }
    }
    Timer {
        interval: mv.board.pollMs
        running: mv.busy
        repeat: true
        onTriggered: mv.follow()
    }
}
