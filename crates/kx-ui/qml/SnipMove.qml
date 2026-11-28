// Moving and resizing the editor (edge 19): a click on its grip or corner starts, the editor follows the pointer as the
// hub keeps it on a screen and no smaller than its least, and the next click anywhere ends it there, kept for the next
// editor. Fill the editor with it, last, so its catcher lies over every control while it runs.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: mv

    // Nothing, following the pointer to move, or following it to resize.
    enum Mode { Idle, Moving, Resizing }

    // Snip's windows, and the editor's window, whose `place` it sets.
    required property var snips
    required property var editor

    property int mode: SnipMove.Idle
    // How far the pointer sits from the corner it holds, in physical pixels: the top left's while moving, the bottom
    // right's while resizing.
    property point grab: Qt.point(0, 0)
    // True between the two clicks.
    readonly property bool busy: mode !== SnipMove.Idle
    readonly property var win: snips.win
    // The catchers' text key, also their UI Automation id in every language, and what readers call them.
    readonly property string uid: "catch"
    readonly property string name: snips.res.tr(uid)
    // The catchers, one per screen.
    readonly property alias covers: covers

    // The pointer in physical pixels, or null when Windows could not say or no keyboard window is there.
    function pointer() {
        return win ? JSON.parse(win.keyboard.cursor()) : null;
    }
    // The first click on the grip: the editor follows the pointer, held where it was grabbed.
    function startMove() {
        start(SnipMove.Moving, (p, at) => Qt.point(p.x - at.x, p.y - at.y));
    }
    // The first click on the corner: its bottom right follows the pointer.
    function startResize() {
        start(SnipMove.Resizing, (p, at) => Qt.point(at.x + at.w - p.x, at.y + at.h - p.y));
    }
    function start(next, held) {
        const [p, at] = [pointer(), editor.place];
        if (!p || !at)
            return;
        grab = held(p, at);
        mode = next;
    }
    // One poll: the box the pointer asks for, as the hub keeps it.
    function follow() {
        const [p, at] = [pointer(), editor.place];
        if (!p || !at || !busy)
            return;
        const moving = mode === SnipMove.Moving;
        const r = moving ? [p.x - grab.x, p.y - grab.y, at.w, at.h] : [at.x, at.y, p.x + grab.x - at.x, p.y + grab.y - at.y];
        const kept = JSON.parse(snips.snip.editorFit(...r.map(Math.round)));
        if (kept)
            editor.place = kept;
    }
    // The second click: the editor stays where it is, at the size it reached, which the hub keeps for next time.
    function finish() {
        if (!busy)
            return;
        follow();
        mode = SnipMove.Idle;
        const at = editor.place;
        const kept = at ? JSON.parse(snips.snip.editorPlaced(at.x, at.y, at.w, at.h)) : null;
        if (kept)
            editor.place = kept;
    }
    // The editor went: a move under way ends with nothing kept.
    function stop() {
        mode = SnipMove.Idle;
    }

    // A click on the editor itself ends it too, at let-go wherever the pointer went (ADR-0024); the second click of a
    // double click on the grip or corner is ignored, so a shaky start never ends at once.
    Act {
        anchors.fill: parent
        visible: mv.busy
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
        interval: mv.win ? mv.win.cfg.pollMs : 0
        running: mv.busy && !!mv.win
        repeat: true
        onTriggered: mv.follow()
    }
}
