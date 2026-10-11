// Debug builds only: steps the keyboard window through a churn plan as its buttons do, for a memory check: theme
// switches, Caps Lock changes with no key sent, then sizes up and back down; it tells the keyboard when it ends.
import QtQuick

Item {
    id: churn

    // The keyboard window (KbWindow), and the plan: ms between steps and how many of each change.
    required property var win
    required property var plan
    // Steps taken so far.
    property int step: 0
    readonly property int steps: plan.themes + plan.caps + plan.sizes

    // Step `i`: a theme, then Caps Lock, then the first half of the sizes up and the rest down.
    function act(i) {
        const caps = i - plan.themes;
        const size = caps - plan.caps;
        if (caps < 0)
            win.restyle();
        else if (size < 0)
            win.redraw(win.keyboard.flipCaps());
        else
            win.redraw(size < plan.sizes / 2 ? win.keyboard.bigger() : win.keyboard.smaller());
    }

    Timer {
        interval: churn.plan.step_ms
        running: churn.step < churn.steps
        repeat: true
        onTriggered: {
            churn.act(churn.step);
            churn.step++;
            if (churn.step === churn.steps)
                churn.win.keyboard.churned();
        }
    }
}
