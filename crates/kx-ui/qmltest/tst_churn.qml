// The debug churn steps the keyboard window as its buttons do: each theme switch, each Caps Lock change, then sizes
// up and back down, in that order, and tells the keyboard once when it ends.
import QtQuick
import QtTest
import "../qml"

Item {
    id: root

    // What the churn did, in order: `t` a theme, `c` Caps Lock, `+` and `-` a size step; and how often it ended.
    property string did: ""
    property int ended: 0
    // A short plan, so the test runs quickly.
    readonly property var plan: ({ step_ms: 1, themes: 2, caps: 3, sizes: 4 })

    // Stands in for the keyboard window: each call notes itself.
    QtObject {
        id: win
        readonly property var keyboard: QtObject {
            function flipCaps() {
                root.did += "c";
                return "{}";
            }
            function bigger() {
                root.did += "+";
                return "{}";
            }
            function smaller() {
                root.did += "-";
                return "{}";
            }
            function churned() {
                root.ended++;
            }
        }
        function restyle() {
            root.did += "t";
        }
        function redraw(json) {
        }
    }
    Component {
        id: churnC
        Churn {}
    }

    TestCase {
        name: "churn"

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
            root.did = "";
            root.ended = 0;
        }

        function test_the_churn_steps_in_order_and_ends_once() {
            const c = createTemporaryObject(churnC, root, { win: win, plan: root.plan });
            tryCompare(root, "ended", 1, 1000);
            compare(root.did, "ttccc++--");
            compare(c.step, c.steps);
            verify(!c.data.find(o => o.interval !== undefined).running, "its timer stops");
        }
    }
}
