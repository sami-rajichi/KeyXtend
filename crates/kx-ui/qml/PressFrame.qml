// When a pressed key acts against the frame that shows it pressed (`press_frame`): at once, drawn after ("after"), once
// that frame is out ("first"), or once it is drawn at once ("now"). As a timer it ends the "first" wait.
import QtQuick

Timer {
    id: presses

    // The keyboard window, whose keys act and whose frames it watches.
    required property Window win
    // The press waiting for its frame, as a function, or null.
    property var waiting: null
    readonly property string mode: win.cfg ? win.cfg.pressFrame : "after"
    // The frame log marks each press's action (`[ui] frame_log`).
    readonly property bool logs: win.cfg ? win.cfg.frameLog : false

    // Once the frame showing the press is out, the press acts, after Qt's frame ends.
    property Connections swaps: Connections {
        target: presses.win
        enabled: presses.waiting !== null && presses.mode === "first"
        function onFrameSwapped() {
            Qt.callLater(presses.settle);
        }
    }

    interval: win.cfg ? win.cfg.pressFrameMs : 0
    onTriggered: settle()

    // A click on key `id`; `back` is the language key's back part.
    function tap(id, back) {
        run(() => win.tap(id, back));
    }
    // A repeating key `id` went down.
    function hold(id) {
        run(() => win.hold.press(id));
    }
    // Key `id` came up: a press still waiting acts first, so the let-go always follows it.
    function letGo(id) {
        settle();
        win.hold.letGo(id);
    }
    // Qt took the press away: a press still waiting acts first.
    function leave() {
        settle();
        win.hold.leave();
    }
    // Runs press `act` at once, or once its frame is out.
    function run(act) {
        settle();
        if (mode === "after") {
            timed(act);
            return;
        }
        waiting = act;
        if (mode === "now") {
            Qt.callLater(draw);
            return;
        }
        restart();
        win.update();
    }
    // Draws the waiting press's frame at once, then it acts.
    function draw() {
        if (waiting)
            win.keyboard.drawNow(win.title);
        settle();
    }
    // A waiting press acts now, unless the window hid meanwhile.
    function settle() {
        stop();
        const act = waiting;
        waiting = null;
        if (act && win.visible)
            timed(act);
    }
    // Runs press `act`, between the frame log's marks while it logs.
    function timed(act) {
        if (logs)
            win.keyboard.pressStart();
        act();
        if (logs)
            win.keyboard.pressEnd();
    }
}
