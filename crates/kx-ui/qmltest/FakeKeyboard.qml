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
        return JSON.stringify({ board: false, state: false });
    }
    function placed(x, y, w, h) {
        return note("placed");
    }
    function cursor() {
        return "null";
    }
    function bubbleAt(x, y, corner) {
        note("bubbleAt " + corner);
        return "null";
    }
    function shape(title, boxes, dpr) {
        note("shape " + title);
        return true;
    }
    function guard() {
        note("guard");
        return false;
    }
}
