// Compare's cloud models over the hub's views: a click ticks one at let-go, a reader's press or toggle does too, a ticked
// one reads checked with its tick while the warning names its service, none acts while the microphone is open, and a
// pager turns their pages within the panel's height, in English, French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
import "fixtures/ui-en.js" as En

Item {
    id: root
    width: 200
    height: 100

    VoiceCase {
        name: "voicecloudpicks"
        host: root

        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // The warning bands shown.
        function warnings(box) {
            return Board.all(box.contentItem, o => o.visible && o.warn === true && o.pieces !== undefined);
        }
        // Hands the panel view `v`, as the hub's next view does.
        function turn(box, v) {
            box.view = v.panel;
            box.round = v.round;
            verify(waitForPolish(box), "laid out");
        }
        // The click a pick at `at` sends.
        function tick(box, at) {
            return "act row  " + at + " " + box.round;
        }

        function test_a_click_ticks_a_cloud_model_and_the_warning_names_its_service_data() {
            return langs_data();
        }
        function test_a_click_ticks_a_cloud_model_and_the_warning_names_its_service(d) {
            const box = make("compareCloudUnticked", d.t);
            const first = one(box, "voice.row-0");
            verify(first.Accessible.checkable && !first.Accessible.checked, "a tick target, unticked");
            compare(first.Accessible.role, Accessible.CheckBox, "readers hear a check box");
            compare(warnings(box).length, 0, "no warning before a tick");
            mouseClick(first);
            mouseClick(one(box, "voice.row-1"));
            compare(acts(box), [tick(box, 0), tick(box, 1)], "each pick sends its place");
            const v = Kit.view(d.t, "compareCloudUnticked");
            const s = v.panel.sheet;
            s.picks[0].tick = true;
            s.send = Kit.view(d.t, "compareCloud").panel.sheet.send;
            turn(box, v);
            const ticked = one(box, "voice.row-0");
            verify(ticked.Accessible.checked && !one(box, "voice.row-1").Accessible.checked, "only the ticked one");
            verify(Board.all(ticked, o => o.visible && o.number === 1).length === 1, "its tick shows, not only its colour");
            const warn = Kit.only(warnings(box));
            compare(warn.Accessible.name, s.send, "the send warning");
            verify(s.send.indexOf(s.picks[0].service) >= 0, "it names the service");
            box.visible = false;
        }

        function test_a_readers_press_or_toggle_ticks_a_cloud_model_data() {
            return langs_data();
        }
        function test_a_readers_press_or_toggle_ticks_a_cloud_model(d) {
            const box = make("compareCloudUnticked", d.t);
            one(box, "voice.row-0").Accessible.pressAction();
            one(box, "voice.row-1").Accessible.toggleAction();
            compare(acts(box), [tick(box, 0), tick(box, 1)]);
            box.visible = false;
        }

        function test_no_cloud_model_acts_while_the_microphone_is_open_data() {
            return langs_data();
        }
        // Start fixed the plan: the foot shows Cancel, and the picks only show what the run sends to.
        function test_no_cloud_model_acts_while_the_microphone_is_open(d) {
            const v = Kit.view(d.t, "compareCloud");
            v.panel.foot = Kit.view(d.t, "comparingCloud").panel.foot;
            const box = show(v, d.t);
            const first = one(box, "voice.row-0");
            mouseClick(first);
            first.Accessible.pressAction();
            compare(acts(box).filter(a => a.startsWith("act row")), [], "no tick");
            verify(first.Accessible.checked, "the planned pick still shows ticked");
            compare(warnings(box).length, 1, "and the warning stays");
            box.visible = false;
        }

        function test_a_pager_turns_the_picks_pages_within_the_panels_height_data() {
            return langs_data();
        }
        function test_a_pager_turns_the_picks_pages_within_the_panels_height(d) {
            const v = Kit.view(d.t, "compareLong");
            const s = v.panel.sheet;
            s.picks = Array.from({ length: En.settings.pageRows }, (_, i) => s.picks[i % s.picks.length]);
            v.panel.pages = { at: 1, count: 3 };
            const box = show(v, d.t);
            const [back, on] = [one(box, "voice.page-back"), one(box, "voice.page-on")];
            mouseClick(back);
            mouseClick(on);
            compare(acts(box), ["act page  -1 " + box.round, "act page  1 " + box.round]);
            const page = Kit.only(Board.all(box.contentItem, o => o.visible && o.pager !== undefined && o.body !== undefined));
            verify(page.pager > 0, "the pager's room beside the picks");
            verify(page.body.height <= page.height, "a page of picks fits: " + page.body.height + " of " + page.height);
            const edge = page.width - page.pager + 0.5;
            const picks = Board.all(page.body, o => o.visible && o.Accessible.role === Accessible.CheckBox);
            compare(picks.length, En.settings.pageRows, "a page of picks");
            compare(picks.filter(o => o.mapToItem(page, o.width, 0).x > edge).length, 0, "no pick under the pager");
            box.visible = false;
        }
    }
}
