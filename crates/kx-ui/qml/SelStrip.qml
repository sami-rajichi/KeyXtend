// The selection strip over its shadow, as App shows it: not owned by the keyboard's shadow, so it shows while the
// keyboard is minimised, and placed clear of the keyboard, its bubble and the open panels.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: sel

    // The app, whose keyboard window gives the look, size, texts and spots, and whose windows the strip keeps clear of.
    required property var app

    // The strip's own window, for App and the tests.
    readonly property alias chips: chips
    readonly property Window face: app.face
    // One key row: a character key's height on the board drawn, 0 while none is.
    readonly property real row: rowOf(face.board)
    // Our shown windows' boxes in physical pixels; a move, resize, show or hide of any places the strip again.
    readonly property string kept: JSON.stringify(boxes())

    transientParent: null
    lk: face.lk
    box: Qt.rect(chips.x, chips.y, chips.width, chips.height)
    radius: face.lk ? face.lk.look.window_radius_px : 0
    follow: chips.body

    onKeptChanged: chips.follow()

    // The height of a character key on `board`, or 0 with none.
    function rowOf(board) {
        const key = board ? board.keys.find(k => !k.side && k.kind === "char") : null;
        return key ? key.h : 0;
    }
    // The shown keyboard's plate, then each other shown window of ours, in whole physical pixels.
    function boxes() {
        const f = face;
        const panes = [f.panel ? f.panel.list : null, f.clips ? f.clips.pane : null, f.fills ? f.fills.pane : null];
        const shown = [app.bubble].concat(panes).filter(w => !!w && w.visible);
        const others = shown.map(w => boxOf(w.x, w.y, w.width, w.height));
        return f.visible ? [boxOf(f.at.x, f.at.y, f.drawn.width, f.drawn.height)].concat(others) : others;
    }
    // The Qt box at (`x`, `y`), `w` by `h`, in physical pixels on the screen that holds it.
    function boxOf(x, y, w, h) {
        const a = face.spots.physical(x, y);
        return { x: Math.round(a.x), y: Math.round(a.y), w: Math.round(w * a.dpr), h: Math.round(h * a.dpr) };
    }

    SelChips {
        id: chips
        ui: sel.app.ui
        lk: sel.face.lk
        res: sel.face.res
        s: sel.face.s
        row: sel.row
        ar: !!sel.face.tongue && !!sel.face.tongue.rtl
        clear: () => JSON.parse(sel.kept)
        logical: (x, y) => sel.face.spots.logical(x, y)
        onShowing: on => sel.app.shadow(sel, on, chips)
        onTouched: sel.app.menu.shut()
    }
}
