// Stage-1 extras: the snip overlay, with the tools behind it; the pill left in P10 and the voice caption in P11.
// It takes the keyboard's theme, light or dark, like the mock-up's .snip-*.
import QtQuick
import KeyXtend.Spike

Item {
    id: ex

    // Qt units for a physical point (the keyboard's `logical`), and the look and fonts.
    required property var logical
    required property var lk
    required property var res

    // A note for the status line.
    signal note(string text)

    Tools { id: tl }

    // Passes a note on unless it is empty.
    function say(text) {
        if (text)
            note(text);
    }

    // Guards our windows and lifts the one called `title` to the top.
    function guard(title) {
        say(tl.guard(title));
    }

    // Runs side tool `i` and shows what it says.
    function tool(i) {
        const r = JSON.parse(tl.tool(i));
        say(r.note);
        if (r.snip) {
            over.begin(logical(r.snip.x, r.snip.y), r.snip);
            // The image loads at once, so the private copy on disk can go now.
            say(tl.forgetFrozen());
            say(tl.guard(tl.overlayTitle));
        }
    }

    SnipOverlay {
        id: over
        title: tl.overlayTitle
        lk: ex.lk
        res: ex.res
        edge: tl.snipEdge
        hint: tl.snipHint
        sizeText: tl.snipSize
        onPicked: {
            const n = tl.pick();
            if (n.length > 0) {
                over.end();
                ex.say(n);
            }
        }
    }
}
