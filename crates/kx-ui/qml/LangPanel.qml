// The List design's language list with its shadow: placed by the language key above the keyboard, then opened. Its
// picks and page buttons go to the keyboard, which redraws from the answer; a press on it counts as the keyboard's.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: panel

    // The keyboard window, whose state, look and keyboard the list reads and calls.
    required property Window win

    lk: win.lk
    box: Qt.rect(list.x, list.y, list.width, list.height)
    radius: list.radius
    follow: list.body

    // Puts the list above the keyboard, its left edge at the language key's.
    function place() {
        const w = panel.win;
        const key = w.board.keys.find(k => k.kind === "lang");
        const p = w.popSpot(list.width, list.height, key ? w.at.x + key.x : NaN);
        if (p) {
            list.x = p.x;
            list.y = p.y;
        }
    }
    // Placed again as the keyboard moves or changes size.
    Connections {
        target: panel.win
        function onAtChanged() {
            panel.place();
        }
        function onDrawnChanged() {
            panel.place();
        }
    }

    LangList {
        id: list
        ui: panel.win.ui
        lk: panel.win.lk
        res: panel.win.res
        s: panel.win.s
        view: panel.win.st.list
        rows: panel.win.cfg.pageRows
        rtl: panel.win.cfg.rtl
        title: panel.win.res.tr("window.list")
        onPick: i => panel.win.redraw(panel.win.keyboard.langPick(i))
        onPage: id => panel.win.redraw(panel.win.keyboard.panelKey(id))
        onTouched: panel.win.touched()
        onWidthChanged: if (visible) panel.place()
        onHeightChanged: if (visible) panel.place()
        Component.onCompleted: Qt.callLater(() => {
            panel.place();
            panel.visible = true;
            list.open();
            panel.win.guardRun();
        })
    }
}
