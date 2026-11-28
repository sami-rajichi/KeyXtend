// Snip's windows over the Snip object (design "What the user sees"): a window per frozen screen, the capture bar and its
// hint, the strip, the countdown, the red edge and the record bar, then the editor, the close question and the Snip
// panel. It reads the view at each change, sends the theme's sizes as a snip starts and as each editor opens, and shows
// the windows over the frozen screens once those are up, which then go under every other window of ours.
pragma ComponentBehavior: Bound
import QtQuick

QtObject {
    id: snips

    // The hub's Snip object (or the tests' stand-in, or null for none), the look and texts, and what aims the keyboard's
    // letters at Snip: every press on a window with a line calls it, never the countdown's or the record bar's.
    required property QtObject snip
    required property var lk
    required property var res
    property var aim: () => {}
    // How long the pointer rests on a key before its tip shows, in ms (settings).
    property int tipMs: 0
    // The screens, each with its corner in physical pixels and size in Qt units; tests give scaled ones.
    property var screens: Qt.application.screens
    // The keyboard window, whose pointer, poll time, state, size and spots the editor and panel use; null shows neither.
    property var win: null
    // The keyboard is aside and Hidden, so the Snip panel hides too (Aside.qml).
    property bool hush: false

    // The view as last read (`Snip.view`), null before any; the stage whose windows show, set once the frozen screens
    // are up so they show above them; and the pointer in physical pixels, which every frozen screen follows.
    property var view: null
    property string shown: ""
    property point pointer: Qt.point(NaN, NaN)
    // The display scale of the screen the part was taken on, which the editor's steps take; and the sizes last sent.
    property real taken: 1
    property var sent: null
    // For the debug log's open times, on the wall clock in ms: the last click on Snip's windows, and when the last new
    // stage came with the ms from a click since the stage before it (-1 for none).
    property real clicked: 0
    property var came: ({ at: 0, click: -1 })

    readonly property int round: view ? view.round : 0
    readonly property string stage: view ? view.stage : names.stages.idle
    // The names the views and clicks use.
    readonly property SnipNames names: SnipNames {}
    // The system's Save or folder dialog sits in front, so Snip's windows leave the top.
    readonly property bool low: !!view && view.low
    // The frozen screens' title, which the hub lays them under our other windows by (ADR-0059).
    readonly property string screenTitle: lk ? res.tr("window.snip-screen") : ""
    readonly property var sh: lk ? lk.shape.snip : null
    // The keyboard's letters go to Snip's windows, so the text being typed draws its caret.
    readonly property bool aimed: !!win && !!win.st && win.st.aim === names.aim
    // The dark behind Snip's white words (size tag, hint, countdown); opaque in high contrast (drawn/snip.rs).
    readonly property color tag: lk && lk.contrast ? Qt.rgba(tagged.r, tagged.g, tagged.b, 1) : tagged
    readonly property color tagged: lk ? lk.common.snip_tag : "transparent"
    // A click on key `id` of `kind` on this view, with a step's sign or a caret in `arg`.
    function act(kind, id, arg) {
        clicked = Date.now();
        snip.act(kind, id, arg || 0, round);
    }
    // Stage `st`'s window showed, first drew and ended its pop at those wall-clock times: the hub logs them at debug in
    // whole ms from the view that brought the stage, with the click before it.
    function timed(st, shownAt, drawnAt, readyAt) {
        const ms = t => Math.round(t - came.at);
        snip.opened(st, came.click, ms(shownAt), ms(drawnAt), ms(readyAt));
    }
    // Whether the hub keeps capture window `id` hidden, as its shield was refused during a countdown or recording.
    function hidden(id) {
        return !!view && view.hide.indexOf(id) >= 0;
    }
    // Whether capture window `id` of stage `st` shows: its stage is up and its shield held.
    function up(st, id) {
        return stage === st && shown === st && !hidden(id);
    }
    // Key `cap`'s icon from its window's `map`: a sound switch shows its own while off.
    function iconOf(cap, map) {
        const ic = lk.icons.snip;
        if (names.switches.indexOf(cap.key) >= 0 && !cap.on)
            return cap.key === names.mic ? ic.mic_off : ic.sound_off;
        return map[cap.key] || "";
    }
    // Physical box `r` (`{x, y, w, h}`) in Qt units, by the scale of the screen holding its corner.
    function box(r) {
        const at = spots.logical(r.x, r.y);
        const sc = spots.screenOf(r.x, r.y, true);
        const d = sc ? sc.devicePixelRatio : 1;
        return Qt.rect(at.x, at.y, r.w / d, r.h / d);
    }
    // Reads the view; a snip that starts gets the theme's sizes first, and an editor that opens gets them at the scale
    // its part was taken at. The windows of a new stage show after the frozen screens it brings, or at once with none.
    function read() {
        if (!snip)
            return;
        const next = JSON.parse(snip.view());
        const starts = stage === names.stages.idle && !!next && next.stage !== names.stages.idle;
        const opens = !!next && next.stage === names.stages.edit && stage !== names.stages.edit;
        took(next);
        if (starts || opens)
            send(opens ? taken : 1);
        if (!!next && next.stage !== stage)
            stamp();
        drop(next);
        view = next;
        if (next && next.frozen.length > 0)
            Qt.callLater(show);
        else
            show();
    }
    // The frozen screens view `next` drops hide now: Qt frees their windows only once its loop runs again, which a busy
    // loop puts off.
    function drop(next) {
        for (let i = next ? next.frozen.length : 0; i < overlays.count; i++)
            overlays.objectAt(i).visible = false;
    }
    // A new stage came now, after any click since the stage before it.
    function stamp() {
        const now = Date.now();
        came = { at: now, click: clicked > came.at ? Math.round(now - clicked) : -1 };
    }
    // The new stage's windows show, then the frozen screens go under them and every other window of ours (ADR-0059).
    function show() {
        shown = stage;
        order();
    }
    function order() {
        if (snip)
            snip.order(screenTitle);
    }
    // A frozen screen moved or took a new scale: they go under ours again, once however many moved.
    function reorder() {
        Qt.callLater(order);
    }
    // Keeps the scale of the screen holding the centre of view `v`'s part, while a strip or recording shows one.
    function took(v) {
        const t = v ? v.strip || v.record : null;
        if (!t)
            return;
        const sc = spots.screenOf(t.part.x + t.part.w / 2, t.part.y + t.part.h / 2, true);
        taken = sc ? sc.devicePixelRatio : 1;
    }
    // Sends the theme's sizes with the editor's steps at display scale `scale`.
    function send(scale) {
        sent = fit.sizes(scale);
        snip.fit(JSON.stringify(sent));
    }

    // Qt units for physical pixels and back, on the screens above.
    property Spots spots: Spots {
        win: null
        screens: snips.screens
    }
    property SnipFit fit: SnipFit {
        lk: snips.lk
        res: snips.res
    }
    property Connections woke: Connections {
        target: snips.snip
        function onViewChanged() {
            snips.read();
        }
    }
    // One window per frozen screen, made while the view holds them and freed with their pictures.
    property Instantiator overlays: Instantiator {
        active: !!snips.lk
        model: snips.view ? snips.view.frozen.length : 0
        delegate: SnipOverlay {
            required property int index
            snips: snips
            at: index
        }
    }
    property SnipBar bar: SnipBar {
        snips: snips
    }
    property SnipHint hint: SnipHint {
        snips: snips
    }
    property SnipStrip strip: SnipStrip {
        snips: snips
    }
    property SnipCount count: SnipCount {
        snips: snips
    }
    property SnipEdge edge: SnipEdge {
        snips: snips
    }
    property SnipRecord record: SnipRecord {
        snips: snips
    }
    // The editor, question and panel stack in declaration order, as each makes its windows when it completes.
    property SnipEditor editor: SnipEditor {
        snips: snips
    }
    property SnipAsk ask: SnipAsk {
        snips: snips
    }
    property SnipPanel panel: SnipPanel {
        snips: snips
    }

    Component.onCompleted: read()
    // A new language retitles the frozen screens, and the hub lays them low by the new title.
    onScreenTitleChanged: order()
}
