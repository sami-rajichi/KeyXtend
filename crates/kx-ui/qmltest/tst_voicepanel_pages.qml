// The Voice panel's other pages over the hub's views: devices with their memory, check and mark, Compare's set text,
// its run with each step's mark and its table, Options with switches, steppers and chips and never a slider, and the
// drives, in English, French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-reduced.js" as NR

Item {
    id: root
    width: 200
    height: 100

    VoiceCase {
        name: "voicepanel_pages"
        host: root

        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // The progress line under `item`.
        function bar(item) {
            return Board.all(item, o => o.visible && o.share !== undefined && o.track !== undefined)[0];
        }

        function test_each_device_shows_its_memory_and_check_data() {
            return langs_data();
        }
        function test_each_device_shows_its_memory_and_check(d) {
            const box = make("device", d.t);
            const p = box.view;
            const all = rows(box);
            compare(all.map(r => r.Accessible.name), p.sheet.rows.map(r => r.name));
            compare(all.map(r => r.Accessible.description), p.sheet.rows.map(r => r.line), "each one's memory");
            const tiles = all.map(r => Board.all(r, o => o.visible && o.icon !== undefined && o.sh !== undefined && o.label === undefined)[0].icon);
            compare(tiles, p.sheet.rows.map(r => N.lk.icons.voice.panel.kinds[r.kind]), "each kind's icon");
            for (const at of [0, 2, 3])
                compare(marks(all[at]).map(m => [m.markId, m.Accessible.name]), [[p.sheet.rows[at].mark.id, p.sheet.rows[at].mark.text]]);
            const check = p.sheet.rows[4].check;
            compare(heard(all[4], check.line).length, 1, "what the check does now");
            compare(bar(all[4]).share, check.share);
            compare(heard(box.contentItem, p.top.text).length, 1, "why the check failed");
            mouseClick(one(box, "voice.check"));
            compare(acts(box), ["act button check 0 " + box.round]);
            box.visible = false;
        }

        function test_compare_reads_the_set_text_aloud_data() {
            return langs_data();
        }
        function test_compare_reads_the_set_text_aloud(d) {
            const box = make("compare", d.t);
            const s = box.view.sheet;
            for (const line of [s.intro, s.text, s.line])
                compare(heard(box.contentItem, line).length, 1, line);
            const text = Kit.texts(box.contentItem, s.text)[0];
            compare(text.effectiveHorizontalAlignment, d.t === "ar" ? Text.AlignRight : Text.AlignLeft, "in its language's way");
            mouseClick(one(box, "voice.start"));
            compare(acts(box), ["act compare  0 " + box.round]);
            box.visible = false;
        }

        function test_compare_marks_each_model_as_it_runs_data() {
            return langs_data();
        }
        function test_compare_marks_each_model_as_it_runs(d) {
            const box = make("comparing", d.t);
            const p = box.view;
            const all = rows(box);
            compare(heard(box.contentItem, p.top.text + " " + p.top.name).length, 1, "the step, named");
            compare(all.map(r => r.Accessible.name), p.sheet.rows.map(r => r.name));
            const failed = marks(all[1])[0];
            compare([failed.markId, failed.Accessible.name, failed.tone], ["run-failed", p.sheet.rows[1].mark.text, "out"]);
            compare(failed.icon, N.lk.icons.voice.panel.marks["run-failed"]);
            compare(marks(all[2]).length, 0, "Writing draws its line");
            compare(heard(all[2], p.sheet.rows[2].mark.text).length, 1);
            const line = bar(all[2]);
            verify(line.running, "Writing draws the progress line");
            compare(line.part.width, line.width * N.lk.shape.voice.run_share[0], "a part of it");
            tryVerify(() => line.part.x > 0, 2000, "slides while it writes");
            mouseClick(all[0]);
            mouseClick(one(box, "voice.cancel"));
            compare(acts(box), ["act button cancel 0 " + box.round], "a running row takes no tick");
            box.visible = false;
        }

        // Reduced motion (mock-up .still .prog.run i): the line fills, dimmed, and stands still.
        function test_the_writing_line_stands_still_with_reduced_motion() {
            const box = make("comparing", "en", NR.lk);
            const line = bar(rows(box)[2]);
            compare([line.part.width, line.part.opacity, line.part.x], [line.width, NR.lk.shape.voice.run_share[1], 0]);
            box.visible = false;
        }

        function test_compare_shows_its_table_for_the_device_it_ran_on_data() {
            return langs_data();
        }
        function test_compare_shows_its_table_for_the_device_it_ran_on(d) {
            const box = make("results", d.t);
            const p = box.view;
            verify(Kit.texts(box.contentItem, p.sub).length === 1, "the day and the device it ran on");
            for (const h of p.sheet.head)
                compare(Kit.texts(box.contentItem, h).length, 1, "the column " + h);
            const all = rows(box);
            compare(all.map(r => r.Accessible.checked), p.sheet.rows.map(r => r.tick));
            for (const [at, r] of p.sheet.rows.entries())
                for (const cell of [r.wrong, r.wait, r.memory])
                    compare(Kit.texts(all[at], cell).length, 1, r.name + ": " + cell);
            compare(heard(box.contentItem, p.top.text).length, 1, "what is kept");
            mouseClick(all[0]);
            const free = make("freeResults", d.t);
            compare(Kit.texts(rows(free)[0], free.view.sheet.rows[0].text).length, 1, "with no set text, its text");
            compare(acts(box), ["act row  0 " + box.round]);
            box.visible = free.visible = false;
        }

        function test_options_use_switches_and_steppers_never_a_slider_data() {
            return langs_data();
        }
        function test_options_use_switches_and_steppers_never_a_slider(d) {
            const box = make("options", d.t);
            const s = box.view.sheet;
            compare(Board.all(box.contentItem, o => o.Accessible.role === Accessible.Slider).length, 0, "no slider");
            const switches = Board.all(box.contentItem, o => o.visible && o.Accessible.role === Accessible.CheckBox);
            compare(switches.map(o => o.Accessible.name), [s.rows[0].label, s.rows[1].label]);
            compare(switches.map(o => o.Accessible.checked), [true, false]);
            for (const [at, r] of s.rows.entries())
                compare(Kit.texts(box.contentItem, r.value).length, 1, r.key + " shows " + r.value);
            const dim = [one(box, "voice.minus-2"), one(box, "voice.plus-2")];
            compare(dim.map(b => b.enabled), [false, false], "under its switch, it waits");
            compare(dim.map(b => b.Accessible.name), [s.rows[2].steps.minus, s.rows[2].steps.plus]);
            verify(!one(box, "voice.minus-4").enabled, "the first place");
            compare(one(box, "voice.plus-4").icon, N.lk.icons.voice.panel.after, "a choice steps with chevrons");
            compare(one(box, "voice.plus-3").icon, N.lk.icons.voice.panel.plus);
            mouseClick(switches[1]);
            mouseClick(one(box, "voice.plus-3"));
            mouseClick(one(box, "voice.minus-3"));
            mouseClick(dim[1]);
            const r = box.round;
            compare(acts(box), ["act flip  1 " + r, "act step plus 3 " + r, "act step minus 3 " + r]);
            verify(Kit.texts(box.contentItem, box.view.line).length === 1, "changes apply at once");
            const more = make("optionsMore", d.t);
            compare(Kit.texts(more.contentItem, more.view.sheet.rows[0].value).length, 1, "Threads reads Automatic");
            verify(!one(more, "voice.plus-3").enabled, "the most texts kept");
            box.visible = more.visible = false;
        }

        function test_insert_the_text_is_two_chips_and_paste_shows_its_hint_below_data() {
            return langs_data();
        }
        function test_insert_the_text_is_two_chips_and_paste_shows_its_hint_below(d) {
            const box = make("optionsInsert", d.t);
            const [pair, hint] = [box.view.sheet.rows[1], box.view.sheet.rows[2]];
            const chips = Board.all(box.contentItem, o => o.visible && o.Accessible.role === Accessible.RadioButton);
            compare(chips.map(c => c.Accessible.name), pair.chips.map(c => c.label), "both ways");
            compare(chips.map(c => c.Accessible.checked), [false, true], "Paste at once chosen");
            compare(chips.map(c => c.Accessible.id), ["voice.minus-1", "voice.plus-1"], "a stepper's ids");
            compare(heard(box.contentItem, pair.label).length, 1, "the row's name");
            const line = heard(box.contentItem, hint.label);
            compare(line.length, 1, "the hint takes the next row");
            verify(!line[0].nameText.truncated, "the hint reads whole");
            verify(!line[0].takes, "the hint takes no click");
            mouseClick(chips[0]);
            mouseClick(chips[1]);
            mouseClick(line[0]);
            const r = box.round;
            compare(acts(box), ["act step minus 1 " + r, "act step plus 1 " + r]);
            box.visible = false;
        }

        function test_the_drives_pick_the_models_folder() {
            const box = make("folders");
            const all = rows(box);
            compare(all.map(r => r.Accessible.name), box.view.sheet.rows);
            verify(!one(box, "voice.here").enabled, "Use this folder waits for a folder");
            mouseClick(all[1]);
            compare(acts(box), ["act row  1 " + box.round]);
            box.visible = false;
        }
    }
}
