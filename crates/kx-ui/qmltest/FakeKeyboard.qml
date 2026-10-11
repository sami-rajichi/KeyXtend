// Stands in for the hub's Keyboard object: it answers with the face's golden board and state and records each call.
import QtQuick
import "fixtures/face.js" as Face

QtObject {
    // The board (null while the keyboard is not running), the state, and what each action asks to redraw.
    property var board: Face.board
    property var state: Face.states.en
    property var reply: ({ board: false, state: false })
    // Every call so far, as `name` or `name args`.
    property var calls: []
    // Where `bubbleAt` puts the bubble, in physical pixels, or null for nowhere, and the point it was last asked from.
    property var bubble: null
    property var bubbleFrom: null
    // How many more guard tries answer "try again", as when a window has not shown yet.
    property int again: 0
    // Where the pointer is, in physical pixels, or null when Windows cannot say.
    property var pointer: null
    // The last place the window sent: its box in physical pixels and the corner's overhang at size 1.
    property var placedAt: null
    // The place sent becomes the state's, as the keyboard saves a move, though it answers that nothing changed.
    property bool keeps: false
    // The scale the window last cut itself at.
    property var shapeDpr: null

    // A window titled `title` asked to be lifted alone, as the call runs.
    signal lifted(string title)
    // A guard try runs, as the call starts.
    signal guarding

    function note(call) {
        calls = calls.concat([call]);
        return JSON.stringify(reply);
    }
    function boardJson() {
        return JSON.stringify(board);
    }
    function stateJson() {
        return JSON.stringify(state);
    }
    function tap(id, back) {
        return note("tap " + id + " " + back);
    }
    function press(id) {
        return note("press " + id);
    }
    function tick() {
        return note("tick");
    }
    function letGo(id) {
        return note("letGo " + id);
    }
    function leave() {
        return note("leave");
    }
    function smaller() {
        return note("smaller");
    }
    function bigger() {
        return note("bigger");
    }
    function preset(index) {
        return note("preset " + index);
    }
    function resizeBegin() {
        return note("resizeBegin");
    }
    function resizeAt() {
        return note("resizeAt");
    }
    function resizeEnd() {
        return note("resizeEnd");
    }
    function langPick(index) {
        return note("langPick " + index);
    }
    function panelKey(id) {
        return note("panelKey " + id);
    }
    function relabel() {
        note("relabel");
        return JSON.stringify({ board: false, state: false });
    }
    function placed(x, y, w, h, overhang) {
        placedAt = { x: x, y: y, w: w, h: h, overhang: overhang };
        if (keeps)
            state = Object.assign({}, state, { at: { x: x, y: y } });
        return note("placed");
    }
    function cursor() {
        return JSON.stringify(pointer);
    }
    function bubbleAt(x, y, corner) {
        bubbleFrom = { x: x, y: y };
        note("bubbleAt " + corner);
        return JSON.stringify(bubble);
    }
    function shape(title, boxes, dpr) {
        shapeDpr = dpr;
        note("shape " + title);
        return true;
    }
    function liftAlone(title) {
        note("liftAlone " + title);
        lifted(title);
        return true;
    }
    function guard(top) {
        guarding();
        note(top ? "guard " + top : "guard");
        if (again <= 0)
            return false;
        again--;
        return true;
    }
}
