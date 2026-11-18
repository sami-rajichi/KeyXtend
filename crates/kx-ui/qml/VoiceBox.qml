// The Voice panel's window (design "The gear and the Voice panel"; mock-up .vp): the header, the bands, the page's
// paged list or Compare's set text, the folder line, then the footer. It sends each click to the hub as `Voice.act` on
// the round it was drawn from, and never takes focus (ADR-0013).
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: box

    // The app's Ui object (page numbers and the scroll pad's mark), the hub's Voice object, the look, fonts, icons and
    // texts, the size, the panel's view and its round, rows per page, and whether the keyboard speaks Arabic.
    required property var ui
    required property var voice
    required property var lk
    required property var res
    required property real s
    required property var view
    required property int round
    required property int rows
    required property bool ar

    // A press landed on the panel, which closes the tray menu; the press still reaches the part under it.
    signal touched
    // The display scale changed, so its place, set in the old Qt units, is stale.
    signal rescaled

    // The box whose fade, rise and scale the shadow follows, the footer, and what counts the wheel's steps.
    readonly property alias body: body
    readonly property alias foot: foot
    readonly property alias wheel: wheel
    readonly property VoiceNames names: VoiceNames {}
    readonly property var pn: lk.shape.panel
    readonly property real radius: lk.look.window_radius_px * pn.radius_share
    readonly property var sheet: view.sheet
    readonly property bool asking: view.foot.some(b => b.kind === names.acts.confirm)
    // A question holds the pages still, but a card's, which asks about the whole card, lets its two parts turn.
    readonly property bool held: asking && sheet.is !== names.sheets.card
    readonly property int per: Math.max(1, rows)
    // A row fits its tick box, its tile, or its name over its line and a progress line, in the tallest of its fonts, so
    // every page and language keeps the panel's size.
    readonly property real nameH: Math.max(latinName.implicitHeight, arabicName.implicitHeight)
    readonly property real metaH: Math.max(latinMeta.implicitHeight, arabicMeta.implicitHeight)
    readonly property real lines: nameH + pn.hint_px[1] * s + metaH + (pn.row_pad_px + lk.shape.voice.progress_px) * s
    readonly property real rowH: Math.max(pn.row_px[0] * s, lk.shape.clip.tick_px[0] * s, pn.tile_px[1] * s, lines + 2 * pn.row_pad_px * s)
    readonly property real pageH: per * rowH + (per - 1) * pn.row_px[1] * s
    // The bands over the list: the page's note, the slow-PC line and the Cloud side's band with their button to the Cloud
    // page, the untested line, why cloud services send nothing, a card's or Compare's warning, then a note, which readers
    // hear at once.
    readonly property var bands: {
        const i = lk.icons.voice;
        const models = sheet.is === names.sheets.models;
        const all = [];
        const band = (icon, text, more) => all.push(Object.assign({ icon: icon, text: text, name: "", alert: false, parts: [], warn: false, button: null }, more));
        if (view.top)
            band(i.panel.tops[view.top.id] || "", view.top.text, { name: view.top.name || "" });
        if (sheet.slow)
            band(i.panel.slow, sheet.slow.text, { button: sheet.slow.button });
        if (sheet.band)
            band(i.panel.cloud.switch, sheet.band.text, { button: sheet.band.button });
        if (models && sheet.untested)
            band(i.panel.untested, sheet.untested, {});
        if (sheet.off)
            band(i.panel.cloud.off, sheet.off, {});
        if (sheet.warn)
            band("", sheet.warn.text, { parts: sheet.warn.parts, warn: true });
        if (sheet.send)
            band("", sheet.send, { warn: true });
        if (view.note)
            band(i.notes[view.note.id] || "", view.note.text, { alert: true });
        return all;
    }
    // The sheets drawn as a page of their own rather than a paged list.
    readonly property var whole: [names.sheets.ready, names.sheets.card, names.sheets.service]

    // The scroll pad pages the panel while it shows (ADR-0029); the mark goes with the window.
    property bool paged: false
    // Its height per unit of size as last laid out, from which a fit to the room on screen starts.
    property real unit: 0

    // Shows the panel, popping in (mock-up .vp.in), and asks for the page mark at once.
    function open() {
        visible = true;
        mark();
        popIn.restart();
    }
    // Asks for the page mark while the panel shows, until it holds.
    function mark() {
        if (visible && !paged)
            paged = ui.markPages(title);
    }
    // Sends click `kind` with `id` and `arg` on the round of the view it was drawn from.
    function send(kind, id, arg) {
        voice.act(kind, id, arg, round);
    }
    // One of the hub's buttons was clicked.
    function press(b) {
        send(b.kind, b.id, b.arg);
    }
    // A notch of the wheel: what the page buttons do, silently, and nothing past either end or while the pages are held.
    function turn(by) {
        const at = view.pages.at + by;
        if (!held && at >= 0 && at < view.pages.count)
            send(names.acts.wheel, "", by);
    }

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: view.window
    color: "transparent"
    visible: false
    // Whole units up, as a window's size is whole.
    width: rescale.keep(Math.ceil(lk.shape.voice.panel_px * s))
    height: rescale.keep(Math.ceil(col.height))

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
        // High contrast outlines it, as the other panels do.
        border.width: box.lk.contrast ? box.lk.shape.line.px : 0
        border.color: box.lk.palette.pop_line
        Accessible.role: Accessible.Dialog
        Accessible.name: box.view.title

        PanelWheel {
            id: wheel
            notch: box.pn.wheel_notch
            rest: box.pn.wheel_rest_s
            onTurn: by => box.turn(by)
        }
        Column {
            id: col
            width: parent.width
            onPositioningComplete: box.unit = height / box.s
            VoiceHead {
                width: col.width
                host: box
            }
            Column {
                x: box.pn.body_pad_px[1] * box.s
                width: col.width - 2 * x
                visible: box.bands.length > 0 || !!box.sheet.switch || !!box.sheet.sides
                topPadding: box.pn.body_gap_px * box.s
                spacing: box.pn.body_gap_px * box.s
                VoiceToggle {
                    visible: !!box.sheet.switch
                    width: parent.width
                    host: box
                    toggle: box.sheet.switch || ({ kind: "", label: "", value: "", on: false })
                    icon: box.lk.icons.voice.panel.cloud.switch
                    uid: box.res.uid(box.names.group, box.names.acts.cloud)
                }
                VoiceSides {
                    visible: !!box.sheet.sides
                    host: box
                    sides: box.sheet.sides || ({ name: "", shown: "", chips: [] })
                }
                Repeater {
                    model: box.bands
                    delegate: VoiceBand {
                        required property var modelData
                        width: parent ? parent.width : 0
                        host: box
                        icon: modelData.icon
                        text: modelData.text
                        name: modelData.name
                        alert: modelData.alert
                        parts: modelData.parts
                        warn: modelData.warn
                        button: modelData.button
                    }
                }
            }
            VoiceList {
                visible: box.whole.indexOf(box.sheet.is) < 0
                width: col.width
                host: box
            }
            VoiceRead {
                visible: box.sheet.is === box.names.sheets.ready
                width: col.width
                host: box
            }
            VoiceCard {
                id: cardPage
                visible: cardPage.shows
                width: col.width
                host: box
            }
            VoiceKeys {
                id: keyPage
                visible: keyPage.shows
                width: col.width
                host: box
            }
            VoiceRoom {
                width: col.width
                host: box
            }
            VoiceFoot {
                id: foot
                width: col.width
                host: box
            }
        }
        PressWatch {
            onTouched: box.touched()
        }
        // Hidden lines laid out as the rows' are, which round each line up, so a row always fits them.
        Label {
            id: latinName
            visible: false
            text: " "
            font.family: box.res.fam.latin
            font.pixelSize: box.pn.row_text_px[0] * box.s
        }
        Label {
            id: arabicName
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
            font.pixelSize: box.pn.hint_px[0] * box.s
        }
        Label {
            id: arabicMeta
            visible: false
            text: " "
            font.family: box.res.fam.arabic
            font.pixelSize: box.pn.hint_px[0] * box.s * box.lk.look.arabic_scale
        }
    }

    PopIn {
        id: popIn
        target: body
        move: box.lk.motion.moves.panel
        amount: box.lk.motion.amount
    }
}
