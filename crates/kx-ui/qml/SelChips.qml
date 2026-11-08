// The selection strip (design 4, "Keycaps"): the keyboard's own keycaps on its plate, by the selected text where the
// hub places it. It pops in, shows a clicked keycap's tick for a moment, then fades; it never takes focus.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: strip

    // The app's Ui object, the look, fonts and icons, the keyboard's size and key row height, whether the keyboard
    // speaks Arabic, our windows' boxes to keep clear of (`clear()`, physical pixels) and Qt units for a physical spot.
    required property var ui
    required property var lk
    required property var res
    required property real s
    required property real row
    required property bool ar
    required property var clear
    required property var logical

    // The strip as the hub writes it, or null while none shows. Its keycaps and reader name stay as last shown while it
    // hides, so no keycap reads a strip that is gone and the window keeps its title.
    property var view: null
    property var chips: []
    property string name: ""
    // What a keycap past the end of a shorter strip reads until it goes.
    readonly property var none: ({ id: "", word: "", done: "" })

    // The strip shows (true) or hides (false); its shadow follows.
    signal showing(bool on)
    // A press landed on the strip, which closes the tray menu; the press still reaches the keycap.
    signal touched

    // The plate whose pop, fade and rise the shadow follows, and the tick's timer, for the tests.
    readonly property alias body: body
    readonly property alias done: done
    readonly property var sh: lk.shape.chips
    readonly property var mv: lk.motion.moves
    readonly property real pad: sh.pad_px * s
    // No keycap is ticked, so a click still acts.
    readonly property bool live: !!view && view.acted === null

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: name
    color: "transparent"
    visible: false
    // Whole units up, as a window's size is whole.
    width: rescale.keep(Math.ceil(caps.width + 2 * pad))
    height: rescale.keep(Math.ceil(row + 2 * pad))

    // Keeps the size above across display scale changes, and the place by the text on a new scale.
    Rescale {
        id: rescale
        onRatioChanged: Qt.callLater(strip.follow)
    }

    Connections {
        target: strip.ui
        function onSelChanged() {
            strip.read();
        }
        // A new `[ui] language` wakes the settings, not the strip.
        function onSettingsChanged() {
            strip.read();
        }
    }
    Component.onCompleted: read()
    onWidthChanged: Qt.callLater(follow)
    onHeightChanged: Qt.callLater(follow)

    // Reads the hub's strip: a new round pops in, the same one is measured again, and a tick shows for its time.
    function read() {
        const next = JSON.parse(ui.sel());
        if (!next) {
            view = null;
            shut();
            return;
        }
        const fresh = !view || view.round !== next.round;
        view = next;
        chips = next.chips;
        name = next.name;
        if (fresh || !visible)
            open();
        else
            follow();
        // A ticked strip with no room has already ended its round.
        if (view && view.acted !== null && !done.running && !fade.running)
            done.restart();
    }
    // Places the strip by its text and pops it in; nothing shows where the hub finds no room.
    function open() {
        fade.stop();
        done.stop();
        if (row <= 0 || !place())
            return drop();
        body.opacity = 1;
        body.y = 0;
        body.scale = 1;
        showing(true);
        visible = true;
        if (mv.chips.ms > 0)
            pop.restart();
    }
    // Places the strip again, as its size, the keyboard or a panel changed; one that found no room tries again.
    function follow() {
        if (!view)
            return;
        if (!visible) {
            if (live)
                open();
            return;
        }
        if (!place())
            drop();
    }
    // No room: a live strip hides until there is room again, and a ticked one ends its round with the hub.
    function drop() {
        if (live)
            shut();
        else
            gone();
    }
    // Asks the hub where the strip goes for its size in physical pixels, scaled once by its screen's scale.
    function place() {
        caps.forceLayout();
        const sc = view.scale;
        const p = JSON.parse(ui.selAt(view.round, width * sc, height * sc, JSON.stringify(clear())));
        if (!p)
            return false;
        const at = logical(p.x, p.y);
        x = Math.round(at.x);
        y = Math.round(at.y);
        return true;
    }
    // The tick's time is up: the strip fades with its shadow, or goes at once with reduced motion.
    function fadeOut() {
        done.stop();
        if (mv.chips_fade.ms > 0)
            fade.restart();
        else
            gone();
    }
    // The ticked strip went: the hub forgets its round.
    function gone() {
        const round = view ? view.round : -1;
        view = null;
        shut();
        ui.selFaded(round);
    }
    function shut() {
        pop.stop();
        fade.stop();
        done.stop();
        if (!visible)
            return;
        visible = false;
        showing(false);
    }

    Timer {
        id: done
        interval: strip.lk.motion.shows.chips_done
        onTriggered: strip.fadeOut()
    }

    Rectangle {
        id: body
        width: strip.width
        height: strip.height
        radius: strip.lk.look.window_radius_px
        color: strip.lk.palette.plate
        border.width: strip.lk.contrast ? strip.lk.shape.line.px : 0
        border.color: strip.lk.palette.pop_line
        Accessible.role: Accessible.ToolBar
        Accessible.name: strip.name

        // Copy, Cut, Paste, Translate from left to right in every language (ADR-0020).
        Row {
            id: caps
            x: strip.pad
            y: strip.pad
            layoutDirection: Qt.LeftToRight
            Repeater {
                model: strip.chips.length
                delegate: SelKey {
                    required property int index
                    chip: strip.chips[index] || strip.none
                    acted: !!strip.view && strip.view.acted === chip.id
                    live: strip.live
                    icons: !!strip.view && strip.view.icons
                    words: !!strip.view && strip.view.words
                    lk: strip.lk
                    res: strip.res
                    s: strip.s
                    row: strip.row
                    ar: strip.ar
                    onHit: strip.ui.selAct(strip.view.round, chip.id)
                }
            }
        }
        PressWatch {
            onTouched: strip.touched()
        }
    }

    // It pops in (popIn), and once its tick has shown it fades and rises with its shadow (.chips.go).
    PopIn {
        id: pop
        target: body
        move: strip.mv.chips
        amount: strip.lk.motion.amount
    }
    ParallelAnimation {
        id: fade
        onFinished: strip.gone()
        Tween {
            target: body
            property: "opacity"
            to: 0
            move: strip.mv.chips_fade
        }
        Tween {
            target: body
            property: "y"
            to: -strip.lk.motion.amount.fade_rise_px * strip.s
            move: strip.mv.chips_fade
        }
    }
}
