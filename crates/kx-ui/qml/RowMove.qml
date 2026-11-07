// A list row's moves, as the clipboard's and Quick-fill's rows play them: a page turn slides the row in from the side
// it came, and a new row in its place swaps in. It plays the Rise the row's parts are in.
import QtQuick

QtObject {
    id: mv

    // The Rise to play, the moves of a page turn and a swap, the look and size.
    required property Rise rise
    required property var turn
    required property var swap
    required property var lk
    required property real s

    // Who the slot showed last (empty for none), and on which page.
    property string lastKey: ""
    property int lastAt: -1

    // The slot now shows `key` (empty for none) on page `at`: plays what changed since the last view, and is true when
    // the same row stays in place, so the row may play its own.
    function moved(key, at) {
        const was = { key: lastKey, at: lastAt };
        start(key, at);
        if (key === "" || was.at < 0)
            return false;
        if (at !== was.at)
            slide(turn, at > was.at ? 1 : -1);
        else if (key !== was.key)
            slide(swap, 1);
        return at === was.at && key === was.key;
    }
    // Takes `key` on page `at` as shown, playing nothing, as on the slot's first view.
    function start(key, at) {
        lastKey = key;
        lastAt = at;
    }
    // Slides the parts in with `how`, from `by` times the pop-in's rise below their place, or above when negative.
    function slide(how, by) {
        rise.move = how;
        rise.from = by * lk.motion.amount.pop_rise_px * s;
        rise.play();
    }
}
