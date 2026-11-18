// The Voice panel's cloud pages over the hub's views (ADR-0047): the Cloud page's switch, services and marks, the
// warning in its own tone, a card's facts with the day read and its link, an old card's warning, the key page's dots,
// answers and refused lines, the keep-nothing switch, the Models page's Local | Cloud chips and band, and Compare's
// cloud models with their warning and why one failed, in English, French and Arabic, each page fitting its height.
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
        name: "voicecloud"
        host: root

        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // The `[id, text]` of each mark a row shows.
        function shown(row) {
            return marks(row).map(m => [m.markId, m.Accessible.name]);
        }
        // The shown part under `box` drawn with property `key`, of which there is one.
        function part(box, key) {
            return Kit.only(Board.all(box.contentItem, o => o.visible && o[key] !== undefined && o.shows === true));
        }
        // The shown bands under `box` in the warning tone.
        function warnings(box) {
            return Board.all(box.contentItem, o => o.visible && o.warn === true && o.pieces !== undefined);
        }
        // `page` lays its parts inside its own height, which is at least a page of rows, as the mock-up's .card is.
        function fits(page, what) {
            verify(page.body.height <= page.height, what + " fits its page: " + page.body.height + " of " + page.height);
            verify(page.height >= page.host.pageH, what + " keeps at least a page of rows");
        }

        function test_the_cloud_page_shows_its_switch_and_each_service_with_its_marks_data() {
            return langs_data();
        }
        function test_the_cloud_page_shows_its_switch_and_each_service_with_its_marks(d) {
            const box = make("cloud", d.t);
            const s = box.view.sheet;
            const sw = one(box, "voice.cloud");
            compare([sw.Accessible.name, sw.Accessible.checked], [s.switch.label, true]);
            const all = rows(box);
            compare(all.map(r => r.Accessible.name), s.rows.map(r => r.name));
            compare(all.map(r => r.Accessible.description), s.rows.map(r => r.line));
            compare(shown(all[0]), [[box.names.untested, s.rows[0].tag]].concat(s.rows[0].marks.map(m => [m.id, m.text])));
            const warn = marks(all[2])[0];
            compare([warn.markId, warn.tone, warn.icon], ["warn", "warn", ""], "the warning in its tone, with no sign");
            verify(Qt.colorEqual(warn.fill, N.lk.palette.warn));
            verify(Kit.ratio(warn.ink, warn.fill) >= Kit.text, "its word reads");
            compare(marks(all[2])[1].tone, "out", "Off outlined");
            compare(buttons(box.foot).map(b => b.Accessible.name), box.view.foot.map(b => b.label));
            verify(all[0].Accessible.checked, "the ticked service");
            mouseClick(sw);
            mouseClick(all[1]);
            mouseClick(one(box, "voice.lines"));
            compare(acts(box), ["act cloud  0 " + box.round, "act row  1 " + box.round, "act service lines 0 " + box.round]);
            box.visible = false;
        }

        function test_cloud_services_off_say_so_and_their_rows_only_show() {
            const box = make("cloudOff");
            const s = box.view.sheet;
            compare(heard(box.contentItem, s.off).length, 1, "nothing is offered or sent");
            const all = rows(box);
            verify(all.every(r => !r.Accessible.checkable), "no tick while off");
            mouseClick(all[0]);
            mouseClick(one(box, "voice.cloud"));
            compare(acts(box), ["act cloud  1 " + box.round]);
            box.visible = false;
        }

        function test_a_card_lists_its_facts_with_the_day_read_and_its_link_data() {
            return langs_data();
        }
        function test_a_card_lists_its_facts_with_the_day_read_and_its_link(d) {
            const box = make("card", d.t);
            const s = box.view.sheet;
            for (const f of s.facts)
                compare(heard(box.contentItem, f.label + " " + f.value.text).length, 1, f.id);
            compare(heard(box.contentItem, s.read.text).length, 1, "the day the facts were read");
            compare(heard(box.contentItem, s.untested).length, 1, "edge 18: Untested");
            compare(warnings(box).length, 0, "no warning");
            fits(part(box, "read"), "the card");
            mouseClick(one(box, "voice.link"));
            mouseClick(one(box, "voice.yes"));
            compare(acts(box), ["act service link 0 " + box.round, "act confirm  1 " + box.round]);
            box.visible = false;
        }

        function test_an_old_card_says_so_in_the_warning_tone_data() {
            return langs_data();
        }
        function test_an_old_card_says_so_in_the_warning_tone(d) {
            const box = make("cardOld", d.t);
            const s = box.view.sheet;
            compare(heard(box.contentItem, s.read.text + " " + s.read.old).length, 1, "edge 15: the facts may be old");
            const old = Kit.texts(box.contentItem, s.read.old)[0];
            verify(Qt.colorEqual(old.color, N.lk.palette.warn_ink), "in warn_ink");
            const band = Kit.only(warnings(box));
            compare(band.Accessible.name, s.warn.text);
            compare(band.pieces.map(p => p.text), s.warn.parts.map(p => p.text), "a service's own words apart");
            verify(Kit.ratio(band.ink, band.fill) >= Kit.text, "the warning reads");
            const fact = s.facts.find(f => f.warn);
            const words = Kit.texts(box.contentItem, fact.value.parts[0].text)[0];
            verify(Qt.colorEqual(words.color, N.lk.palette.warn_ink), "a warning fact in warn_ink");
            fits(part(box, "read"), "the old card");
            box.visible = false;
        }

        function test_a_card_and_a_typed_key_page_turn_by_the_pager_data() {
            return langs_data();
        }
        // A card's two parts and a typed key page's two parts each keep the panel's height, with the pager beside them.
        function test_a_card_and_a_typed_key_page_turn_by_the_pager(d) {
            for (const [name, by] of [["cardLong", 1], ["cardLongPrice", -1], ["serviceLong", 1], ["serviceLongModel", -1]]) {
                const box = make(name, d.t);
                const [back, on] = [one(box, "voice.page-back"), one(box, "voice.page-on")];
                compare([back.enabled, on.enabled], [by < 0, by > 0], name + ": one way to turn");
                mouseClick(by > 0 ? on : back);
                compare(acts(box), ["act page  " + by + " " + box.round], name);
                box.visible = false;
            }
            const drawn = (box, id) => Board.all(box.contentItem, o => o.visible && o.Accessible.id === "voice." + id).length;
            const [key, model] = [make("serviceLong", d.t), make("serviceLongModel", d.t)];
            compare(["line-key", "line-address", "line-model", "keep"].map(id => drawn(key, id)), [1, 0, 0, 0], "the key first");
            compare(["line-key", "line-address", "line-model", "keep"].map(id => drawn(model, id)), [0, 1, 1, 1], "then the rest");
            key.visible = model.visible = false;
        }

        function test_the_key_page_shows_the_key_as_dots_with_its_answer_data() {
            return langs_data();
        }
        function test_the_key_page_shows_the_key_as_dots_with_its_answer(d) {
            const box = make("service", d.t);
            const s = box.view.sheet;
            const key = one(box, "voice.line-key");
            verify(key.masked, "the key never shows");
            compare(key.drawn, s.key.mask.charAt(0).repeat(s.key.dots));
            compare(key.Accessible.description, s.key.mask, "readers hear the mask");
            verify(!key.aims, "a click never aims letters at it");
            compare(heard(box.contentItem, s.check.text).length, 1, "the test's answer");
            compare(heard(box.contentItem, s.link.text).length, 1, "where keys are made");
            compare(heard(box.contentItem, box.view.line).length, 1, "the key is sealed");
            fits(part(box, "lines"), "the key page");
            mouseClick(one(box, "voice.paste"));
            mouseClick(one(box, "voice.test"));
            mouseClick(one(box, "voice.link"));
            mouseClick(one(box, "voice.remove"));
            const sent = ["paste", "test", "link", "remove"].map(id => "act service " + id + " 0 " + box.round);
            compare(acts(box), sent);
            box.visible = false;
        }

        function test_refused_lines_say_why_and_keep_nothing_switches_data() {
            return langs_data();
        }
        function test_refused_lines_say_why_and_keep_nothing_switches(d) {
            const router = make("serviceRouter", d.t);
            const r = router.view.sheet;
            compare(heard(router.contentItem, r.model.fault).length, 1, "edge 17: no host keeps nothing");
            const keep = one(router, "voice.keep");
            compare([keep.Accessible.name, keep.Accessible.checked], [r.keep.label, true]);
            mouseClick(keep);
            compare(acts(router), ["act keep  0 " + router.round]);
            fits(part(router, "lines"), "OpenRouter's page");
            const custom = make("serviceCustom", d.t);
            const c = custom.view.sheet;
            compare(heard(custom.contentItem, c.address.fault).length, 1, "carry 1: the address fault");
            compare(heard(custom.contentItem, c.model.fault).length, 1, "carry 5: the refused model");
            compare(one(custom, "voice.line-address").drawn, c.address.text);
            compare(one(custom, "voice.line-key").drawn, "", "no key yet shows its hint");
            fits(part(custom, "lines"), "the Custom page");
            router.visible = custom.visible = false;
        }

        function test_the_cloud_side_lists_its_models_with_their_service_data() {
            return langs_data();
        }
        function test_the_cloud_side_lists_its_models_with_their_service(d) {
            const box = make("modelsCloud", d.t);
            const s = box.view.sheet;
            const local = one(box, "voice.side-local");
            const cloud = one(box, "voice.side-cloud");
            verify(cloud.shown && !local.shown, "edge 29: the Cloud side shows");
            compare(heard(box.contentItem, s.band.text).length, 1, "the band");
            const all = rows(box);
            for (const [at, r] of s.rows.entries())
                compare(shown(all[at]), r.marks.map(m => [m.id, m.text]), r.name);
            compare(all[1].Accessible.description, s.rows[1].line, "why it is Unavailable");
            compare(marks(all[0])[0].icon, N.lk.icons.voice.panel.marks.cloud, "the cloud mark");
            mouseClick(local);
            mouseClick(one(box, "voice.cloud"));
            compare(acts(box), ["act side local 0 " + box.round, "act open cloud 0 " + box.round]);
            box.visible = false;
        }

        function test_compare_warns_where_the_voice_goes_and_says_why_a_cloud_model_failed_data() {
            return langs_data();
        }
        function test_compare_warns_where_the_voice_goes_and_says_why_a_cloud_model_failed(d) {
            const box = make("compareCloud", d.t);
            const s = box.view.sheet;
            compare(Kit.only(warnings(box)).Accessible.name, s.send, "the send warning before Start");
            const p = s.picks[0];
            compare(heard(box.contentItem, p.name + " " + p.service).length, 1, "a ticked cloud model");
            const run = make("comparingCloud", d.t);
            compare(rows(run)[0].Accessible.description, run.view.sheet.rows[0].why, "carry 5: why it failed");
            box.visible = run.visible = false;
        }
    }
}
