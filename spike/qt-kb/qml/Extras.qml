// Stage-1 extras: the Copy pill, the voice caption and the snip overlay, with the tools and voice worker behind them,
// and gate G12's ring test.
// They take the keyboard's theme, light or dark, like the mock-up's .selpill, .capbar and .snip-*.
import QtQuick
import KeyXtend.Spike

Item {
    id: ex

    // Qt units for a physical point (the keyboard's `logical`), the pointer as `{x, y}` or `{note}`, and the look and fonts.
    required property var logical
    required property var cursor
    required property var lk
    required property var res

    // The mic is recording.
    readonly property bool recording: vc.recording
    // The ring test's switch label, and whether the test runs.
    readonly property string ringLabel: ring.label
    readonly property bool ringOn: ring.visible

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

    // Starts or stops the ring test, lifting its window above ours.
    function flipRing() {
        ring.toggle();
        if (ring.visible)
            guard(ring.title);
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
            // The shadow owns the pill, so it is shown first.
            pillGlow.visible = true;
            pill.visible = true;
            say(tl.guard(tl.pillTitle));
        }
        if (r.hide) {
            pill.visible = false;
            pillGlow.visible = false;
        }
    }

    Glow {
        id: pillGlow
        lk: ex.lk
        box: Qt.rect(pill.x, pill.y, pill.width, pill.height)
        radius: pill.height / 2
        follow: pill.popped

        Pill {
            id: pill
            title: tl.pillTitle
            least: tl.pillWidth
            height: tl.pillHeight
            label: tl.pillLabel
            icon: tl.pillIcon
            lk: ex.lk
            res: ex.res
            onCopyClicked: ex.say(tl.copy())
        }
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
    Overlay {
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
    Ring {
        id: ring
        logical: ex.logical
        cursor: ex.cursor
        lk: ex.lk
        onNote: text => ex.say(text)
    }
    Timer {
        interval: tl.pollMs
        running: true
        repeat: true
        onTriggered: ex.tick()
    }
}
