// The held key's beat (spec §4.6): it runs only while a repeating key is held, waits the delay, then keeps the pace,
// and only asks the keyboard whether a repeat is due. A let-go, Qt's cancel, or a hidden or stopped keyboard ends it.
import QtQuick

Timer {
    id: hold

    // The keyboard window, whose keyboard it calls and whose state it redraws.
    required property Window win
    // The held key's id, or empty.
    property string held: ""
    // The delay has passed, so the beat keeps the pace.
    property bool paced: false
    // The keyboard shows and runs; a hold ends when it stops.
    readonly property bool live: win.ready && win.visible

    interval: win.ready ? (paced ? win.board.repeat_every_ms : win.board.repeat_delay_ms) : 0
    running: held !== "" && live
    repeat: true
    onTriggered: {
        paced = true;
        win.redraw(win.keyboard.tick());
    }
    onLiveChanged: if (!live) leave()

    // Key `id` went down: it types now and holds, unless the open list takes it, which it then drives at once. The
    // beat starts after the keyboard timed the press, so its first tick is never early.
    function press(id) {
        win.hint("", null);
        if (win.listing(id)) {
            win.tap(id, false);
            return;
        }
        win.redraw(win.keyboard.press(id));
        paced = false;
        held = id;
    }
    // Key `id` came up, wherever the pointer is: its hold ends.
    function letGo(id) {
        if (held !== id)
            return;
        held = "";
        win.redraw(win.keyboard.letGo(id));
    }
    // Qt took the press away, or the keyboard hid or stopped: the hold ends with nothing more typed.
    function leave() {
        if (held === "")
            return;
        held = "";
        win.redraw(win.keyboard.leave());
    }
}
