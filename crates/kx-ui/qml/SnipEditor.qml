// Snip's editor (design "The editor", snip-editor.js editor()): its own window where the hub keeps it, no smaller than
// the theme's least (edge 19), with the title bar, the picture's tools and options, the picture and the resize corner;
// a video has none (ADR-0060). It never takes focus; a press on it aims the keyboard's letters at Snip, the close
// question dims it, and a system dialog in front drops it from the top.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: glow

    // Snip's windows.
    required property var snips

    readonly property var sh: lk.shape.snip
    readonly property var n: snips.names
    readonly property var v: snips.view
    // The editor's view and its picture's tools. Both go with the view's editor, on every way out, so the picture's
    // pixels go too (G8).
    property var ed: null
    readonly property var pic: ed ? ed.picture : null
    // It shows once its stage is up, its place known and its picture in, which the hub's synchronous provider serves at
    // once.
    readonly property bool shows: !!v && !!v.edit && snips.shown === n.stages.edit && !!win.place && canvas.loaded
    readonly property bool asking: shows && !!v.ask
    // It drew and its pop ended, so a click lands where it rests.
    readonly property bool ready: drawn && !pop.running
    // Maximised, so the box before it comes back.
    readonly property bool full: win.before !== null
    // The editor's window, its picture and its mover, for the close question and the tests.
    readonly property alias editor: win
    readonly property alias canvas: canvas
    readonly property alias mover: mover
    readonly property alias tools: tools

    lk: snips.lk
    res: snips.res
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput | (snips.low ? 0 : Qt.WindowStaysOnTopHint)
    box: Qt.rect(win.x, win.y, win.width, win.height)
    radius: lk.look.window_radius_px
    follow: frame
    shadowed: win
    visible: shows

    onVChanged: ed = v && v.edit ? v.edit : null

    // Shows a tip over `item`, or hides it for an empty `text`.
    function hint(text, item) {
        tips.hint(text, item);
    }
    // Copy or Save `id`: the picture is flattened at its own pixels first.
    function keep(id) {
        canvas.flatten(id);
    }
    // Maximise fills the work area of the editor's screen; a second click puts back the box it had, or with no screen
    // to fit it on keeps the editor where it shows. The hub keeps each for the next editor.
    function maximise() {
        if (full) {
            const back = JSON.parse(snips.snip.editorPlaced(win.before.x, win.before.y, win.before.w, win.before.h));
            if (back) {
                win.place = back;
                win.before = null;
            }
            return;
        }
        const was = win.place;
        const work = was ? JSON.parse(snips.snip.editorFull(was.x, was.y, was.w, was.h)) : null;
        if (!work)
            return;
        win.before = was;
        win.place = work;
    }
    // Opens where the hub keeps it once its stage comes, maximised over its box if it was left so.
    function open() {
        mover.stop();
        const at = JSON.parse(snips.snip.editorAt());
        win.before = at && at.back ? at.back : null;
        win.place = at ? at.place : null;
    }

    Connections {
        target: glow.snips
        function onStageChanged() {
            if (glow.snips.stage === glow.n.stages.edit)
                glow.open();
            else
                glow.mover.stop();
        }
    }
    SnipOpen {
        snips: glow.snips
        shadow: glow
        ready: glow.ready
        stage: glow.n.stages.edit
    }

    Window {
        id: win

        // Its box in physical pixels (`{x, y, w, h}`) as the hub keeps it, and the box before a maximise, or null.
        property var place: null
        property var before: null
        readonly property rect at: place ? glow.snips.box(place) : Qt.rect(0, 0, 0, 0)

        // Set just after its shadow's flags, so it stays above its shadow (Glow.owns).
        flags: glow.owns
        title: glow.res.tr("window.snip-editor")
        color: "transparent"
        x: Math.round(at.x)
        y: Math.round(at.y)
        width: Math.max(1, Math.round(at.width))
        height: Math.max(1, Math.round(at.height))

        // Made once, its shadow first, before it ever shows, so each Edit shows windows already made.
        Component.onCompleted: glow.snips.snip.make(title)
        onVisibleChanged: if (visible && glow.lk.motion.moves.snip_pop.ms > 0) pop.restart()

        Rectangle {
            id: frame
            anchors.fill: parent
            radius: glow.lk.look.window_radius_px
            color: glow.lk.palette.pop_bg
            border.width: glow.lk.shape.line.px
            border.color: glow.lk.palette.pop_line
            Accessible.role: Accessible.Pane
            Accessible.name: win.title

            Column {
                id: rows
                width: parent.width
                SnipHead {
                    width: rows.width
                    editor: glow
                }
                SnipTools {
                    id: tools
                    visible: !!glow.pic
                    width: rows.width
                    editor: glow
                }
            }
            SnipCanvas {
                id: canvas
                anchors.top: rows.bottom
                anchors.bottom: parent.bottom
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: glow.sh.editor_pad_px[1]
                editor: glow
            }
            SnipButton {
                visible: !glow.full
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                lk: glow.lk
                res: glow.res
                name: glow.res.tr("snip.edit-" + glow.n.own.size)
                icon: glow.lk.icons.snip.edit.size
                uid: "snip.edit." + glow.n.own.size
                tip: glow.hint
                onClicked: mover.startResize()
            }
            // The close question dims the editor and takes its clicks.
            Rectangle {
                anchors.fill: parent
                visible: glow.asking
                radius: parent.radius
                color: glow.lk.common.snip_dim
                MouseArea {
                    anchors.fill: parent
                }
            }
            // A press on the editor aims the keyboard's letters at Snip; the press still reaches what lies below.
            PressWatch {
                onTouched: glow.snips.aim()
            }
            SnipMove {
                id: mover
                anchors.fill: parent
                snips: glow.snips
                editor: win
            }
        }
        PopIn {
            id: pop
            target: frame
            move: glow.lk.motion.moves.snip_pop
            amount: glow.lk.motion.amount
        }
        Tip {
            id: tips
            transientParent: win
            lk: glow.lk
            res: glow.res
            ms: glow.snips.tipMs
            lift: () => {}
        }
    }
}
