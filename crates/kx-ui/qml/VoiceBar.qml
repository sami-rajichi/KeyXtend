// The dictation bar (design "The bar"; design/voice-bar.css .vbar): a pill in the keyboard's theme where the hub
// places it, one key row tall, taller by a card's warning or a note's acts under the row. It listens, writes, then
// shows the text or a note with each state's buttons in one order in every language (ADR-0020), taking clicks but
// never focus, playing the start and stop sounds and telling the module when the pointer is over it.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: bar

    // The hub's Voice object, the look, fonts, icons and size, whether the keyboard speaks Arabic, the keyboard's box
    // (`box()`, JSON in physical pixels, "null" while hidden), Qt units for a physical spot, and the clock in ms.
    required property var voice
    required property var lk
    required property var res
    required property real s
    required property bool ar
    required property var box
    required property var logical
    property var now: () => Date.now()
    // Hands a line to screen readers; a test puts its own in place to hear what is told.
    property var announce: said => body.Accessible.announce(said)

    // The view as the hub writes it (null before any), the pointer over the bar as last told, the round Copy was
    // clicked on, and when the listening view came on the clock.
    property var view: null
    property bool over: false
    property int copiedAt: -1
    property real heardAt: 0

    // The bar shows (true) or hides (false); its shadow follows.
    signal showing(bool on)
    // A press landed on the bar, which closes the tray menu; the press still reaches the button.
    signal touched

    // The box whose pop the shadow follows, and the wave, for the app and the tests.
    readonly property alias body: body
    readonly property VoiceWave wave: middle.wave
    readonly property VoiceNames names: VoiceNames {}
    readonly property var shown: view ? view.bar : null
    readonly property string mode: shown ? shown.state : names.states.hidden
    readonly property bool listens: mode === names.states.listening
    readonly property bool writes: mode === names.states.transcribing
    readonly property bool wide: mode === names.states.text || mode === names.states.note
    // The card's warning while the first dictation to its service listens, which widens the bar and sits under its row.
    readonly property string warning: listens ? field("warn", "") : ""
    readonly property bool warns: warning !== ""
    readonly property var cap: lk.shape.caption
    readonly property var p: lk.palette
    readonly property real gap: cap.gap_px * s
    readonly property real pad: cap.pad_px * s
    readonly property var buttons: shown && shown.buttons ? shown.buttons : []
    // Cancel leads while it listens or writes; the rest end the bar, the gear before the last.
    readonly property var front: listens || writes ? buttons.slice(0, 1) : []
    readonly property var back: gearIn(listens || writes ? buttons.slice(1) : buttons)
    // A note the service's own pages settle: its acts under the line, Open service last (design/voice-cloud.js .cna).
    readonly property var acts: shown && shown.acts ? shown.acts : []

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: view ? view.words.window : ""
    color: "transparent"
    visible: false
    // Whole units up, as a window's size is whole; a warning or a note's acts add their height under the row.
    width: rescale.keep(Math.ceil(span))
    // The text's width for a text, a note or a warning; else the narrow one, widened up to it for the wave's least room.
    readonly property real span: wide || warns ? broad : Math.min(broad, Math.max(narrow, broad - leadRoom + lead.width))
    readonly property real narrow: lk.shape.voice.bar_px[0] * s
    readonly property real broad: lk.shape.voice.bar_px[1] * s
    // The most the lead may take for the wave's least room within the text's width; a model's name on the chip fits it.
    readonly property real leadRoom: broad - tail.width - lk.shape.voice.wave_room_px * s - 2 * (pad + gap)
    height: rescale.keep(Math.ceil(row + under))
    // The row's height: one key row, or its parts with their padding.
    readonly property real row: Math.max(cap.bar_px * s, Math.max(lead.height, middle.need, tail.height) + 2 * pad)
    // What sits under the row: the warning's lines, or a note's acts with the padding under them.
    readonly property real under: warns ? warnLine.height : acts.length > 0 ? actRow.height + pad : 0
    // How far the row's parts rise from the middle, so what sits under them fits.
    readonly property real lift: under / 2

    // Keeps the size above across display scale changes, and the place on a new scale.
    Rescale {
        id: rescale
        onRatioChanged: Qt.callLater(bar.follow)
    }

    Connections {
        target: bar.voice
        function onViewChanged() {
            bar.read();
        }
        function onLevelChanged() {
            wave.heard(bar.voice.level());
        }
    }
    Component.onCompleted: read()
    onWidthChanged: follow()
    onHeightChanged: follow()

    // Reads the hub's view: a new state pops in, plays its sound and is told to readers; the same one is placed again.
    function read() {
        if (!voice)
            return;
        const was = view;
        view = JSON.parse(voice.view());
        if (!view || mode === names.states.hidden)
            return shut();
        const fresh = !was || was.bar.state !== mode;
        if (listens && (fresh || was.round !== view.round))
            heardAt = now();
        if (listens && fresh)
            wave.reset();
        chime(was);
        if (fresh || !visible)
            open();
        else
            follow();
        if (fresh || hintOf(was) !== hintOf(view))
            tell();
    }
    // The start sound as it starts listening, the stop sound as it stops, unless the view has sounds off.
    function chime(was) {
        const before = !!was && was.bar.state === names.states.listening;
        if (was && view.sounds && before !== listens)
            voice.chime(listens);
    }
    function open() {
        place();
        pop.stop();
        body.opacity = 1;
        body.y = 0;
        body.scale = 1;
        if (!visible) {
            showing(true);
            visible = true;
        }
        if (lk.motion.moves.caption.ms > 0)
            pop.restart();
        point(hover.hovered);
    }
    // Places the shown bar again, as its size, its place or the keyboard changed.
    function follow() {
        if (visible)
            place();
    }
    // Asks the hub where the bar goes by the keyboard for its size in Qt units.
    function place() {
        const at = JSON.parse(voice.barAt(box(), width, height));
        if (!at)
            return;
        const q = logical(at.x, at.y);
        x = Math.round(q.x);
        y = Math.round(q.y);
    }
    // Hides the bar; the hub tells the module the pointer left, so the bar only forgets it.
    function shut() {
        pop.stop();
        over = false;
        if (!visible)
            return;
        visible = false;
        showing(false);
    }
    // The pointer came over the bar (`on`) or left it: the module hears each change once.
    function point(on) {
        if (!visible || !view || on === over)
            return;
        over = on;
        voice.act(names.acts.over, "", on ? 1 : 0, view.round);
    }
    // A button's click, on the view it was clicked on; Copy shows Copied until the next view.
    function press(b) {
        if (b.kind === names.acts.bar && b.id === names.copy)
            copiedAt = view.round;
        voice.act(b.kind, b.id, b.arg, view.round);
    }
    // `list` with the gear before its last button.
    function gearIn(list) {
        if (!view || list.length === 0)
            return list;
        const gear = { kind: names.acts.gear, id: names.gear, arg: 0, label: view.words.gear, on: true, tone: names.plain };
        return list.slice(0, -1).concat([gear], list.slice(-1));
    }
    // The shown state's `key`, or `none` when it has none.
    function field(key, none) {
        const v = shown ? shown[key] : undefined;
        return v === undefined || v === null ? none : v;
    }
    // The hint view `v` shows, by id or words, empty for none.
    function hintOf(v) {
        const h = v && v.bar.hint;
        return !h ? "" : (h.id || h);
    }
    // Screen readers hear each new state and line, since the bar never takes focus; Listening's sentence is the hub's.
    function tell() {
        const said = spoken();
        if (said)
            announce(said);
    }
    // What readers hear of the shown bar: Listening's sentence, else its line, else its word or its text.
    function spoken() {
        const h = shown.hint;
        const line = !h ? "" : (h.text || h);
        return listens ? shown.said : writes ? (line || shown.word) : (line || (shown.text ? shown.text.text : ""));
    }

    Rectangle {
        id: body
        width: bar.width
        height: bar.height
        radius: bar.wide || bar.warns ? bar.cap.words_radius_px * bar.s : height / 2
        color: bar.p.pop_bg
        border.width: bar.lk.contrast ? bar.lk.shape.line.px : 0
        border.color: bar.p.pop_line
        Accessible.role: Accessible.ToolBar
        Accessible.name: bar.view ? bar.view.words.title : ""

        HoverHandler {
            id: hover
            onHoveredChanged: bar.point(hovered)
        }
        // Cancel, then the status and the language while it listens or writes, or the mark before a text or a note.
        Row {
            id: lead
            x: bar.pad
            anchors.verticalCenter: parent.verticalCenter
            anchors.verticalCenterOffset: -bar.lift
            spacing: bar.gap
            layoutDirection: Qt.LeftToRight
            Repeater {
                model: bar.front
                delegate: VoiceBarButton {
                    host: bar
                }
            }
            VoiceStatus {
                visible: bar.listens || bar.writes
                anchors.verticalCenter: parent.verticalCenter
                host: bar
            }
            VoiceSlot {
                visible: bar.wide
                anchors.verticalCenter: parent.verticalCenter
                host: bar
            }
        }
        VoiceMiddle {
            id: middle
            x: lead.x + lead.width + bar.gap
            width: Math.max(0, tail.x - bar.gap - x)
            anchors.verticalCenter: parent.verticalCenter
            anchors.verticalCenterOffset: -bar.lift
            host: bar
        }
        // The timer while it listens, then the buttons, the gear before the last.
        Row {
            id: tail
            x: bar.width - bar.pad - width
            anchors.verticalCenter: parent.verticalCenter
            anchors.verticalCenterOffset: -bar.lift
            spacing: bar.gap
            layoutDirection: Qt.LeftToRight
            VoiceTimer {
                visible: bar.listens
                anchors.verticalCenter: parent.verticalCenter
                lk: bar.lk
                res: bar.res
                s: bar.s
                format: bar.view ? bar.view.words.timer : ""
                from: bar.field("elapsedMs", 0)
                at: bar.heardAt
                now: bar.now
            }
            Repeater {
                model: bar.back
                delegate: VoiceBarButton {
                    host: bar
                }
            }
        }
        VoiceWarn {
            id: warnLine
            y: body.height - height
            host: bar
        }
        VoiceActs {
            id: actRow
            host: bar
        }
        PressWatch {
            onTouched: bar.touched()
        }
    }

    // Each new state pops in (mock-up .vbar.in, @keyframes rise).
    PopIn {
        id: pop
        target: body
        move: bar.lk.motion.moves.caption
        amount: bar.lk.motion.amount
    }
}
