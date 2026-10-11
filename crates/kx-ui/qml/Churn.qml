// Debug builds only: steps the app through a churn plan as its buttons do, for a memory check, and tells the keyboard
// when it ends. Each round shows a key's tip, opens and closes the List design's language list, minimises to the
// bubble and back, then switches through the themes; Caps Lock changes with no key sent and sizes up and down follow.
import QtQuick

Item {
    id: churn

    // The app (App), whose keyboard window and bubble it steps, and the plan: ms between steps, rounds, theme switches
    // per round, Caps Lock changes and size changes.
    required property var app
    required property var plan
    readonly property var win: app.face
    // Steps taken so far; a round's pop-up steps, in order, before its theme switches.
    property int step: 0
    readonly property var pops: [churn.tip, churn.untip, churn.list, churn.list, churn.minimise, churn.back]
    readonly property int roundSteps: pops.length + plan.themes
    readonly property int steps: plan.rounds * roundSteps + plan.caps + plan.sizes

    // Step `i`: a round's pop-up or theme switch, then Caps Lock, then the first half of the sizes up and the rest down.
    function act(i) {
        const caps = i - plan.rounds * roundSteps;
        const size = caps - plan.caps;
        if (caps < 0 && i % roundSteps < pops.length)
            pops[i % roundSteps]();
        else if (caps < 0)
            win.restyle();
        else if (size < 0)
            win.redraw(win.keyboard.flipCaps());
        else
            win.redraw(size < plan.sizes / 2 ? win.keyboard.bigger() : win.keyboard.smaller());
    }
    // The wait after step `i`: a tip step waits out the hover time too, so the tip shows before it goes.
    function after(i) {
        const tipping = i < plan.rounds * roundSteps && pops[i % roundSteps] === churn.tip;
        return plan.step_ms + (tipping ? win.cfg.tipMs : 0);
    }

    // A key with an icon and no note shows its name, as when the pointer rests on it, and takes it back as it leaves.
    function tip() {
        const k = tipKey();
        if (k)
            k.tip(k.name, k);
    }
    function untip() {
        const k = tipKey();
        if (k)
            k.tip("", k);
    }
    function tipKey() {
        return key(k => k.icon && !k.note);
    }
    // The language key opens or closes the List design's list; in another design it would switch the layout, so it
    // stays still.
    function list() {
        const k = win.board.lang_key === "list" ? key(k => k.kv.kind === "lang") : null;
        if (k)
            k.hit(false);
    }
    function minimise() {
        win.minimise();
    }
    function back() {
        app.bubble.clicked();
    }
    // The window's first key that `ok` takes, or null; a key is the item with a board cell, and nothing inside it is.
    function key(ok, item) {
        const kids = (item || win.contentItem).children;
        for (let i = 0; i < kids.length; i++) {
            const c = kids[i];
            const found = c.kv !== undefined ? (ok(c) ? c : null) : key(ok, c);
            if (found)
                return found;
        }
        return null;
    }

    Timer {
        interval: churn.plan.step_ms
        running: churn.step < churn.steps
        repeat: true
        onTriggered: {
            churn.act(churn.step);
            interval = churn.after(churn.step);
            churn.step++;
            if (churn.step === churn.steps)
                churn.win.keyboard.churned();
        }
    }
}
