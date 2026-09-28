// Qt face of the P1 spike: a plain key block and tools row that never take focus.
// Bound: delegates may use the window's ids, and get their data only from `modelData` and `index`.
pragma ComponentBehavior: Bound
import QtQuick
import KeyXtend.Spike

Window {
    id: win

    // Every key with its box from spike-core, in one flat list.
    property var keys: []
    // The tool buttons with their boxes.
    property var tools: []
    // Last note shown at the end of the status line.
    property string note: kb.guardPending

    flags: Qt.Tool | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: kb.title
    visible: true
    color: pal.window
    width: kb.faceWidth
    height: kb.faceHeight + line.implicitHeight + kb.gapPx

    Keyboard { id: kb }
    Tools { id: tl }
    Voice {
        id: vc
        onCaption: json => win.caption(json)
    }
    SystemPalette { id: pal }

    // The keys of every row in one list.
    function flat(json) {
        return JSON.parse(json).reduce((all, row) => all.concat(row), []);
    }

    function tap(code) {
        note = kb.tap(code);
    }

    // Shows `text` on the status line unless it is empty.
    function say(text) {
        if (text && text.length > 0)
            note = text;
    }

    // Qt units for physical pixel (px, py), using the scale of the screen that holds it; Qt keeps each screen's corner unscaled.
    function logical(px, py) {
        const all = Qt.application.screens;
        for (let i = 0; i < all.length; i++) {
            const s = all[i];
            const d = s.devicePixelRatio;
            const inX = px >= s.virtualX && px < s.virtualX + s.width * d;
            const inY = py >= s.virtualY && py < s.virtualY + s.height * d;
            if (inX && inY)
                return { x: s.virtualX + (px - s.virtualX) / d, y: s.virtualY + (py - s.virtualY) / d, dpr: d };
        }
        return { x: px, y: py, dpr: 1 };
    }

    function tool(i) {
        const r = JSON.parse(tl.tool(i));
        say(r.note);
        if (r.snip) {
            over.begin(logical(r.snip.x, r.snip.y), r.snip);
            // The image loads at once, so the private copy on disk can go now.
            say(tl.forgetFrozen());
            say(tl.guard(tl.overlayTitle));
        }
        if (r.mic)
            vc.click();
    }

    // Shows a caption from the voice worker on top of every window, and any note it carries; a note may come alone.
    function caption(json) {
        const r = JSON.parse(json);
        say(r.note);
        if (r.text === undefined)
            return;
        const shown = bar.visible;
        bar.say(r, logical(r.at[0], r.at[1]));
        if (!shown)
            say(tl.guard(bar.title));
    }

    // Shows, moves or hides the pill, and shows any new fill note.
    function tick() {
        const r = JSON.parse(tl.tick());
        say(r.note);
        const at = r.show || r.move;
        if (at)
            pill.place(logical(at[0], at[1]));
        if (r.show) {
            pill.visible = true;
            say(tl.guard(tl.pillTitle));
        }
        if (r.hide)
            pill.visible = false;
    }

    Component.onCompleted: {
        keys = flat(kb.rowsJson());
        tools = JSON.parse(tl.buttonsJson());
    }

    Repeater {
        model: win.keys
        delegate: Cap {
            required property var modelData
            x: modelData.x
            y: modelData.y
            width: modelData.w
            height: modelData.h
            label: modelData.label
            fontPx: kb.fontPx
            onClicked: win.tap(modelData.code)
        }
    }
    Repeater {
        model: win.tools
        delegate: Cap {
            required property var modelData
            required property int index
            x: modelData.x
            y: modelData.y
            width: modelData.w
            height: modelData.h
            label: modelData.label
            fontPx: kb.fontPx
            onClicked: win.tool(index)
        }
    }

    Text {
        id: line
        x: kb.gapPx
        y: kb.faceHeight
        width: kb.faceWidth - 2 * kb.gapPx
        elide: Text.ElideRight
        font.pixelSize: kb.fontPx
        color: pal.windowText
        Accessible.role: Accessible.StaticText
        Accessible.name: text
        text: kb.line(win.note)
    }

    Pill {
        id: pill
        title: tl.pillTitle
        width: tl.pillWidth
        height: tl.pillHeight
        label: tl.pillLabel
        fontPx: kb.fontPx
        onCopyClicked: win.say(tl.copy())
    }
    Caption {
        id: bar
        title: vc.barTitle
        width: vc.barWidth
        height: vc.barHeight
        fontPx: kb.fontPx
        pad: kb.gapPx
    }
    Overlay {
        id: over
        title: tl.overlayTitle
        edge: tl.snipEdge
        hint: tl.snipHint
        onPicked: {
            const n = tl.pick();
            if (n.length > 0) {
                over.end();
                win.note = n;
            }
        }
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
    Timer {
        interval: tl.pollMs
        running: true
        repeat: true
        onTriggered: win.tick()
    }
}
