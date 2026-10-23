// The keyboard window: the board the keyboard module lays out, drawn in the look and cut to what takes clicks, over a
// click-through shadow; it never takes focus (ADR-0013). Bound: delegates take data only from modelData and index.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: win

    // The app's Ui and Keyboard objects.
    required property var ui
    required property var keyboard

    // Settings, language, look, board and state as last read; each is null until it reads, the board while it stops.
    property var cfg: JSON.parse(ui.settingsJson())
    property var tongue: JSON.parse(ui.langJson())
    property var lk: JSON.parse(ui.lookJson())
    property var board: JSON.parse(keyboard.boardJson())
    property var st: JSON.parse(keyboard.stateJson())
    // The side cells' ids on the board before it was last read, so only cells new to the side block swap in (Plate).
    property var sideWas: null
    // The icon folder and type, compiled into the app.
    readonly property var assets: JSON.parse(ui.assets())
    // Bumped on each settings or language change, so every text is looked up again in the new language.
    property int texts: 0
    // Where the window is, in Qt units, and the same in whole units as the window takes it; its shadow is drawn around
    // `origin`, so both move in the same step and their edges meet.
    property point at: Qt.point(0, 0)
    readonly property point origin: Qt.point(Math.round(at.x), Math.round(at.y))
    // The plate size the shadow follows, set once the keys are laid out, so the shadow never runs ahead.
    property size drawn: Qt.size(0, 0)
    // Bumped when the hub loads another theme's fonts, so families are picked again from them.
    property int faces: 0
    // A system backdrop is behind the plate, so the plate lets some of it through.
    property bool frosted: false
    // The keyboard has placed the window (Placing).
    property alias placed: placing.placed
    // The guard keeps the pointer's last point outside this window at each entry, for the scroll pad's target.
    property bool watched: false
    // The scroll module runs (`scrollJson`'s `live`); the entries are watched only then.
    property bool scrolls: false
    // The plate's mover while the plate is drawn, and whether a live resize runs: its shadow then draws nothing.
    readonly property var mover: body.item ? body.item.mover : null
    readonly property bool resizing: !!body.item && body.item.resizing
    // The window's display scale, as its heal keeps it (Rescale).
    readonly property real ratio: rescale.ratio
    // The held key's beat, which repeating keys press, let go and leave.
    readonly property alias hold: beat
    // When a pressed key acts against the frame that shows it pressed.
    readonly property alias presses: frame
    // The language list with its shadow while it shows, else null.
    readonly property alias panel: lists.item

    // The minimise or close button was clicked; `putAway` leaves Window.close() unshadowed.
    signal minimise
    signal putAway
    // A press landed on the keyboard, which closes the tray menu; the press still reaches the key.
    signal touched

    readonly property bool running: board !== null
    readonly property bool ready: running && st !== null && lk !== null && cfg !== null && tongue !== null
    readonly property real s: running ? board.size : 1
    readonly property real corner: ready ? lk.shape.extra.corner_px[0] * s : 0
    // How far the corner hangs past the plate at size 1, which placement keeps on screen too.
    readonly property real unitOverhang: ready ? lk.shape.extra.corner_px[0] / 2 : 0
    // How far the window reaches past the plate's right and bottom edges: the half of the corner that hangs over.
    readonly property real overhang: unitOverhang * s
    // The window's size in Qt units: the plate and the overhang.
    readonly property real wide: ready ? board.plate.w + overhang : 0
    readonly property real high: ready ? board.plate.h + overhang : 0
    readonly property var res: kit.resources(ui, lk, tongue, assets, texts, faces)
    // Units and pop-up spots: physical pixels or Qt units on the screen that holds them.
    readonly property Spots spots: Spots {
        win: win
    }
    // What takes clicks: the plate and the resize corner overhanging it.
    readonly property var boxes: ready ? [
        { x: 0, y: 0, w: board.plate.w, h: board.plate.h },
        { x: board.plate.w - overhang, y: board.plate.h - overhang, w: corner, h: corner }
    ] : []

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: res.tr("window.keyboard")
    visible: false
    color: "transparent"
    x: rescale.keep(origin.x)
    y: rescale.keep(origin.y)
    width: rescale.keep(wide)
    height: rescale.keep(high)

    onBoxesChanged: reshape()
    onBoardChanged: Qt.callLater(() => drawn = running ? Qt.size(board.plate.w, board.plate.h) : Qt.size(0, 0))
    onVisibleChanged: if (visible) Qt.callLater(shown)
    // A running guard asks for the watch at its next try; a stopped module ends it at once.
    onScrollsChanged: {
        if (!scrolls)
            unwatch();
        else if (!guard.running)
            watch();
    }
    Component.onCompleted: {
        drawn = running ? Qt.size(board.plate.w, board.plate.h) : Qt.size(0, 0);
        Qt.callLater(rescroll);
    }

    KbWakes {
        win: win
    }
    KbRes {
        id: kit
    }
    // Keeps the size and place above across display scale changes, and places the keyboard again on a new scale.
    Rescale {
        id: rescale
        onRatioChanged: Qt.callLater(placing.rescreen)
    }
    Placing {
        id: placing
        win: win
    }
    // Owned by the keyboard, so it stays above it when the guard lifts it; the bubble owns its own.
    Loader {
        id: tips
        active: win.lk !== null && win.cfg !== null
        sourceComponent: Tip {
            transientParent: win
            lk: win.lk
            res: win.res
            ms: win.cfg.tipMs
            lift: t => win.keyboard.liftAlone(t)
        }
    }
    // The List design's language list, while the state holds one; the list and its shadow stay above the keyboard.
    Loader {
        id: lists
        active: win.ready && win.visible && !!win.st.list
        sourceComponent: LangPanel {
            win: win
        }
    }

    // Reads everything again: the settings or face changed.
    function reread() {
        cfg = JSON.parse(ui.settingsJson());
        retext();
        relook();
        reboard();
    }
    // Reads the board and the state again; the new state reaches the keys drawn now first, so each key keeps a state
    // while the shortcuts layer changes the side cells.
    function reboard() {
        sideWas = board ? board.keys.filter(k => k.side).map(k => k.id) : null;
        const next = JSON.parse(keyboard.stateJson());
        if (st && next)
            st = Object.assign({}, next, { keys: Object.assign({}, st.keys, next.keys) });
        board = JSON.parse(keyboard.boardJson());
        restate(next);
    }
    // The keyboard's language changed: only its texts are read again, and a tip in the last language closes (Tip).
    function retext() {
        tongue = JSON.parse(ui.langJson());
        texts++;
    }
    // Reads the look again, and the backdrop, which follows light or dark.
    function relook() {
        const next = JSON.parse(ui.lookJson());
        if (next)
            lk = next;
        if (visible)
            frosted = ui.backdrop(title);
    }
    // Reads the legends, lights and place again, or takes `next`, a state just read; the window goes where the keyboard
    // says, unless it is moving.
    function restate(next) {
        st = next !== undefined ? next : JSON.parse(keyboard.stateJson());
        if (st && st.at && !(mover && mover.busy))
            moveTo(st.at);
    }
    // Rereads what an action's `json` answer asks for, and the state and place too when `always`; returns the answer.
    function redraw(json, always) {
        const r = JSON.parse(json) || {};
        if (r.board)
            reboard();
        else if (always || r.state)
            restate();
        return r;
    }
    function moveTo(p) {
        const l = spots.logical(p.x, p.y);
        at = Qt.point(l.x, l.y);
    }
    // Tells the keyboard where the window is (Placing).
    function place() {
        placing.place();
    }
    // Once shown: the cut, then the guard, which also puts the backdrop on.
    function shown() {
        reshape();
        guardRun();
    }
    // Cuts the window to `boxes`, so clicks around the keyboard reach the app below; a hidden window waits for `shown`.
    function reshape() {
        if (visible && ready)
            keyboard.shape(title, JSON.stringify(boxes), rescale.ratio);
    }
    // Guards our shown windows (Guard); our window `top`, if given, lands in front until the run ends. A shown tip
    // hides first, as the guard would lift its window with it over a pop-up.
    function guardRun(top) {
        hint("", null);
        guard.run(top);
    }
    // Guards our shown windows without hiding a tip or changing the window the run puts in front, for a window that draws
    // nothing over ours, as the pointer overlay.
    function guardQuiet() {
        guard.run(null);
    }
    // While the scroll module runs, asks for the pointer's entries to be watched at each guard try until it holds.
    function watch() {
        if (scrolls && !watched && visible)
            watched = ui.watchEntries(title);
    }
    function unwatch() {
        ui.unwatchEntries(title);
        watched = false;
    }
    // Reads whether the scroll module runs; only `live` is kept, never the box.
    function rescroll() {
        const now = JSON.parse(ui.scrollJson());
        scrolls = !!now && now.live;
    }

    // Where the minimise bubble goes, in Qt units: a bottom corner of this window's screen; null when none is known.
    function bubbleSpot() {
        const c = spots.physical(at.x + drawn.width / 2, at.y + drawn.height / 2);
        const p = JSON.parse(keyboard.bubbleAt(c.x, c.y, board.bubble_corner));
        return p ? spots.logical(p.x, p.y) : null;
    }
    // Shows button `name` over `at` after a hover, or hides the tip when `name` is empty.
    function hint(name, item) {
        if (tips.item)
            tips.item.hint(name, item);
    }
    // Ends a move or resize where it is.
    function finish() {
        if (mover)
            mover.finish();
    }
    // A click on key `id`; while the language list is open, its keys drive the list instead.
    function tap(id, back) {
        hint("", null);
        redraw(listing(id) ? keyboard.panelKey(id) : keyboard.tap(id, back));
    }
    // Key `id` drives the open language list.
    function listing(id) {
        return !!(st && st.list && st.list.keys.indexOf(id) >= 0);
    }
    function flip() {
        if (ui.flipMode())
            relook();
    }
    function restyle() {
        if (ui.stepTheme())
            relook();
    }

    Loader {
        id: body
        anchors.fill: parent
        active: win.ready
        sourceComponent: Plate {
            win: win
        }
    }
    // Above the plate, it watches presses without taking them.
    PressWatch {
        onTouched: win.touched()
    }

    Guard {
        id: guard
        win: win
    }
    Hold {
        id: beat
        win: win
    }
    PressFrame {
        id: frame
        win: win
    }
    // The labels follow the layout in front, at once as the keyboard shows, since the timer rests while it is hidden.
    Timer {
        interval: win.ready ? win.board.relabel_ms : 0
        running: win.ready && win.visible
        repeat: true
        triggeredOnStart: true
        onTriggered: win.redraw(win.keyboard.relabel())
    }
}
