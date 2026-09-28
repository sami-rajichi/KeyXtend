// Stage-1 extras: the Copy pill, the voice caption and the snip overlay, with the tools and voice worker behind them.
import QtQuick
import KeyXtend.Spike

Item {
    id: ex

    // Qt units for a physical point (the keyboard's `logical`), and the text size and padding of the small windows.
    required property var logical
    required property real fontPx
    required property real pad

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

    // Shows, moves or hides the pill, and shows any new fill note.
    function tick() {
        const r = JSON.parse(tl.tick());
        say(r.note);
        const at = r.show || r.move;
        if (at)
            pill.place(logical(at[0], at[1]));
        if (r.show) {
            pill.visible = true;
            say(tl.guard(tl.pillTitle));
        }
        if (r.hide)
            pill.visible = false;
    }

    Pill {
        id: pill
        title: tl.pillTitle
        width: tl.pillWidth
        height: tl.pillHeight
        label: tl.pillLabel
        fontPx: ex.fontPx
        onCopyClicked: ex.say(tl.copy())
    }
    Caption {
        id: cap
        title: vc.barTitle
        width: vc.barWidth
        height: vc.barHeight
        fontPx: ex.fontPx
        pad: ex.pad
    }
    Overlay {
        id: over
        title: tl.overlayTitle
        edge: tl.snipEdge
        hint: tl.snipHint
        onPicked: {
            const n = tl.pick();
            if (n.length > 0) {
                over.end();
                ex.say(n);
            }
        }
    }
    Timer {
        interval: tl.pollMs
        running: true
        repeat: true
        onTriggered: ex.tick()
    }
}
