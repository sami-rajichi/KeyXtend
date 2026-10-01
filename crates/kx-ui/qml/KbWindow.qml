// The keyboard window: the board the keyboard module lays out, drawn in the look; it never takes focus (ADR-0013).
// Its shadow is a separate click-through window, and the window is cut to what takes clicks.
// Bound: delegates may use the window's ids, and get their data only from `modelData` and `index`.
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
    // Where the window is, in Qt units; its shadow reads it too, so both move in the same step.
    property point at: Qt.point(0, 0)
    // The plate size the shadow follows, set once the keys are laid out, so the shadow never runs ahead.
    property size drawn: Qt.size(0, 0)
    // Font files loaded so far, so families are picked again once they are in.
    property int fontsIn: 0
    // A system backdrop is behind the plate, so the plate lets some of it through.
    property bool frosted: false
    // The keyboard has placed the window; a restarted keyboard places it again.
    property bool placed: false

    // The minimise or close button was clicked.
    signal minimise
    signal close
    // A press landed on the keyboard, which closes the tray menu; the press still reaches the key.
    signal touched

    readonly property bool running: board !== null
    readonly property bool ready: running && st !== null && lk !== null && cfg !== null && tongue !== null
    readonly property real s: running ? board.size : 1
    readonly property real corner: ready ? lk.shape.extra.corner_px[0] * s : 0
    readonly property var res: resources(texts, fontsIn)
    // Units and pop-up spots: physical pixels or Qt units on the screen that holds them.
    readonly property Spots spots: Spots {
        win: win
    }
    // What takes clicks: the plate and the resize corner overhanging it.
    readonly property var boxes: ready ? [
        { x: 0, y: 0, w: board.plate.w, h: board.plate.h },
        { x: board.plate.w - corner / 2, y: board.plate.h - corner / 2, w: corner, h: corner }
    ] : []

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: res.tr("window.keyboard")
    visible: false
    color: "transparent"
    x: at.x
    y: at.y
    width: ready ? board.plate.w + corner / 2 : 0
    height: ready ? board.plate.h + corner / 2 : 0

    onBoxesChanged: reshape()
    onBoardChanged: Qt.callLater(() => drawn = running ? Qt.size(board.plate.w, board.plate.h) : Qt.size(0, 0))
    onVisibleChanged: if (visible) Qt.callLater(shown)
    onRunningChanged: if (!running) placed = false
    Screen.onDevicePixelRatioChanged: reshape()
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
            // A top-bar button's UI Automation id: its icons.toml [bar] id, the same in every language.
            barId: id => "bar." + id,
            dpad: id => ui.nameOf("dpad", id),
            space: parts => ui.space(JSON.stringify(parts)),
            // A dimmed button's tip: its name, then when it comes, on two lines.
            dimTip: (name, later) => name + "\n" + later
        };
    }
    // The Latin and Arabic families (each list's first that Qt has, else its last), and the keyboard's and tray's.
    function families(f) {
        const all = Qt.fontFamilies();
        const pick = list => list.find(n => all.indexOf(n) >= 0) || list[list.length - 1];
        const latin = pick(lk.look.latin_fonts);
        const arabic = pick(lk.look.arabic_fonts);
        const of = l => l && l.rtl ? arabic : latin;
        return { latin: latin, arabic: arabic, ui: of(tongue), tray: of(tongue && tongue.tray) };
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
        if (st && st.at && !(body.item && body.item.mover.busy))
            moveTo(st.at);
    }
    // Rereads what an action's `json` answer asks for.
    function redraw(json) {
        const r = JSON.parse(json);
        if (r && r.board)
            board = JSON.parse(keyboard.boardJson());
        if (r && (r.state || r.board))
            restate();
    }
    function moveTo(p) {
        const l = spots.logical(p.x, p.y);
        at = Qt.point(l.x, l.y);
    }
    // Tells the keyboard where the window is; the first time, it answers with the saved or home place.
    function place() {
        if (!running)
            return;
        const p = spots.physical(at.x, at.y);
        placed = true;
        redraw(keyboard.placed(p.x, p.y, board.plate.w * p.dpr, board.plate.h * p.dpr));
    }

    // Once shown: the cut, then the guard, which also puts the backdrop on.
    function shown() {
        reshape();
        guardRun();
    }
    // Cuts the window to `boxes`, so clicks around the keyboard reach the app below; a hidden window waits for `shown`.
    function reshape() {
        if (visible && ready)
            keyboard.shape(title, JSON.stringify(boxes), Screen.devicePixelRatio);
    }
    // Guards our shown windows, trying again after the delay while none shows yet; our window titled `top`, if given,
    // lands in front. A shown tip hides first, as the guard would lift its window with it over a pop-up.
    function guardRun(top) {
        hint("", null);
        if (top)
            guard.top = top;
        guard.restart();
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
        if (body.item)
            body.item.mover.finish();
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

    Timer {
        id: guard
        // The window to land in front, until the run ends.
        property string top: ""
        interval: win.cfg ? win.cfg.guardDelayMs : 0
        onTriggered: {
            const again = win.keyboard.guard(guard.top);
            win.reshape();
            win.frosted = win.ui.backdrop(win.title);
            if (again)
                restart();
            else
                guard.top = "";
        }
    }
    Timer {
        interval: win.ready ? win.board.relabel_ms : 0
        running: win.ready && win.visible
        repeat: true
        onTriggered: win.redraw(win.keyboard.relabel())
    }
}
