// The keyboard window: the designed keyboard, drawn from spike-core's view, state and look; it never takes focus.
// Its shadow is a separate click-through window, and the window is cut to what takes clicks.
// Bound: delegates may use the window's ids, and get their data only from `modelData` and `index`.
pragma ComponentBehavior: Bound
import QtQuick
import QtQml.Models
import KeyXtend.Spike

Window {
    id: win

    // Top bar, strip, fonts and icons (fixed); the keys at this size; their legends and lights; the look now.
    property var bar: JSON.parse(board.barJson())
    property var view: JSON.parse(board.viewJson())
    property var st: JSON.parse(board.stateJson())
    property var lk: JSON.parse(board.lookJson())
    // Last note shown at the end of the status line.
    property string note: kb.guardPending
    // Where the window is, in Qt units; its shadow reads it too, so both move in the same step.
    property point at: Qt.point(0, 0)
    // The plate size the shadow follows, set once the keys are laid out, so the shadow never runs ahead.
    property size drawn: Qt.size(0, 0)
    // Font files loaded so far, so families are picked again once they are in.
    property int fontsIn: 0

    // The minimise button was clicked.
    signal minimise

    readonly property real s: view.size
    readonly property var strip: lk.shape.extra.strip_px
    readonly property real corner: lk.shape.extra.corner_px[0] * s
    readonly property var res: ({ fam: families(fontsIn), icons: bar.icons, ext: bar.iconExt })
    // The keyboard's screen in Qt units; tooltips stay on it.
    readonly property rect screenBox: Qt.rect(Screen.virtualX, Screen.virtualY, Screen.width, Screen.height)
    // What takes clicks: the plate, the resize corner overhanging it, and the test strip.
    readonly property var boxes: [
        { x: 0, y: 0, w: view.plate.w, h: view.plate.h },
        { x: view.plate.w - corner / 2, y: view.plate.h - corner / 2, w: corner, h: corner },
        { x: 0, y: view.plate.h + strip[1], w: view.plate.w, h: strip[0] }
    ]

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: kb.title
    visible: false
    color: "transparent"
    x: at.x
    y: at.y
    width: view.plate.w + corner / 2
    height: view.plate.h + Math.max(corner / 2, strip[1] + strip[0])

    onBoxesChanged: reshape()
    onViewChanged: Qt.callLater(() => drawn = Qt.size(view.plate.w, view.plate.h))
    Screen.onDevicePixelRatioChanged: reshape()

    Keyboard { id: kb }
    Board {
        id: board
        onLookChanged: win.relook()
    }
    Extras {
        id: ex
        logical: win.logical
        cursor: () => JSON.parse(board.cursor())
        lk: win.lk
        res: win.res
        onNote: text => win.say(text)
        // The mic key turns red while the mic records.
        onRecordingChanged: {
            board.setRec(ex.recording);
            win.restate();
        }
    }
    // The Settings key's Arabic test panel (gate G7).
    Panel {
        id: panel
        lk: win.lk
        res: win.res
        s: win.s
        logical: win.logical
        onNote: text => win.say(text)
        onTip: (name, at) => tip.hint(name, at)
    }
    // Not owned by the keyboard: Qt shows no owned window while its owner is hidden, and the bubble needs tips too.
    Tip {
        id: tip
        transientParent: null
        lk: win.lk
        res: win.res
        ms: win.bar.tipMs
        area: win.screenBox
        arabic: text => kb.isArabic(text)
    }
    Instantiator {
        model: win.bar.fonts
        delegate: FontLoader {
            required property string modelData
            source: modelData
            onStatusChanged: if (status === FontLoader.Ready) win.fontsIn++
        }
    }

    // The first family in `list` that Qt has, else the last one; `n` makes it run again as fonts load.
    function pick(list, n) {
        const all = Qt.fontFamilies();
        return list.find(f => all.indexOf(f) >= 0) || list[list.length - 1];
    }
    function families(n) {
        return { latin: pick(lk.look.latin_fonts, n), arabic: pick(lk.look.arabic_fonts, n) };
    }

    // Cuts the window to `boxes`, so clicks around the keyboard reach the app below; a hidden window waits for `open`.
    function reshape() {
        if (visible)
            say(kb.shape(JSON.stringify(boxes), Screen.devicePixelRatio));
    }

    // Where the minimise bubble goes, in Qt units, or null with a note when Windows could not say.
    function bubbleSpot() {
        const p = JSON.parse(kb.bubbleAt());
        say(p.note);
        return p.note ? null : logical(p.x, p.y);
    }

    // Guards our windows and lifts the one called `title` to the top.
    function guardTop(title) {
        ex.guard(title);
    }

    // Shows button `name` over `at` after a hover, or hides the tip when `name` is empty.
    function hint(name, at) {
        tip.hint(name, at);
    }

    // Ends a move or resize where it is.
    function finish() {
        mover.finish();
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
            const sc = all[i];
            const d = sc.devicePixelRatio;
            const inX = px >= sc.virtualX && px < sc.virtualX + sc.width * d;
            const inY = py >= sc.virtualY && py < sc.virtualY + sc.height * d;
            if (inX && inY)
                return { x: sc.virtualX + (px - sc.virtualX) / d, y: sc.virtualY + (py - sc.virtualY) / d, dpr: d };
        }
        return { x: px, y: py, dpr: 1 };
    }

    function tap(id, back) {
        tip.hint("", null);
        const r = JSON.parse(board.tap(id, back));
        say(r.note);
        if (r.tool !== undefined)
            ex.tool(r.tool);
        if (r.panel)
            panel.toggle(view.plate);
        restate();
    }

    // Reads the legends and lights again.
    function restate() {
        st = JSON.parse(board.stateJson());
    }

    // Reads the look again.
    function relook() {
        lk = JSON.parse(board.lookJson());
    }

    function resized(changed) {
        if (changed)
            view = JSON.parse(board.viewJson());
    }

    // The top bar's light or dark switch.
    function flip() {
        board.flipMode();
        relook();
    }

    // Starts at the bottom centre of the primary screen's work area.
    Component.onCompleted: {
        const p = JSON.parse(kb.startAt(width, height));
        say(p.note || board.startNote);
        const l = p.note ? { x: Screen.virtualX, y: Screen.virtualY } : logical(p.x, p.y);
        at = Qt.point(l.x, l.y);
        drawn = Qt.size(view.plate.w, view.plate.h);
    }

    Item {
        id: content
        anchors.fill: parent

        Item {
            id: plate
            width: win.view.plate.w
            height: win.view.plate.h

            Rectangle {
                anchors.fill: parent
                radius: win.lk.look.window_radius_px
                color: win.lk.palette.plate
                // A new theme or mode fades in (mock-up .kb transition).
                Behavior on color {
                    ColourTween {
                        move: win.lk.motion.moves.plate
                    }
                }
            }
            // The ring while moving or resizing (mock-up .kb.moving), inside the edge since the window ends there.
            Rectangle {
                visible: mover.busy
                anchors.fill: parent
                radius: win.lk.look.window_radius_px
                color: "transparent"
                border.width: win.lk.shape.press.ring_px
                border.color: win.lk.palette.ring
            }
            TopBar {
                box: win.view.bar
                lk: win.lk
                res: win.res
                cfg: win.bar.bar
                s: win.s
                preset: win.view.preset
                dark: win.lk.dark
                onGrip: mover.startMove()
                onChip: i => win.say(board.chip(i))
                onFlip: win.flip()
                onSmaller: win.resized(board.step(false))
                onBigger: win.resized(board.step(true))
                onPickSize: i => win.resized(board.preset(i))
                onMinimise: win.minimise()
                onClose: Qt.quit()
                onTip: (name, at) => tip.hint(name, at)
            }
            // A count, not the list: a new size then moves the keys instead of making them all again.
            Repeater {
                model: win.view.keys.length
                delegate: Key {
                    required property int index
                    kv: win.view.keys[index]
                    ks: win.st.keys[kv.id]
                    lk: win.lk
                    lang: win.st.lang
                    res: win.res
                    s: win.s
                    onHit: back => win.tap(kv.id, back)
                    onTip: (name, at) => tip.hint(name, at)
                }
            }
            DPad {
                box: win.view.dpad
                lk: win.lk
                res: win.res
                s: win.s
                buttons: win.bar.bar.dpad
                onTip: (name, at) => tip.hint(name, at)
            }
            Corner {
                x: parent.width - width / 2
                y: parent.height - height / 2
                lk: win.lk
                res: win.res
                s: win.s
                button: win.bar.bar.corner
                onClicked: mover.startResize()
                onTip: (name, at) => tip.hint(name, at)
            }
        }

        ThemeStrip {
            y: plate.height + win.strip[1]
            width: plate.width
            height: win.strip[0]
            lk: win.lk
            res: win.res
            themes: win.bar.themes
            modes: win.bar.bar.modes
            theme: win.lk.theme
            mode: win.lk.mode
            status: kb.line(win.note)
            ringLabel: ex.ringLabel
            ringOn: ex.ringOn
            onFlipRing: ex.flipRing()
            onPickTheme: i => {
                board.setTheme(i);
                win.relook();
                win.view = JSON.parse(board.viewJson());
            }
            onPickMode: i => {
                board.setMode(i);
                win.relook();
            }
        }
        Mover {
            id: mover
            anchors.fill: parent
            at: win.at
            board: board
            logical: win.logical
            name: win.bar.bar.catch_name
            onMoved: to => win.at = to
            onResized: win.resized(true)
            onBusyChanged: tip.hint("", null)
        }
    }

    // Guards our window; an empty result means no window was visible yet, so it tries again.
    Timer {
        interval: kb.guardDelayMs
        running: true
        onTriggered: {
            const result = kb.guard();
            if (result.length > 0) {
                win.note = result;
                win.reshape();
            } else {
                start();
            }
        }
    }
    Timer {
        interval: kb.relabelMs
        running: true
        repeat: true
        onTriggered: if (board.relabel()) win.restate()
    }
}
