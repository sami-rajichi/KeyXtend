// Stage-1 extras: the voice caption and the snip overlay, with the tools and voice worker behind them.
// They take the keyboard's theme, light or dark, like the mock-up's .capbar and .snip-*; the pill left in P10.
import QtQuick
import KeyXtend.Spike

Item {
    id: ex

    // Qt units for a physical point (the keyboard's `logical`), and the look and fonts.
    required property var logical
    required property var lk
    required property var res

    // The mic is recording.
    readonly property bool recording: vc.recording

    // A note for the status line.
    signal note(string text)

    Tools { id: tl }
    Voice {
        id: vc
        onCaption: json => ex.caption(json)
    }

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
        if (r.mic)
            vc.click();
    }

    // Shows a caption from the voice worker on top of every window, and any note it carries; a note may come alone.
    function caption(json) {
        const r = JSON.parse(json);
        say(r.note);
        if (r.text === undefined)
            return;
        const shown = cap.visible;
        cap.say(r, logical(r.at[0], r.at[1]));
        if (!shown)
            say(tl.guard(cap.title));
    }

    Caption {
        id: cap
        title: vc.barTitle
        barWidth: vc.barWidth
        barHeight: vc.barHeight
        busyIcon: vc.busyIcon
        wordsIcon: vc.wordsIcon
        lk: ex.lk
        res: ex.res
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
