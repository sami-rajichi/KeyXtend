// The debug churn steps the app as its buttons do: each round a key's tip, the List design's language list opened and
// closed, minimise to the bubble and back, then each theme switch; then each Caps Lock change, then sizes up and back
// down, in that order, and it tells the keyboard once when it ends; the app starts the churn its keyboard object plans.
import QtQuick
import QtTest
import "../qml"
import "kbwin.js" as Win
import "fixtures/face.js" as Face

Item {
    id: root

    // What the churn did, in order: `k` a key's tip, `u` that tip taken back, `l` the language key, `m` minimise, `b`
    // the bubble, `t` a theme, `c` Caps Lock, `+` and `-` a size step; how often it ended; and the churn's wait as the
    // tip was taken back, and as the language key was pressed after it.
    property string did: ""
    property int ended: 0
    property int tipWait: 0
    property int stepWait: 0
    // The stand-in window's language key design and hover time, and the churn made last.
    property string design: "list"
    readonly property int tipMs: 7
    property var made: null
    // A short plan, so the test runs quickly.
    readonly property var plan: ({ step_ms: 1, rounds: 2, themes: 2, caps: 3, sizes: 4 })

    // Stands in for the app: its keyboard window, with a key that has an icon and the language key, and its bubble.
    QtObject {
        id: app
        readonly property var face: win
        readonly property var bubble: QtObject {
            function clicked() {
                root.did += "b";
            }
        }
    }
    Item {
        id: win
        readonly property Item contentItem: plate
        readonly property var board: ({ lang_key: root.design })
        readonly property var cfg: ({ tipMs: root.tipMs })
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
        function minimise() {
            root.did += "m";
        }

        Item {
            id: plate
            // A key with a note, whose tip says more than its name, comes first and is passed over.
            Item {
                property var kv: ({ kind: "named" })
                property string icon: "noted.svg"
                property string note: "a note"
                signal tip(string name, Item at)
                onTip: root.did += "!"
            }
            Item {
                id: bksp
                property var kv: ({ kind: "named" })
                property string icon: "bksp.svg"
                property string note: ""
                property string name: "Backspace"
                signal tip(string name, Item at)
                onTip: (name, at) => {
                    if (name === bksp.name && at === bksp) {
                        root.did += "k";
                    } else if (name === "") {
                        root.did += "u";
                        root.tipWait = root.made.data.find(o => o.interval !== undefined).interval;
                    }
                }
            }
            Item {
                property var kv: ({ kind: "lang" })
                property string icon: ""
                property string note: ""
                signal hit(bool back)
                onHit: back => {
                    root.did += back ? "L" : "l";
                    root.stepWait = root.made.data.find(o => o.interval !== undefined).interval;
                }
            }
        }
    }
    Component {
        id: churnC
        Churn {}
    }
    // An app over stand-ins whose keyboard plans a churn, as a debug build with the churn's variable set does.
    Component {
        id: appC
        App {
            ui: FakeUi {}
            keyboard: FakeKeyboard {
                board: Object.assign({}, Face.board, { lang_key: "list" })
                state: Object.assign({}, Face.states.en, { at: { x: 100, y: 200 } })
                bubble: ({ x: 40, y: 50 })
                function churnPlan() {
                    return JSON.stringify({ step_ms: 1, rounds: 1, themes: 0, caps: 0, sizes: 2 });
                }
                function churned() {
                    root.ended++;
                }
            }
        }
    }

    TestCase {
        name: "churn"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
            root.did = "";
            root.ended = 0;
            root.tipWait = 0;
            root.stepWait = 0;
            root.made = null;
            root.design = "list";
        }
        function churned() {
            root.made = createTemporaryObject(churnC, root, { app: app, plan: root.plan });
            tryCompare(root, "ended", 1, 1000);
            return root.made;
        }

        function test_the_churn_steps_in_order_and_ends_once() {
            const c = churned();
            compare(root.did, "kullmbttkullmbttccc++--");
            compare(c.step, c.steps);
            verify(!c.data.find(o => o.interval !== undefined).running, "its timer stops");
        }

        function test_a_tip_shows_for_the_hover_time_before_it_goes() {
            churned();
            compare(root.tipWait, root.plan.step_ms + root.tipMs);
        }

        function test_a_step_with_no_tip_waits_the_step_time_alone() {
            churned();
            compare(root.stepWait, root.plan.step_ms);
        }

        function test_another_design_keeps_the_language_key_still_as_it_would_switch_the_layout() {
            root.design = "carousel";
            churned();
            compare(root.did, "kumbttkumbttccc++--");
        }

        function test_the_app_starts_the_planned_churn_through_its_windows() {
            const a = createTemporaryObject(appC, root);
            const bubbled = [];
            a.bubble.visibleChanged.connect(() => bubbled.push(a.bubble.visible));
            tryCompare(root, "ended", 1, 3000);
            const kb = a.keyboard;
            compare(bubbled, [true, false], "the bubble showed, then went as the keyboard came back");
            compare(Win.heard(kb, "tap lang false"), 2, "the language key opened and closed the list");
            compare(Win.heard(kb, "bubbleAt right"), 1, "minimise asked where the bubble goes");
            verify(kb.calls.some(c => c.startsWith("liftAlone")), "the tip showed and lifted itself");
            verify(a.face.visible && !a.bubble.visible, "the bubble brought the keyboard back");
            compare(Win.heard(kb, "bigger") + Win.heard(kb, "smaller"), 2, "both size steps");
        }
    }
}
