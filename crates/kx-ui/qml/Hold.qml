// The held key's beat (spec §4.6): it runs only while a repeating key is held, waits the delay, then keeps the pace,
// and only asks the keyboard whether a repeat is due. A hidden or stopped keyboard ends the hold.
import QtQuick

Timer {
    id: hold

    // The keyboard window, whose keyboard it calls and whose state it redraws.
    required property Window win
    // The held key's id, or empty.
    property string held: ""
    // The delay has passed, so the beat keeps the pace.
    property bool paced: false
    // A repeat of this press went out, so sliding off ends the press.
    property bool repeated: false
    // The key the pointer slid off before any repeat; coming back onto it while still down holds it again.
    property string lost: ""
    // The keyboard shows and runs; a hold ends when it stops.
    readonly property bool live: win.ready && win.visible

    interval: win.ready ? (paced ? win.board.repeat_every_ms : win.board.repeat_delay_ms) : 0
    running: held !== "" && live
    repeat: true
    onTriggered: {
        paced = true;
        const answer = win.keyboard.tick();
        repeated = repeated || !!JSON.parse(answer).repeated;
        win.redraw(answer);
    }
    onLiveChanged: if (!live) leave()

    // Key `id` went down: it holds, unless the open list takes it, which then only clicks. The beat starts after
    // the keyboard timed the press, so its first tick is never early.
    function press(id) {
        win.hint("", null);
        lost = "";
        if (win.listing(id))
            return;
        win.redraw(win.keyboard.press(id));
        paced = false;
        repeated = false;
        held = id;
    }
    // The pointer came back onto key `id` while still down: a key it slid off before any repeat holds again.
    function reenter(id) {
        if (lost === id)
            press(id);
    }
    // Key `id` came up over itself: the held key's let-go, or the open list's key.
    function letGo(id) {
        const was = held === id;
        held = "";
        lost = "";
        if (was)
            win.redraw(win.keyboard.letGo(id));
        else if (win.listing(id))
            win.tap(id, false);
    }
    // The pointer slid off the held key: before any repeat it may come back, after one the press ends.
    function slide() {
        const back = repeated ? "" : held;
        leave();
        lost = back;
    }
    // The keyboard hid or stopped, or the pointer slid off: the hold ends with nothing typed.
    function leave() {
        lost = "";
        if (held === "")
            return;
        held = "";
        win.redraw(win.keyboard.leave());
    }
}
