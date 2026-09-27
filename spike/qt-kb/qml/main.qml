// Qt face of the P1 spike: a plain key block that never takes focus.
// Bound: the key delegate may use the window's ids, and gets its data only from `modelData`.
pragma ComponentBehavior: Bound
import QtQuick
import KeyXtend.Spike

Window {
    id: win

    // Key edge width; drawn in the text colour, so edges keep at least 3:1 contrast.
    readonly property int edge: 1
    // Every key with its box from spike-core, in one flat list.
    property var keys: []
    // Last note shown at the end of the status line.
    property string note: kb.guardPending

    flags: Qt.Tool | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: kb.title
    visible: true
    color: pal.window
    width: kb.blockWidth
    height: kb.blockHeight + line.implicitHeight + kb.gapPx

    Keyboard { id: kb }
    SystemPalette { id: pal }

    // The keys of every row in one list.
    function flat(json) {
        return JSON.parse(json).reduce((all, row) => all.concat(row), []);
    }

    function tap(code) {
        note = kb.tap(code);
    }

    Component.onCompleted: keys = flat(kb.rowsJson())

    Repeater {
        model: win.keys
        delegate: Rectangle {
            id: cap
            required property var modelData
            x: modelData.x
            y: modelData.y
            width: modelData.w
            height: modelData.h
            color: area.pressed ? pal.highlight : pal.base
            border.width: win.edge
            border.color: pal.windowText
            Accessible.role: Accessible.Button
            Accessible.name: modelData.label
            Accessible.onPressAction: win.tap(modelData.code)

            Text {
                width: parent.width
                anchors.verticalCenter: parent.verticalCenter
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
                text: cap.modelData.label
                font.pixelSize: kb.fontPx
                color: area.pressed ? pal.highlightedText : pal.text
                // The key already carries the label; hide the text so readers say it once.
                Accessible.ignored: true
            }
            MouseArea {
                id: area
                anchors.fill: parent
                onClicked: win.tap(cap.modelData.code)
            }
        }
    }

    Text {
        id: line
        x: kb.gapPx
        y: kb.blockHeight
        width: kb.blockWidth - 2 * kb.gapPx
        elide: Text.ElideRight
        font.pixelSize: kb.fontPx
        color: pal.windowText
        Accessible.role: Accessible.StaticText
        Accessible.name: text
        text: kb.line(win.note)
    }

    // Guards our window; an empty result means no window was visible yet, so it tries again.
    Timer {
        interval: kb.guardDelayMs
        running: true
        onTriggered: {
            const result = kb.guard();
            if (result.length > 0)
                win.note = result;
            else
                start();
        }
    }
    Timer {
        interval: kb.relabelMs
        running: true
        repeat: true
        onTriggered: {
            const json = kb.relabel();
            if (json.length > 0)
                win.keys = win.flat(json);
        }
    }
}
