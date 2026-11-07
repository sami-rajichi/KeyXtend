// A press watch hears every press without taking it, and tells a press that stands alone from one that follows another
// inside the double-click time, which asks nothing of a panel's aim (ADR-0040).
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component { id: watchC; PressWatch {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "press_watch"
        when: windowShown

        function initTestCase() {
            Pointer.through([Window.window]);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A watch with spies on both its signals.
        function made() {
            const watch = createTemporaryObject(watchC, root);
            const spy = name => createTemporaryObject(spyC, root, { target: watch, signalName: name });
            return { watch: watch, touched: spy("touched"), alone: spy("alone") };
        }

        function test_it_waits_as_long_as_the_system_says() {
            compare(made().watch.doubleMs, Qt.styleHints.mouseDoubleClickInterval);
        }

        // A shaky third tap must not ask the aim a fill took after the first.
        function test_every_press_is_heard_and_any_following_inside_the_wait_is_not_alone() {
            const { watch, touched, alone } = made();
            Board.doubled(root);
            mouseClick(watch);
            compare([touched.count, alone.count], [1, 1], "the first");
            mouseClick(watch);
            compare([touched.count, alone.count], [2, 1], "the second");
            mouseClick(watch);
            compare([touched.count, alone.count], [3, 1], "the third");
        }

        function test_a_real_double_click_stands_alone_once() {
            const { watch, touched, alone } = made();
            Board.doubled(root);
            mouseDoubleClickSequence(watch);
            compare([touched.count, alone.count], [2, 1]);
        }

        // The first press is moved back past the wait, so nothing here sleeps.
        function test_a_press_after_the_wait_stands_alone() {
            const { watch, alone } = made();
            mouseClick(watch);
            watch.last -= 2 * watch.doubleMs;
            mouseClick(watch);
            compare(alone.count, 2);
        }

        // A clock set back must not make the next press look as if it followed one.
        function test_a_press_stands_alone_when_the_clock_went_back() {
            const { watch, alone } = made();
            Board.doubled(root);
            mouseClick(watch);
            watch.last = 2 * Date.now();
            mouseClick(watch);
            compare(alone.count, 2);
        }

        function test_with_no_wait_every_press_stands_alone() {
            const { watch, touched, alone } = made();
            Board.apart(root);
            for (let i = 0; i < 3; i++)
                mouseClick(watch);
            compare([touched.count, alone.count], [3, 3]);
        }
    }
}
