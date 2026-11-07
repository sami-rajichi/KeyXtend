// The mouse wheel over a list panel (ADR-0040): one notch turns one page as the page buttons do, small steps add up to a
// notch, a burst turns several pages, and it does nothing with no page to turn or while a question shows. It is an extra
// (ADR-0008): it takes no press and aims nothing.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "clipkit.js" as Clip
import "fillkit.js" as Fill
import "pointer.js" as Pointer
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as NL

Item {
    id: root
    width: 200
    height: 100

    readonly property real notch: NL.lk.shape.panel.wheel_notch

    Component { id: uiC; FakeUi {} }
    Component { id: fillC; FakeFill {} }
    Component { id: clipC; ClipBox {} }
    Component { id: quickC; QuickFillBox {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "panel_wheel"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A shown panel from `component` with `props`, the real pointer passing through it.
        function shown(component, props) {
            const base = { ui: createTemporaryObject(uiC, root), lk: NL.lk, res: Board.resOf(NL.lk, En), s: 1, rows: En.settings.pageRows, rtl: false };
            const box = createTemporaryObject(component, root, Object.assign(base, props));
            Pointer.through([box]);
            box.open();
            verify(waitForPolish(box), "laid out");
            return box;
        }
        function clip(view) {
            return shown(clipC, { view: view });
        }
        function quick(view) {
            return shown(quickC, { view: view, fill: createTemporaryObject(fillC, root) });
        }
        function clipActs(box) {
            return box.ui.calls.filter(c => c.startsWith("clipAct"));
        }
        // View `name` of `kit`, on page `at` of `count`.
        function paged(kit, name, at, count) {
            const v = kit.view("en", name);
            v.page = { at: at, count: count };
            return v;
        }
        // Wheels `delta` over `box`'s middle: up is above 0, as Qt counts it.
        function wheel(box, delta) {
            mouseWheel(box.contentItem, box.width / 2, box.height / 2, 0, delta);
        }

        function test_the_notch_comes_from_the_theme() {
            verify(root.notch > 0, "shape.toml panel.wheel_notch");
        }

        function test_one_notch_turns_one_clipboard_page_each_way() {
            const box = clip(paged(Clip, "list", 1, 3));
            wheel(box, -root.notch);
            compare(clipActs(box), ["clipAct wheel 1"], "down goes on");
            wheel(box, root.notch);
            compare(clipActs(box), ["clipAct wheel 1", "clipAct wheel -1"], "up goes back");
            compare(box.ui.clipRounds, [box.view.round, box.view.round], "with the round drawn");
            box.visible = false;
        }

        function test_one_notch_turns_one_quick_fill_page_each_way() {
            const box = quick(paged(Fill, "list", 1, 3));
            wheel(box, -root.notch);
            compare(box.fill.calls, ["wheel 1"], "down goes on");
            wheel(box, root.notch);
            compare(box.fill.calls, ["wheel 1", "wheel -1"], "up goes back");
            compare(box.fill.rounds, [box.view.round, box.view.round], "with the round drawn");
            box.visible = false;
        }

        // Only the page buttons say `page`, the kind that sounds; the wheel has its own, which the hub keeps silent.
        function test_a_page_button_still_sends_the_kind_that_sounds() {
            const box = clip(paged(Clip, "list", 1, 3));
            mouseClick(Board.all(box.contentItem, o => o.visible && o.Accessible.id === "clip.page-on")[0]);
            compare(clipActs(box), ["clipAct page 1"]);
            const fill = quick(paged(Fill, "list", 1, 3));
            mouseClick(Board.all(fill.contentItem, o => o.visible && o.Accessible.id === "fill.page-on")[0]);
            compare(fill.fill.calls, ["page 1"]);
            box.visible = fill.visible = false;
        }

        function test_small_steps_add_up_to_one_notch() {
            const box = clip(paged(Clip, "list", 1, 3));
            wheel(box, -root.notch / 2);
            compare(clipActs(box), [], "half a notch waits");
            wheel(box, -root.notch / 2);
            compare(clipActs(box), ["clipAct wheel 1"], "the other half turns");
            const fine = quick(paged(Fill, "list", 1, 3));
            for (let i = 0; i < 4; i++)
                wheel(fine, root.notch / 4);
            compare(fine.fill.calls, ["wheel -1"], "a fine wheel's four steps");
            box.visible = fine.visible = false;
        }

        // Half a notch left from an earlier spin must not make the next spin's first step turn a page.
        function test_a_leftover_clears_when_the_wheel_rests() {
            const boxes = [clip(paged(Clip, "list", 1, 3)), quick(paged(Fill, "list", 1, 3))];
            for (const box of boxes) {
                compare(box.wheel.activeTimeout, NL.lk.shape.panel.wheel_rest_s, "the rest comes from the theme");
                box.wheel.activeTimeout = 0.01;
                wheel(box, -root.notch / 2);
                compare(box.wheel.held, -root.notch / 2, "half a notch is held");
                tryCompare(box.wheel, "held", 0);
                wheel(box, -root.notch / 2);
                compare(box.wheel.held, -root.notch / 2, "the first half is gone");
                box.visible = false;
            }
            compare(clipActs(boxes[0]).concat(boxes[1].fill.calls), [], "no page turned");
        }

        function test_steps_the_other_way_take_back_what_was_held() {
            const box = clip(paged(Clip, "list", 1, 3));
            wheel(box, -root.notch / 2);
            wheel(box, root.notch / 2);
            wheel(box, -root.notch / 2);
            compare(clipActs(box), [], "only half a notch is held");
            box.visible = false;
        }

        // A burst the view has not caught up with turns a page per notch; the module stops at the last page.
        function test_a_burst_turns_a_page_per_notch() {
            const box = clip(paged(Clip, "list", 0, 3));
            wheel(box, -root.notch);
            wheel(box, -root.notch);
            compare(clipActs(box), ["clipAct wheel 1", "clipAct wheel 1"]);
            const fast = quick(paged(Fill, "list", 0, 3));
            wheel(fast, -2 * root.notch);
            compare(fast.fill.calls, ["wheel 1", "wheel 1"], "two notches in one step");
            box.visible = fast.visible = false;
        }

        function test_nothing_turns_past_either_end() {
            const first = clip(paged(Clip, "list", 0, 3));
            wheel(first, root.notch);
            compare(clipActs(first), [], "before the first clipboard page");
            const last = clip(paged(Clip, "list", 2, 3));
            wheel(last, -root.notch);
            compare(clipActs(last), [], "after the last clipboard page");
            const start = quick(paged(Fill, "list", 0, 3));
            wheel(start, root.notch);
            const end = quick(paged(Fill, "list", 2, 3));
            wheel(end, -root.notch);
            compare(start.fill.calls.concat(end.fill.calls), [], "either end of Quick-fill");
            first.visible = last.visible = start.visible = end.visible = false;
        }

        function test_a_page_with_nothing_to_turn_sends_nothing() {
            for (const name of ["short", "empty", "pinned"]) {
                const box = clip(Clip.view("en", name));
                wheel(box, -root.notch);
                wheel(box, root.notch);
                compare(clipActs(box), [], name);
                box.visible = false;
            }
            for (const name of ["short", "entry", "edit", "pass-new", "add"]) {
                const box = quick(Fill.view("en", name));
                const pages = box.view.page.count;
                wheel(box, -root.notch);
                wheel(box, root.notch);
                compare(box.fill.calls, pages > 1 ? ["wheel 1"] : [], name);
                box.visible = false;
            }
        }

        // The Backup, folder and found pages page as the list does.
        function test_every_paged_quick_fill_page_turns() {
            for (const name of ["backup-paged", "folders"]) {
                const box = quick(Fill.view("en", name));
                wheel(box, -root.notch);
                compare(box.fill.calls, ["wheel 1"], name);
                box.visible = false;
            }
            const found = quick(paged(Fill, "found", 0, 2));
            wheel(found, -root.notch);
            compare(found.fill.calls, ["wheel 1"], "found");
            found.visible = false;
        }

        function test_a_question_blocks_the_clipboards_wheel() {
            const box = clip(paged(Clip, "confirm", 1, 3));
            wheel(box, -root.notch);
            wheel(box, root.notch);
            compare(clipActs(box), []);
            box.visible = false;
        }

        function test_a_question_blocks_quick_fills_wheel() {
            for (const name of ["confirm", "remove", "replace", "foreign"]) {
                const box = quick(paged(Fill, name, 1, 3));
                wheel(box, -root.notch);
                wheel(box, root.notch);
                compare(box.fill.calls, [], name);
                box.visible = false;
            }
        }

        // The page buttons stop on a question as the wheel does.
        function test_a_question_stops_quick_fills_page_buttons() {
            for (const name of ["confirm", "remove", "replace", "foreign"]) {
                const box = quick(paged(Fill, name, 1, 3));
                for (const id of ["fill.page-on", "fill.page-back"]) {
                    const page = Board.all(box.contentItem, o => o.visible && o.Accessible.id === id)[0];
                    compare(page.can, false, name + " " + id);
                    mouseClick(page);
                    page.Accessible.pressAction();
                }
                compare(box.fill.calls, [], name);
                box.visible = false;
            }
        }

        // The preview's text pages itself, as its page buttons do, and tells the hub nothing.
        function test_the_previews_text_turns_its_own_pages() {
            const box = clip(Clip.view("en", "text"));
            const text = Board.all(box.contentItem, o => o.visible && o.ends !== undefined)[0];
            tryVerify(() => text.count > 1, 2000, "a long text takes pages");
            wheel(box, -root.notch);
            compare(text.at, 1, "on");
            wheel(box, root.notch);
            compare(text.at, 0, "back");
            wheel(box, root.notch);
            compare(text.at, 0, "stops at the first");
            compare(clipActs(box), [], "the panel pages it itself");
            box.visible = false;
        }

        function test_the_previews_files_turn_their_own_pages_and_an_image_has_none() {
            const box = clip(Clip.view("en", "files"));
            const files = Board.all(box.contentItem, o => o.visible && o.ends !== undefined)[0];
            wheel(box, -root.notch);
            compare(files.at, 1, "on");
            compare(clipActs(box), []);
            const picture = clip(Clip.view("en", "image"));
            wheel(picture, -root.notch);
            compare(clipActs(picture), [], "nothing to turn");
            box.visible = picture.visible = false;
        }

        // The wheel changes no aim and no focus: only a press does, and neither panel ever takes focus.
        function test_the_wheel_takes_no_press_and_aims_nothing() {
            const box = clip(paged(Clip, "list", 1, 3));
            const quiet = quick(paged(Fill, "list", 1, 3));
            for (const b of [box, quiet]) {
                const spies = ["aiming", "touched"].map(s => createTemporaryObject(spyC, root, { target: b, signalName: s }));
                wheel(b, -root.notch);
                compare(spies.map(s => s.count), [0, 0], "no press, no aim");
                verify(b.flags & Qt.WindowDoesNotAcceptFocus, "no focus");
                b.visible = false;
            }
        }
    }
}
