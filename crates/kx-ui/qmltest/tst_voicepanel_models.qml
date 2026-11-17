// The Voice panel's Models page over the hub's views: each mark says why (in use, default, ready, not downloaded, not
// for this PC with its reason, other file), a download shows its progress with Pause and Cancel, the folder line has
// Change and Open, the first download asks for the folder, the model in use dims Delete, and a language whose default
// is Not for this PC says local models will be slow while they stay pickable, in English, French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    VoiceCase {
        name: "voicepanel_models"
        host: root

        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // The mark ids and texts row `r` of the hub's view shows.
        function want(r) {
            return r.marks.map(m => [m.id, m.text]);
        }
        function shown(row) {
            return marks(row).map(m => [m.markId, m.Accessible.name]);
        }

        function test_each_mark_says_why_data() {
            return langs_data();
        }
        function test_each_mark_says_why(d) {
            const box = make("models", d.t);
            const p = box.view;
            const all = rows(box);
            compare(all.map(r => r.Accessible.name), p.sheet.rows.map(r => r.name));
            for (const at of [0, 1, 2, 4])
                compare(shown(all[at]), want(p.sheet.rows[at]), p.sheet.rows[at].name);
            compare(all[4].Accessible.description, p.sheet.rows[4].line, "why it is not for this PC");
            verify(Kit.texts(all[4], p.sheet.rows[4].line).length === 1, "the reason shows under its name");
            compare(marks(all[0]).map(m => m.tone), ["lit", "plain"], "In use in the badge colours");
            compare(marks(all[2])[0].tone, "out", "Not downloaded outlined");
            compare(marks(all[4])[0].tone, "out", "Not for this PC outlined");
            for (const m of marks(all[0]).concat(marks(all[2]), marks(all[4])))
                compare(m.icon, N.lk.icons.voice.panel.marks[m.markId] || "", m.markId + "'s icon");
            const more = make("modelsMore", d.t);
            compare(shown(rows(more)[0]), want(more.view.sheet.rows[0]), "an other file");
            box.visible = more.visible = false;
        }

        function test_a_download_shows_its_progress_with_pause_and_cancel_data() {
            return langs_data();
        }
        function test_a_download_shows_its_progress_with_pause_and_cancel(d) {
            const box = make("models", d.t);
            const r = box.view.sheet.rows[3];
            const row = rows(box)[3];
            verify(Kit.texts(row, r.line).length === 1, "how much came");
            const bar = Board.all(row, o => o.visible && o.share !== undefined && o.track !== undefined)[0];
            compare(bar.share, r.fetch.share);
            compare(buttons(row).map(b => b.Accessible.name), r.fetch.buttons.map(b => b.label));
            mouseClick(one(box, "voice.pause-3"));
            mouseClick(one(box, "voice.cancel-3"));
            const more = make("modelsMore", d.t);
            mouseClick(one(more, "voice.resume-1"));
            compare(acts(box), ["act fetch pause 3 " + box.round, "act fetch cancel 3 " + box.round]);
            compare(acts(more), ["act fetch resume 1 " + more.round]);
            compare(one(more, "voice.resume-1").icon, N.lk.icons.voice.panel.buttons.resume, "Resume plays");
            box.visible = more.visible = false;
        }

        function test_the_folder_line_has_change_and_open() {
            const box = make("models");
            const room = box.view.sheet.room;
            compare(heard(box.contentItem, room.text).length, 1, "the folder and its free room");
            mouseClick(one(box, "voice.change"));
            mouseClick(one(box, "voice.open"));
            compare(acts(box), ["act button change 0 " + box.round, "act button open 0 " + box.round]);
            box.visible = false;
        }

        function test_the_first_download_asks_for_the_folder() {
            const box = make("modelsOffer");
            const ask = Kit.only(Board.all(box.contentItem, o => o.visible && o.Accessible.role === Accessible.AlertMessage));
            compare(ask.Accessible.name, box.view.line);
            compare(Board.all(box.contentItem, o => o.visible && o.Accessible.id === "voice.change").length, 0, "no folder line yet");
            mouseClick(one(box, "voice.yes"));
            compare(acts(box), ["act confirm  1 " + box.round]);
            box.visible = false;
        }

        function test_the_model_in_use_dims_delete() {
            const box = make("modelsUsed");
            verify(rows(box)[0].Accessible.checked, "the model in use is ticked");
            for (const id of ["voice.delete", "voice.use", "voice.download"]) {
                const b = one(box, id);
                verify(!b.enabled, id + " cannot act");
                compare(b.opacity, N.lk.shape.panel.off_share, id + " shows dimmed, never hidden");
                mouseClick(b);
            }
            compare(acts(box), []);
            box.visible = false;
        }

        function test_a_slow_language_says_so_and_its_models_stay_pickable_data() {
            return langs_data();
        }
        function test_a_slow_language_says_so_and_its_models_stay_pickable(d) {
            const box = make("modelsSlow", d.t);
            const s = box.view.sheet;
            compare(heard(box.contentItem, s.slow).length, 1, "the slow line");
            compare(heard(box.contentItem, s.untested).length, 1, "the untested order");
            const all = rows(box);
            compare(shown(all[0]), want(s.rows[0]), "Default and Not for this PC");
            compare(all[0].Accessible.description, s.rows[0].line);
            mouseClick(all[1]);
            mouseClick(all[0]);
            compare(acts(box), ["act row  1 " + box.round, "act row  0 " + box.round]);
            box.visible = false;
        }

        function test_the_best_model_and_a_note_show_as_bands() {
            const box = make("modelsMore");
            const top = box.view.top;
            compare(heard(box.contentItem, top.text + " " + top.name).length, 1, "the best model, named");
            const note = Kit.only(Board.all(box.contentItem, o => o.visible && o.Accessible.role === Accessible.AlertMessage));
            compare(note.Accessible.name, box.view.note.text, "readers hear the note");
            box.visible = false;
        }

        function test_one_ticked_model_shows_a_check() {
            const box = make("models");
            const all = rows(box);
            compare(all.map(r => r.Accessible.checked), [false, true, false, false, false]);
            const tick = Board.all(all[1], o => o.visible && o.number !== undefined)[0];
            const check = Board.all(tick, o => o.visible && o.icon === N.lk.icons.voice.panel.ticked);
            compare(check.length, 1, "a check, not a number");
            mouseClick(all[2]);
            compare(acts(box), ["act row  2 " + box.round]);
            box.visible = false;
        }
    }
}
