// The keyboard window: the board the keyboard module lays out, drawn in the look and cut to what takes clicks, over a
// click-through shadow; it never takes focus (ADR-0013). Bound: delegates take data only from modelData and index.
pragma ComponentBehavior: Bound
import QtQuick
import QtQml.Models

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
    // The icon folder and type and the font files, compiled into the app.
    readonly property var assets: JSON.parse(ui.assets())
    // Bumped on each settings or language change, so every text is looked up again in the new language.
    property int texts: 0
    // Where the window is, in Qt units, and the same in whole units as the window takes it; its shadow is drawn around
    // `origin`, so both move in the same step and their edges meet.
    property point at: Qt.point(0, 0)
    readonly property point origin: Qt.point(Math.round(at.x), Math.round(at.y))
    // The plate size the shadow follows, set once the keys are laid out, so the shadow never runs ahead.
    property size drawn: Qt.size(0, 0)
    // Font files loaded so far, so families are picked again once they are in.
    property int fontsIn: 0
    // A system backdrop is behind the plate, so the plate lets some of it through.
    property bool frosted: false
    // The keyboard has placed the window (Placing).
    property alias placed: placing.placed
    // The plate's mover while the plate is drawn, and whether a live resize runs: its shadow then draws nothing.
    readonly property var mover: body.item ? body.item.mover : null
    readonly property bool resizing: !!body.item && body.item.resizing
    // The window's display scale, as its heal keeps it (Rescale).
    readonly property real ratio: rescale.ratio

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
    readonly property var res: resources(texts, fontsIn)
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
    Component.onCompleted: drawn = running ? Qt.size(board.plate.w, board.plate.h) : Qt.size(0, 0)

    Connections {
        target: win.ui
        function onSettingsChanged() {
            win.reread();
        }
        function onLookChanged() {
            win.relook();
        }
        function onTextsChanged() {
            win.retext();
        }
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
        active: win.ready && win.visible && !!win.st.list
        sourceComponent: LangPanel {
            win: win
        }
    }
    Instantiator {
        model: win.assets.fonts
        delegate: FontLoader {
            required property string modelData
            source: modelData
            onStatusChanged: if (status === FontLoader.Ready) win.fontsIn++
        }
    }

    // The fonts, icons and texts (keyboard's and tray's) the parts draw with; `n` and `f` rerun it as they change.
    function resources(n, f) {
        const fam = lk ? families(f) : {};
        return {
            fam: fam,
            icons: assets.icons,
            ext: assets.ext,
            tr: key => ui.text(key),
            tray: key => ui.trayText(key),
            bar: id => ui.nameOf("bar", id),
            // A control's UI Automation id is its text key, such as `bar.close`, the same in every language.
            uid: (group, id) => group + "." + id,
            dpad: id => ui.nameOf("dpad", id),
            space: parts => ui.space(JSON.stringify(parts)),
            // A tip with a note, such as when a dimmed button comes: its name, then the note, on two lines.
            noteTip: (name, note) => name + "\n" + note
        };
    }
    // The Latin, Arabic and mark families (each list's first that Qt has, else its last), and the keyboard's and
    // tray's.
    function families(f) {
        const all = Qt.fontFamilies();
        const pick = list => list.find(n => all.indexOf(n) >= 0) || list[list.length - 1];
        const latin = pick(lk.look.latin_fonts);
        const arabic = pick(lk.look.arabic_fonts);
        const mark = pick(lk.look.mark_fonts);
        const of = l => l && l.rtl ? arabic : latin;
        return { latin: latin, arabic: arabic, mark: mark, ui: of(tongue), tray: of(tongue && tongue.tray) };
    }

    // Reads everything again: the settings or face changed.
    function reread() {
        cfg = JSON.parse(ui.settingsJson());
        retext();
        relook();
        board = JSON.parse(keyboard.boardJson());
        restate();
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
    // Reads the legends, lights and place again; the window goes where the keyboard says, unless it is moving.
    function restate() {
        st = JSON.parse(keyboard.stateJson());
        if (st && st.at && !(mover && mover.busy))
            moveTo(st.at);
    }
    // Rereads what an action's `json` answer asks for, and the state and place too when `always`.
    function redraw(json, always) {
        const r = JSON.parse(json);
        if (r && r.board)
            board = JSON.parse(keyboard.boardJson());
        if (always || (r && (r.state || r.board)))
            restate();
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
        const listing = st && st.list && st.list.keys.indexOf(id) >= 0;
        redraw(listing ? keyboard.panelKey(id) : keyboard.tap(id, back));
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
    // The labels follow the layout in front, at once as the keyboard shows, since the timer rests while it is hidden.
    Timer {
        interval: win.ready ? win.board.relabel_ms : 0
        running: win.ready && win.visible
        repeat: true
        triggeredOnStart: true
        onTriggered: win.redraw(win.keyboard.relabel())
    }
}
