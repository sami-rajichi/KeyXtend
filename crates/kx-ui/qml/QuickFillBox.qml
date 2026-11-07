// Quick-fill's window (design "The panel"): the head, then the page the view's sheet names over its two lines and
// five rows, then the footer with the pager and the note. It keeps its size as pages change, sends each click to the
// QuickFill object with the round it was drawn from, and never takes focus (ADR-0013).
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: box

    // The app's Ui object, the QuickFill object, the look, fonts, icons and texts, the keyboard size, the panel's
    // view, rows per page, whether the keyboard's language reads right to left, and whether a line takes the letters
    // (ADR-0040).
    required property var ui
    required property var fill
    required property var lk
    required property var res
    required property real s
    required property var view
    required property int rows
    property bool rtl: false
    property bool aimed: false

    // A press on the panel, unless it follows another inside the double-click time, or a reader's on a line, aims the
    // keys at it, which only the keyboard knows (ADR-0040).
    signal aiming
    // A press landed on the panel, which closes the tray menu; the press still reaches the part under it.
    signal touched
    // The display scale changed, so its place, set in the old Qt units, is stale.
    signal rescaled

    // The box whose fade, rise and scale the shadow follows, the click names, and what counts the wheel's steps.
    readonly property alias body: body
    readonly property alias names: names
    readonly property alias wheel: wheel
    readonly property var pn: lk.shape.panel
    readonly property var fs: lk.shape.fill
    readonly property real radius: lk.look.window_radius_px * pn.radius_share
    readonly property real side: pn.body_pad_px[1] * s
    // A question waits for its answer on the page: Delete entry's, Remove place's, Replace's or a foreign box's.
    readonly property bool asking: !!view.sheet.confirm || !!view.sheet.replace || view.sheet.is === names.pages.foreign
    readonly property int per: Math.max(1, rows)
    // A row fits its tile, its buttons, a line that takes the letters, or its title over its hint in the tallest of
    // their fonts, so every page and language keeps the panel's size.
    readonly property real titleH: Math.max(latinLine.implicitHeight, arabicLine.implicitHeight)
    readonly property real hintH: Math.max(latinHint.implicitHeight, arabicHint.implicitHeight)
    readonly property real lines: titleH + pn.hint_px[1] * s + hintH
    readonly property real rowH: Math.max(pn.tile_px[1] * s, lk.shape.clip.button_px[0] * s, pn.field_px[0] * s, lines) + 2 * pn.row_pad_px * s
    // The page's two lines over its rows: the first a text field high, the second a button high.
    readonly property real lineH: pn.field_px[0] * s
    readonly property real bandY: (fs.top_px[0] + fs.top_px[1]) * s + lineH
    readonly property real topH: bandY + (lk.shape.clip.button_px[0] + fs.top_px[2]) * s
    readonly property real slotsH: per * rowH + (per - 1) * pn.row_px[1] * s
    // Each page's view by the `is` its sheet carries (QuickFillNames.pages).
    readonly property var sheets: {
        const p = names.pages, all = {};
        [[p.list, listC], [p.labels, listC], [p.values, valuesC], [p.entry, entryC], [p.edit, editC], [p.add, addC], [p.backup, backupC], [p.places, placesC], [p.folders, foldersC], [p.password, passwordC], [p.found, foundC], [p.foreign, foundC]].forEach(([is, c]) => all[is] = c);
        return all;
    }

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
    // Sends click `kind` on `id` with `arg`, with the round of the view it was drawn from.
    function act(kind, id, arg) {
        fill.act(kind, id, arg, view.round);
    }
    // A panel button's click.
    function press(id) {
        act(names.acts.button, id, 0);
    }
    // A notch of the wheel: what the page buttons do but silently, and nothing past either end or on a question.
    function turn(by) {
        if (!asking && view.page.at + by >= 0 && view.page.at + by < view.page.count)
            act(names.acts.wheel, "", by);
    }
    // Panel button `id` as QuickFillButtons takes it: its label, gate id and `icon`, its look (ghost unless given), and
    // whether it `can` act (yes unless given).
    function button(id, icon, tone, can) {
        return { id: id, label: view.buttons[id], icon: icon, tone: tone, can: can, uid: res.uid(names.group, id) };
    }
    // Hub words `text`, in the keyboard's language, as a heading or a hint takes them.
    function worded(text) {
        return { text: text, rtl: rtl, arabic: rtl };
    }
    // Place `r` as a slot draws it: its kind's tile, its name over `hint`, and its buttons with `extra` on each.
    function placeRow(r, hint, extra) {
        return { key: r.name.text, icon: names.placeIcon(lk.icons.fill, r.kind), title: r.name, hint: hint, buttons: r.buttons.map(b => Object.assign({}, b, extra)) };
    }

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: view.words.window
    color: "transparent"
    visible: false
    // Whole units up, as a window's size is whole.
    width: rescale.keep(Math.ceil(fs.width_px * s))
    height: rescale.keep(Math.ceil(col.height))

    QuickFillNames {
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

        PanelWheel {
            id: wheel
            notch: box.pn.wheel_notch
            rest: box.pn.wheel_rest_s
            onTurn: by => box.turn(by)
        }
        Column {
            id: col
            onPositioningComplete: box.unit = height / box.s
            width: parent.width
            QuickFillHead {
                width: col.width
                view: box.view
                names: names
                lk: box.lk
                res: box.res
                s: box.s
                onPressed: id => box.press(id)
            }
            // A new page rises in as a panel pops in; the first comes with the panel's own pop-in.
            Rise {
                id: rise
                width: col.width
                height: box.topH + box.slotsH + box.pn.body_pad_px[2] * box.s
                move: box.lk.motion.fill.sheet
                from: box.lk.motion.amount.pop_rise_px * box.s
                Loader {
                    id: page
                    property bool first: true
                    anchors.fill: parent
                    sourceComponent: box.view.sheet.pick === names.picks.keepass ? importC : box.sheets[box.view.sheet.is] || null
                    onLoaded: {
                        if (!first)
                            rise.play();
                        first = false;
                    }
                }
            }
            QuickFillFoot {
                width: col.width
                view: box.view
                asking: box.asking
                names: names
                ui: box.ui
                lk: box.lk
                res: box.res
                s: box.s
                onTurn: by => box.act(names.acts.page, "", by)
            }
        }
        PressWatch {
            onTouched: box.touched()
            onAlone: box.aiming()
        }
        // Hidden lines laid out as the rows' texts are, which round each line up, so a row always fits them.
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
            id: latinHint
            visible: false
            text: " "
            font.family: box.res.fam.latin
            font.pixelSize: box.pn.hint_px[0] * box.s
        }
        Label {
            id: arabicHint
            visible: false
            text: " "
            font.family: box.res.fam.arabic
            font.pixelSize: box.pn.hint_px[0] * box.s
        }
    }

    PopIn {
        id: popIn
        target: body
        move: box.lk.motion.moves.panel
        amount: box.lk.motion.amount
    }

    Component {
        id: listC
        QuickFillList {
            box: box
        }
    }
    Component {
        id: valuesC
        QuickFillLabels {
            box: box
        }
    }
    Component {
        id: entryC
        QuickFillEntry {
            box: box
        }
    }
    Component {
        id: editC
        QuickFillEdit {
            box: box
        }
    }
    Component {
        id: addC
        QuickFillAdd {
            box: box
        }
    }
    Component {
        id: backupC
        QuickFillBackup {
            box: box
        }
    }
    Component {
        id: placesC
        QuickFillPlaces {
            box: box
        }
    }
    Component {
        id: foldersC
        QuickFillFolders {
            box: box
        }
    }
    Component {
        id: importC
        QuickFillImport {
            box: box
        }
    }
    Component {
        id: passwordC
        QuickFillPassword {
            box: box
        }
    }
    Component {
        id: foundC
        QuickFillFound {
            box: box
        }
    }
}
