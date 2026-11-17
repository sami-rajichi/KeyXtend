// The Voice panel with its shadow (design "The gear and the Voice panel"): a list panel fitted as every one is
// (PanelGlow), but placed by the hub above the dictation bar, else below it, off the keyboard (owner), and owned by
// neither, so it stays while the keyboard hides. It asks the scroll pad to page it at the guard's pace until that holds.
pragma ComponentBehavior: Bound
import QtQuick

PanelGlow {
    id: panel

    // The dictation bar, whose Voice object it sends clicks to and above which it opens, and the round of its view.
    required property Window bar
    required property int round

    // A press landed on the panel, which closes the tray menu.
    signal touched

    // The shown bar's box in physical pixels (JSON), or null while it hides.
    function barBox() {
        if (!bar.visible)
            return JSON.stringify(null);
        const a = win.spots.physical(bar.x, bar.y);
        return JSON.stringify({ x: a.x, y: a.y, w: bar.width * a.dpr, h: bar.height * a.dpr });
    }
    // Where a `w` by `h` panel goes in Qt units, with its fit, as the hub places it by the bar and the keyboard.
    function spotFor(w, h) {
        const p = JSON.parse(bar.voice.panelAt(win.spots.beside(NaN)[0], barBox(), w, h));
        return p ? Object.assign(win.spots.logical(p.x, p.y), { fit: p.fit }) : null;
    }

    transientParent: null
    pane: pop
    wide: win.lk.shape.voice.panel_px
    spot: (w, h) => panel.spotFor(w, h)

    VoiceBox {
        id: pop
        ui: panel.win.ui
        voice: panel.bar.voice
        lk: panel.win.lk
        res: panel.win.res
        s: panel.win.s * panel.fit
        view: panel.view
        round: panel.round
        rows: panel.win.cfg.pageRows
        ar: !!panel.win.tongue.rtl
        onTouched: panel.touched()
        onRescaled: panel.rescaled()
        onUnitChanged: panel.relaid()
        Component.onCompleted: panel.start()
    }
    // Placed again as the bar or the keyboard shows, hides, moves or resizes, so a bar that grows never covers it;
    // PanelGlow follows the keyboard's moves.
    Connections {
        target: panel.bar
        function onXChanged() {
            panel.place(true);
        }
        function onYChanged() {
            panel.place(true);
        }
        function onWidthChanged() {
            panel.place(true);
        }
        function onHeightChanged() {
            panel.place(true);
        }
        function onVisibleChanged() {
            panel.place(true);
        }
    }
    Connections {
        target: panel.win
        function onVisibleChanged() {
            panel.place(true);
        }
    }
    // The guard marks only the keyboard window's own panels, so this one asks again at the guard's pace.
    Timer {
        interval: panel.win.cfg.guardDelayMs
        repeat: true
        running: pop.visible && !pop.paged
        onTriggered: pop.mark()
    }
}
