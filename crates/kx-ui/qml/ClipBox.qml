// The clipboard panel's window (spec §7; mock-up .clip): head, search line, filters and paged list, or the Delete-all
// question or preview in their place, then the footer and the storage line, whose text a note can take over. It keeps
// its size as views change, sends each click to the hub as `Ui.clipAct` and never takes focus (ADR-0013).
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: box

    // The app's Ui object, the look, fonts, icons and texts, the keyboard size, the panel's view (`Ui.clip`), rows per
    // page, and whether the keyboard's language reads right to left.
    required property var ui
    required property var lk
    required property var res
    required property real s
    required property var view
    required property int rows
    required property bool rtl

    // A press landed on the panel, which closes the tray menu; the press still reaches the part under it.
    signal touched
    // The display scale changed, so its place, set in the old Qt units, is stale.
    signal rescaled

    // The box whose fade, rise and scale the shadow follows.
    readonly property alias body: body
    readonly property var pn: lk.shape.panel
    readonly property var c: lk.shape.clip
    readonly property real radius: lk.look.window_radius_px * pn.radius_share
    readonly property bool asking: !!view.confirm
    readonly property bool previewing: !!view.preview
    // A row fits its tick box, its tile, or two title lines over the meta line in the tallest of its fonts, so every
    // page and language keeps the panel's size.
    readonly property real titleH: Math.max(latinLine.implicitHeight, arabicLine.implicitHeight)
    readonly property real metaH: Math.max(latinMeta.implicitHeight, arabicMeta.implicitHeight)
    readonly property real lines: 2 * titleH + c.meta_px[1] * s + metaH
    readonly property real rowH: Math.max(c.tick_px[0] * s, c.thumb_px[1] * s, lines) + 2 * c.row_pad_px * s

    // The panel holds the page mark, so the scroll pad pages it while it shows (ADR-0029); the mark goes with the window.
    property bool paged: false
    // Its height per unit of size as last laid out, from which a fit to the room beside the keyboard starts.
    property real unit: 0

    // Shows the panel, popping in (mock-up popIn), and asks for the page mark at once.
    function open() {
        visible = true;
        mark();
        popIn.restart();
    }
    // Asks for the page mark while the panel shows: as it opens, then at each guard try until it holds.
    function mark() {
        if (visible && !paged)
            paged = ui.markPages(title);
    }
    function act(kind, arg) {
        ui.clipAct(kind, arg);
    }

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: view.words.window
    color: "transparent"
    visible: false
    // Whole units up, as a window's size is whole.
    width: rescale.keep(Math.ceil(pn.width_px * s))
    height: rescale.keep(Math.ceil(col.height))

    ClipNames {
        id: names
    }
    // Keeps the size above across display scale changes.
    Rescale {
        id: rescale
        onRatioChanged: box.rescaled()
    }

    Rectangle {
        id: body
        width: box.width
        height: box.height
        radius: box.radius
        color: box.lk.palette.pop_bg
        // High contrast outlines it, as the P1 panel did.
        border.width: box.lk.contrast ? box.lk.shape.line.px : 0
        border.color: box.lk.palette.pop_line

        Column {
            id: col
            onPositioningComplete: box.unit = height / box.s
            width: parent.width
            ClipHead {
                width: col.width
                view: box.view
                names: names
                busy: box.asking || box.previewing
                lk: box.lk
                res: box.res
                s: box.s
                onAct: (kind, arg) => box.act(kind, arg)
            }
            Item {
                width: col.width
                height: list.y + list.height

                Item {
                    anchors.fill: parent
                    visible: !box.asking && !box.previewing
                    ClipSearch {
                        id: seek
                        x: box.c.filters_pad_px[1] * box.s
                        y: box.c.filters_pad_px[0] * box.s
                        width: col.width - 2 * x
                        search: box.view.search
                        rtl: box.rtl
                        lk: box.lk
                        res: box.res
                        s: box.s
                        uid: box.res.uid(names.group, names.ids.search)
                    }
                    ClipFilters {
                        id: chips
                        y: seek.y + seek.height
                        width: col.width
                        filters: box.view.filters
                        names: names
                        rtl: box.rtl
                        lk: box.lk
                        res: box.res
                        s: box.s
                        onAct: (kind, arg) => box.act(kind, arg)
                    }
                    ClipList {
                        id: list
                        y: chips.y + chips.height
                        width: col.width
                        view: box.view
                        rows: box.rows
                        rowH: box.rowH
                        names: names
                        ui: box.ui
                        lk: box.lk
                        res: box.res
                        s: box.s
                        onAct: (kind, arg) => box.act(kind, arg)
                    }
                }
                ClipConfirm {
                    anchors.fill: parent
                    confirm: box.previewing ? null : box.view.confirm
                    yes: box.view.words.yes
                    no: box.view.words.no
                    names: names
                    lk: box.lk
                    res: box.res
                    s: box.s
                    onAct: (kind, arg) => box.act(kind, arg)
                }
                ClipPreview {
                    anchors.fill: parent
                    visible: box.previewing
                    shown: box.view.preview
                    words: box.view.words
                    rows: box.rows
                    rowH: box.rowH
                    names: names
                    ui: box.ui
                    lk: box.lk
                    res: box.res
                    s: box.s
                    onAct: (kind, arg) => box.act(kind, arg)
                }
            }
            ClipFoot {
                width: col.width
                view: box.view
                names: names
                lk: box.lk
                res: box.res
                s: box.s
                onAct: (kind, arg) => box.act(kind, arg)
            }
            // The storage line, with a note in its text's place, so the note adds little height.
            ClipRoom {
                width: col.width
                text: box.view.room
                note: box.view.note
                folder: box.view.words.openFolder
                names: names
                lk: box.lk
                res: box.res
                s: box.s
                onOpen: box.ui.openFolder()
            }
        }
        PressWatch {
            onTouched: box.touched()
        }
        // Hidden lines laid out as the rows' labels are, which round each line up, so a row always fits them.
        Label {
            id: latinLine
            visible: false
            text: " "
            font.family: box.res.fam.latin
            font.pixelSize: box.pn.row_text_px[0] * box.s
        }
        Label {
            id: arabicLine
            visible: false
            text: " "
            font.family: box.res.fam.arabic
            font.pixelSize: box.pn.row_text_px[1] * box.s
        }
        Label {
            id: latinMeta
            visible: false
            text: " "
            font.family: box.res.fam.latin
            font.pixelSize: box.c.meta_px[0] * box.s
        }
        Label {
            id: arabicMeta
            visible: false
            text: " "
            font.family: box.res.fam.arabic
            font.pixelSize: box.c.meta_px[0] * box.s
        }
    }

    PopIn {
        id: popIn
        target: body
        move: box.lk.motion.moves.panel
        amount: box.lk.motion.amount
    }
}
