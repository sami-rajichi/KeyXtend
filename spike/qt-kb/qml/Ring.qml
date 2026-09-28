// Gate G12's ring test: holds loop at the pointer in a click-through window that follows it every frame and counts frames.
pragma ComponentBehavior: Bound
import QtQuick
import KeyXtend.Spike

Window {
    id: win

    // Qt units for a physical point (the keyboard's `logical`), the pointer as `{x, y}` or `{note}`, and the look.
    required property var logical
    required property var cursor
    required property var lk

    // The test strip's label for the switch.
    readonly property string label: rd.label
    // Shown but not drawn yet, so the first frame's set-up is not counted.
    property bool fresh: false

    // A note for the status line.
    signal note(string text)

    title: rd.title
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
        | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    visible: false
    color: "transparent"
    width: face.width
    height: face.height

    // Starts the loop at the pointer, or stops it and writes the frame stats.
    function toggle() {
        if (visible) {
            face.loop.stop();
            const n = rd.stop();
            visible = false;
            if (n)
                note(n);
            return;
        }
        const n = follow();
        if (n)
            note(n);
        rd.start();
        fresh = true;
        visible = true;
        face.loop.restart();
    }

    // Centres the window on the pointer in one move; returns why not when Windows cannot say where it is.
    function follow() {
        const p = cursor();
        if (p.note)
            return p.note;
        const l = logical(p.x, p.y);
        const nx = Math.round(l.x - width / 2), ny = Math.round(l.y - height / 2);
        if (nx !== x || ny !== y)
            setGeometry(nx, ny, width, height);
        return "";
    }

    onFrameSwapped: {
        if (fresh) {
            fresh = false;
            rd.start();
        }
    }

    RingData {
        id: rd
    }
    FrameAnimation {
        running: win.visible
        onTriggered: {
            rd.frame();
            win.follow();
        }
    }
    RingFace {
        id: face
        lk: win.lk
        afterMs: rd.afterMs
        waveMs: rd.waveMs
        gapMs: rd.gapMs
    }
}
