// The keyboard window's body: the plate with its top bar, keys, D-pad and resize corner, and the mover over it all.
// The window loads it only once the settings, look, board and state have all read.
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
            onGrip: mover.startMove()
            onFlip: body.win.flip()
            onTheme: body.win.restyle()
            onSmaller: body.win.redraw(body.win.keyboard.smaller())
            onBigger: body.win.redraw(body.win.keyboard.bigger())
            onPickSize: i => body.win.redraw(body.win.keyboard.preset(i))
            onMinimise: body.win.minimise()
            onClose: body.win.putAway()
            onTip: (name, at) => body.win.hint(name, at)
        }
        // A count, not the list: a new size then moves the keys instead of making them all again.
        Repeater {
            model: body.board.keys.length
            delegate: Key {
                required property int index
                kv: body.board.keys[index]
                ks: body.st.keys[kv.id]
                lk: body.lk
                lang: body.st.lang
                design: body.board.lang_key
                res: body.res
                s: body.s
                onHit: back => body.win.presses.tap(kv.id, back)
                onHeld: body.win.presses.hold(kv.id)
                onLetGo: body.win.presses.letGo(kv.id)
                onLost: body.win.presses.leave()
                onTip: (name, at) => body.win.hint(name, at)
            }
        }
        DPad {
            // Its box and name come with the board; whether it works, and how it scrolls, with the state.
            box: Object.assign({}, body.board.dpad, body.st.dpad)
            lk: body.lk
            res: body.res
            s: body.s
            onTip: (name, at) => body.win.hint(name, at)
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
