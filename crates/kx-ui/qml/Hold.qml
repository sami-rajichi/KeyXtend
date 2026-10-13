// The held key's beat (spec §4.6): it runs only while the keyboard holds a key, waits the delay, then keeps the pace,
// and only asks whether a repeat is due. A let-go, Qt's cancel, the list showing, or a hidden or stopped keyboard ends
// it, and so does a reply that the keyboard no longer holds the key.
import QtQuick

Timer {
    id: hold

    // The keyboard window, whose keyboard it calls and whose state it redraws.
    required property Window win
    // The held key's id, or empty.
    property string held: ""
    // The delay has passed, so the beat keeps the pace.
    property bool paced: false
    // The keyboard shows and runs with no list open; a hold ends when that stops.
    readonly property bool live: win.ready && win.visible && !(win.st && win.st.list)

    interval: win.ready ? (paced ? win.board.repeat_every_ms : win.board.repeat_delay_ms) : 0
    running: held !== "" && live
    repeat: true
    onTriggered: {
        paced = true;
        if (!win.redraw(win.keyboard.tick()).holding)
            held = "";
    }
    onLiveChanged: if (!live) leave()

    // Key `id` went down: it types now and holds while the keyboard says so, unless the open list takes it, which it
    // then drives at once. The beat starts after the keyboard timed the press, so its first tick is never early.
    function press(id) {
        win.hint("", null);
        if (win.listing(id)) {
            win.tap(id, false);
            return;
        }
        const r = win.redraw(win.keyboard.press(id));
        paced = false;
        held = r.holding ? id : "";
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
