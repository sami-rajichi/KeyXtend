// The dictation bar's cloud parts over the hub's views (ADR-0047): the cloud mark with its service while a service
// takes the voice, which readers hear as a sentence, as they hear the hub's sentence while it listens; a cloud note's
// Try again, Use the local model dimmed unless one is ready, and Download when none is (edge 20), under the line with
// Open service for a note the service's pages settle, the longest lines whole; the card's warning in full under the row
// on the first dictation to its service, kept while it writes; and each new state told to readers once, in English,
// French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "pointer.js" as Pointer
import "fixtures/voice.js" as Voice
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    // The keyboard's box the bar is placed by, as the app hands it (JSON, physical pixels).
    readonly property string over: JSON.stringify({ x: 100, y: 600, w: 900, h: 300 })
    readonly property var texts: ({ en: En, fr: Fr, ar: Ar })

    Component { id: voiceC; FakeVoice {} }
    Component { id: barC; VoiceBar {} }

    TestCase {
        name: "voicecloud_bar"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // A shown bar over a stand-in Voice object for the hub's view `name` in language `tag`, with `more` set on it.
        function make(name, tag, more) {
            const v = JSON.parse(JSON.stringify(Voice.views[tag][name]));
            const voice = createTemporaryObject(voiceC, root, { page: null });
            const bar = createTemporaryObject(barC, root, Object.assign({
                voice: voice, lk: N.lk, res: Board.resOf(N.lk, root.texts[tag]), s: 1, ar: tag === "ar",
                box: () => root.over, logical: (x, y) => ({ x: x, y: y })
            }, more || {}));
            Pointer.through([bar]);
            voice.page = v;
            tryVerify(() => bar.visible, 2000, "it shows");
            waitForRendering(bar.contentItem);
            return bar;
        }
        // The bar's shown buttons, left to right on screen.
        function buttons(bar) {
            const all = Board.all(bar.contentItem, o => o.Accessible.role === Accessible.Button && o.visible);
            return all.sort((a, b) => a.mapToItem(null, 0, 0).x - b.mapToItem(null, 0, 0).x);
        }
        // The shown parts readers hear as `name`.
        function heard(bar, name) {
            return Board.all(bar.contentItem, o => o.visible && !o.Accessible.ignored && o.Accessible.name === name);
        }

        function test_the_cloud_mark_names_its_service_only_while_one_takes_the_voice_data() {
            return langs_data();
        }
        function test_the_cloud_mark_names_its_service_only_while_one_takes_the_voice(d) {
            const bar = make("cloudListening", d.t);
            const mark = bar.view.mark;
            compare(heard(bar, mark.said).length, 1, "readers hear where the voice goes");
            const name = Board.all(bar.contentItem, o => o.visible && o.text === mark.name && o.font !== undefined);
            compare(name.length, 1, "the service's name");
            const ar = d.t === "ar";
            const fam = Board.resOf(N.lk, root.texts[d.t]).fam;
            compare(name[0].font.family, ar ? fam.arabic : fam.latin, "in the language's own font, as a mark is");
            compare(name[0].font.pixelSize, Math.round(N.lk.shape.chip.font_px[ar ? 1 : 0]), "at its size, in whole pixels");
            const icon = Board.all(bar.contentItem, o => o.visible && o.icon === N.lk.icons.voice.cloud && o.tint !== undefined);
            compare(icon.length, 1, "the cloud mark");
            const local = make("deviceListening", d.t);
            compare(heard(local, mark.said).length, 0, "a local dictation shows no cloud mark");
            bar.visible = local.visible = false;
        }

        function test_the_first_dictation_to_a_service_that_warns_shows_its_warning_in_full_under_the_row_data() {
            return langs_data();
        }
        function test_the_first_dictation_to_a_service_that_warns_shows_its_warning_in_full_under_the_row(d) {
            const bar = make("cloudWarned", d.t);
            const text = bar.view.bar.warn;
            verify(!!text, "the card's warning");
            const line = Board.all(bar.contentItem, o => o.visible && o.text === text && o.font !== undefined);
            compare(line.length, 1, "one line holds it");
            verify(Qt.colorEqual(line[0].color, N.lk.palette.warn_ink), "in the warning tone");
            verify(!line[0].truncated, "never cut");
            const at = line[0].mapToItem(bar.body, 0, 0);
            verify(at.x >= 0 && at.x + line[0].width <= bar.body.width + 0.5, "inside the bar's width");
            verify(at.y >= 0 && at.y + line[0].height <= bar.body.height + 0.5, "inside the bar's height");
            for (const b of buttons(bar)) {
                const top = b.mapToItem(bar.body, 0, 0);
                verify(top.y >= 0 && top.y + b.height <= at.y + 0.5, b.Accessible.id + " sits above it");
            }
            compare(bar.width, Math.ceil(N.lk.shape.voice.bar_px[1]), "the bar takes its wide width");
            const plain = make("cloudListening", d.t);
            compare(Board.all(plain.contentItem, o => o.visible && o.text === text).length, 0, "no warning");
            verify(plain.height < bar.height, "the warning adds its lines");
            bar.visible = plain.visible = false;
        }

        function test_the_warning_stays_while_its_service_writes_in_the_keyboard_language_data() {
            return [["en", "ar"], ["ar", "fr"]].map(([from, to]) => ({ tag: from + " to " + to, from, to }));
        }
        // The voice leaves at Stop, so the hub words the take's warning on Transcribing while its service writes: a switch
        // of the keyboard's language brings its text, side and font, and the bar takes the taller of its held height
        // and the new one, its row at the top; the model on this PC, a text or a note shows none.
        function test_the_warning_stays_while_its_service_writes_in_the_keyboard_language(d) {
            const bar = make("cloudWarned", d.from);
            const held = bar.height;
            let t = d.from;
            const show = name => {
                bar.voice.page = JSON.parse(JSON.stringify(Voice.views[t][name]));
                waitForRendering(bar.contentItem);
            };
            const lines = text => Board.all(bar.contentItem, o => o.visible && o.text === text && o.font !== undefined);
            for (t of [d.from, d.to]) {
                bar.res = Board.resOf(N.lk, root.texts[t]);
                bar.ar = t === "ar";
                show("cloudWarnedTranscribing");
                const line = lines(bar.view.bar.warn);
                compare(line.length, 1, t + ": under the row");
                compare(line[0].horizontalAlignment, bar.ar ? Text.AlignRight : Text.AlignLeft, t + ": on its side");
                compare(line[0].font.family, bar.res.fam.ui, t + ": in its font");
                const px = N.lk.shape.notice.font_px * (bar.ar ? N.lk.look.arabic_scale : 1);
                compare(line[0].font.pixelSize, Math.round(px), t + ": at its size");
                const own = make("cloudWarnedTranscribing", t);
                compare(bar.height, Math.max(held, own.height), t + ": the taller of the held and its own height");
                own.visible = false;
                for (const b of buttons(bar)) {
                    const mid = b.mapToItem(bar.body, 0, 0).y + b.height / 2;
                    fuzzyCompare(mid, bar.row / 2, 0.5, t + ": " + b.Accessible.id + " in the row at the top");
                }
            }
            const text = bar.view.bar.warn;
            for (const name of ["deviceTranscribing", "text", "cloudNet"]) {
                show(name);
                compare(lines(text).length, 0, name + ": none");
            }
            bar.visible = false;
        }

        function test_a_cloud_note_offers_the_local_model_dimmed_with_download_when_none_is_ready_data() {
            return langs_data();
        }
        function test_a_cloud_note_offers_the_local_model_dimmed_with_download_when_none_is_ready(d) {
            const ids = name => buttons(make(name, d.t)).map(b => b.Accessible.id);
            const gear = "voice.gear";
            compare(ids("cloudNet"), ["voice.again", "voice.local", "voice.download", gear, "voice.close"], "edge 20");
            compare(ids("cloudNoLocal"), ["voice.local", gear, "voice.close"]);
            const ready = make("cloudKey", d.t);
            const use = buttons(ready).find(b => b.Accessible.id === "voice.local");
            verify(use.enabled, "a local model is ready");
            compare(use.icon, N.lk.icons.voice.local, "its own icon");
            mouseClick(use);
            compare(ready.voice.calls, ["act local local 0 " + ready.view.round]);
            const none = make("cloudNet", d.t);
            const dim = buttons(none).find(b => b.Accessible.id === "voice.local");
            verify(!dim.enabled, "dimmed, never hidden");
            compare(dim.opacity, N.lk.shape.panel.off_share);
            mouseClick(dim);
            compare(none.voice.calls, [], "a dimmed button sends nothing");
            ready.visible = none.visible = false;
        }

        // Open service wears a key on the key note, as it opens the key page, and its own page icon on the others.
        function test_open_service_wears_a_key_on_the_key_note_and_a_page_on_the_others() {
            const open = name => buttons(make(name, "en")).find(b => b.Accessible.id === "voice.service");
            const icons = N.lk.icons.voice;
            compare(open("cloudKey").icon, icons.service_key, "the key page");
            compare(open("cloudOff").icon, icons.service, "the card");
            verify(icons.service_key !== "" && icons.service_key !== icons.service, "two icons");
        }

        // The Off note with the most acts, and the account note with English's and French's longest line over them.
        function test_a_note_the_services_pages_settle_puts_its_acts_and_open_service_under_the_line_data() {
            const notes = [["cloudOff", ["voice.again", "voice.local", "voice.download", "voice.service"]],
                ["cloudAccount", ["voice.again", "voice.local", "voice.service"]]];
            const all = [];
            for (const t of ["en", "fr", "ar"])
                notes.forEach(([name, acts]) => all.push({ tag: name + " " + t, t: t, name: name, acts: acts }));
            return all;
        }
        function test_a_note_the_services_pages_settle_puts_its_acts_and_open_service_under_the_line(d) {
            const bar = make(d.name, d.t);
            const under = b => b.mapToItem(bar.body, 0, 0).y >= bar.row - 0.5;
            const all = buttons(bar);
            compare(all.filter(b => !under(b)).map(b => b.Accessible.id), ["voice.gear", "voice.close"], "Close stays");
            const acts = all.filter(under);
            compare(acts.map(b => b.Accessible.id), d.acts);
            const line = noteLine(bar);
            verify(!line.truncated, "the note reads whole over its acts");
            verify(line.mapToItem(bar.body, 0, 0).y + line.height <= bar.row + 0.5, "the line sits in the row");
            for (const b of acts) {
                const at = b.mapToItem(bar.body, 0, 0);
                verify(at.x >= 0 && at.x + b.width <= bar.body.width + 0.5, b.Accessible.id + " inside the bar's width");
                verify(at.y + b.height <= bar.body.height + 0.5, b.Accessible.id + " inside the bar's height");
                verify(b.width > b.height, b.Accessible.id + " is a pill with its word");
            }
            const net = make("cloudNet", d.t);
            verify(bar.height > net.height, "the acts add their row");
            mouseClick(acts[acts.length - 1]);
            compare(bar.voice.calls, ["act bar service 0 " + bar.view.round]);
            bar.visible = net.visible = false;
        }

        // The no-local note, Arabic's longest at 82 characters, has no acts row: it reads whole in the row, clear of Use
        // the local model, dimmed, the gear and Close, in each language.
        function test_the_no_local_note_reads_whole_beside_its_buttons_data() {
            return langs_data();
        }
        function test_the_no_local_note_reads_whole_beside_its_buttons(d) {
            const bar = make("cloudNoLocal", d.t);
            compare(bar.under, 0, "no acts row");
            const all = buttons(bar);
            compare(all.map(b => b.Accessible.id), ["voice.local", "voice.gear", "voice.close"]);
            verify(!all[0].enabled, "dimmed, as no local model is ready");
            const line = noteLine(bar);
            verify(!line.truncated, "the note reads whole");
            const at = line.mapToItem(bar.body, 0, 0);
            verify(at.x >= 0 && at.x + line.width <= bar.body.width + 0.5, "inside the bar's width");
            verify(at.y >= 0 && at.y + line.height <= bar.row + 0.5, "inside the row");
            for (const b of all) {
                const [x, w] = [b.mapToItem(bar.body, 0, 0).x, b.width];
                verify(x >= at.x + line.width - 0.5 || x + w <= at.x + 0.5, b.Accessible.id + " clear of the line");
            }
            bar.visible = false;
        }

        // The note's line in bar `bar`, shown once.
        function noteLine(bar) {
            const text = bar.view.bar.hint.text;
            const line = Board.all(bar.contentItem, o => o.visible && o.text === text && o.font !== undefined);
            compare(line.length, 1, "the note's line shows once");
            return line[0];
        }

        // While it listens, readers hear the hub's sentence, never the shown word, a cloud dictation's too.
        function test_while_it_listens_readers_hear_the_hubs_sentence_data() {
            return langs_data();
        }
        function test_while_it_listens_readers_hear_the_hubs_sentence(d) {
            const nothing = Voice.views[d.t].nothing.bar;
            verify(nothing.said !== nothing.word, "a sentence apart from the word");
            for (const name of ["listening", "cloudListening", "nothing"]) {
                const bar = make(name, d.t);
                compare(bar.spoken(), Voice.views[d.t][name].bar.said, name);
                bar.visible = false;
            }
        }

        // Each new state is told to readers once: the hub's sentence as it listens, then the text it wrote.
        function test_each_new_state_is_told_to_readers_once_data() {
            return langs_data();
        }
        function test_each_new_state_is_told_to_readers_once(d) {
            const told = [];
            const views = Voice.views[d.t];
            const bar = make("cloudListening", d.t, { announce: said => told.push(said) });
            bar.voice.page = JSON.parse(JSON.stringify(views.text));
            bar.voice.page = JSON.parse(JSON.stringify(views.text));
            compare(told, [views.cloudListening.bar.said, views.text.bar.text.text], "the sentence, then the text");
            bar.visible = false;
        }
    }
}
