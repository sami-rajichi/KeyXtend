// The keyboard window's body, loaded once everything has read: the plate with its top bar and chip lane, main keys,
// side block, D-pad, email list and resize corner, and the mover over it all. The shortcuts layer takes the side block
// and hides the pad.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: body

    // The keyboard window, whose data and actions it draws and calls.
    required property Window win
    // The move and resize control, so the window can end it.
    property alias mover: mover
    // A live resize runs.
    readonly property bool resizing: mover.mode === Mover.Resizing

    readonly property var lk: win.lk
    readonly property var board: win.board
    readonly property var st: win.st
    readonly property var res: win.res
    readonly property real s: win.s
    readonly property color plateColour: lk.palette.plate
    // The shortcuts layer shows; the pad swaps back in when it goes.
    readonly property bool layered: !!board.layer
    // The board's main keys, then its side block, which a flip of the shortcuts layer alone makes again.
    readonly property var mains: board.keys.filter(k => !k.side)
    readonly property var sides: board.keys.filter(k => k.side)
    // The chip lane as last read (null without a hub): a new lane, language or settings reads it again.
    readonly property var lane: words(win.ui, win.texts, woke)
    property int woke: 0

    onLayeredChanged: if (!layered) padSwap.play()

    // Board key `kv` is a side cell new to the side block since the board was last read; none is on the first board.
    function fresh(kv) {
        return !!kv && !!win.sideWas && kv.side && win.sideWas.indexOf(kv.id) < 0;
    }
    // The lane `ui` gives now, none once it has gone; the other arguments only rerun the binding that calls it.
    function words(ui) {
        return ui ? JSON.parse(ui.words()) : null;
    }

    Connections {
        target: body.win.ui
        function onWordsChanged() {
            body.woke++;
        }
    }

    Item {
        id: plate
        width: body.board.plate.w
        height: body.board.plate.h

        Rectangle {
            anchors.fill: parent
            radius: body.lk.look.window_radius_px
            // Over a system backdrop the plate keeps the theme's share of its colour, so the material shows through.
            color: body.win.frosted ? Qt.rgba(body.plateColour.r, body.plateColour.g, body.plateColour.b, body.plateColour.a * body.lk.shape.backdrop.plate) : body.plateColour
            // A new theme or mode fades in (mock-up .kb transition).
            Behavior on color {
                ColourTween {
                    move: body.lk.motion.moves.plate
                }
            }
        }
        // The ring while moving or resizing (mock-up .kb.moving), inside the edge since the window ends there.
        Rectangle {
            visible: mover.busy
            anchors.fill: parent
            radius: body.lk.look.window_radius_px
            color: "transparent"
            border.width: body.lk.shape.press.ring_px
            border.color: body.lk.palette.ring
        }
        TopBar {
            box: body.board.bar
            lk: body.lk
            res: body.res
            s: body.s
            preset: body.board.preset
            presets: body.board.presets.length
            sizes: body.win.tongue.sizes
            dark: body.lk.dark
            lane: body.lane
            rtl: !!body.win.tongue.rtl
            onGrip: mover.startMove()
            onPick: (round, index) => body.win.ui.pickChip(round, index)
            onAsk: (kind, arg) => body.win.ui.wordsAsk(kind, arg)
            onFlip: body.win.flip()
            onTheme: body.win.restyle()
            onSmaller: body.win.redraw(body.win.keyboard.smaller())
            onBigger: body.win.redraw(body.win.keyboard.bigger())
            onPickSize: i => body.win.redraw(body.win.keyboard.preset(i))
            onMinimise: body.win.minimise()
            onClose: body.win.putAway()
            onTip: (name, at) => body.win.hint(name, at)
        }
        KeyBlock {
            anchors.fill: parent
            plate: body
            keys: body.mains
        }
        KeyBlock {
            anchors.fill: parent
            plate: body
            keys: body.sides
        }
        DPad {
            id: pad
            // It is swapping back in now.
            readonly property bool swapping: padSwap.running
            // Its box and name come with the board; whether it works, and how it scrolls, with the state.
            box: Object.assign({}, body.board.dpad, body.st.dpad)
            visible: !body.layered
            transform: Translate {
                id: padRise
            }
            lk: body.lk
            res: body.res
            s: body.s
            onTip: (name, at) => body.win.hint(name, at)
            onTapped: dir => body.win.redraw(body.win.keyboard.pad(dir))
            onHovered: on => body.win.redraw(body.win.keyboard.padHover(on))

            Swap {
                id: padSwap
                target: pad
                shift: padRise
                lk: body.lk
                s: body.s
            }
        }
        // The email list over the keys and the pad while the lane holds one; it pops in once as it opens.
        Loader {
            active: !!body.lane && !!body.lane.list
            sourceComponent: WordList {
                list: body.lane ? body.lane.list : null
                board: body.board
                rows: body.win.cfg.pageRows
                ui: body.win.ui
                lk: body.lk
                res: body.res
                s: body.s
                onAsk: (kind, arg) => body.win.ui.wordsAsk(kind, arg)
            }
        }
        Corner {
            x: parent.width - width / 2
            y: parent.height - height / 2
            lk: body.lk
            res: body.res
            s: body.s
            onClicked: mover.startResize()
            onTip: (name, at) => body.win.hint(name, at)
        }
    }

    Mover {
        id: mover
        anchors.fill: parent
        at: body.win.at
        keyboard: body.win.keyboard
        logical: body.win.spots.logical
        pollMs: body.win.cfg.pollMs
        res: body.res
        onMoved: to => body.win.at = to
        onRedraw: json => body.win.redraw(json)
        onBusyChanged: body.win.hint("", null)
    }
}
