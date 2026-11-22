// The bar's chip over the hub's views (owner, 2026-11-22): while a model on this PC listens or writes, the processor's
// picture and the model's short name, which readers hear in a sentence, as the cloud mark names its service whole; a
// user's own long file name cut in the middle; the bar holding the longest names in all eight looks, in English, French
// and Arabic; and one size for the whole take, Listening to Transcribing, warned or not, so Stop and Cancel never move.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "pointer.js" as Pointer
import "fixtures/voice.js" as Voice
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-native-dark.js" as ND
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NH
import "../../kx-theme/fixtures/look-native-light-reduced.js" as NR
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-et66-dark.js" as ED
import "../../kx-theme/fixtures/look-dolch-light.js" as DL
import "../../kx-theme/fixtures/look-dolch-dark.js" as DD

Item {
    id: root
    width: 200
    height: 100

    // The keyboard's box the bar is placed by, as the app hands it (JSON, physical pixels).
    readonly property string over: JSON.stringify({ x: 100, y: 600, w: 900, h: 300 })
    readonly property var texts: ({ en: En, fr: Fr, ar: Ar })
    readonly property var looks: ({ "native-light": NL, "native-dark": ND, "native-contrast": NH, "native-reduced": NR, "et66-light": EL, "et66-dark": ED, "dolch-light": DL, "dolch-dark": DD })
    // The views whose chip names the model on this PC with the table's longest short name, and those naming a service:
    // Groq's, then each services row's.
    readonly property var device: ["deviceListening", "deviceNothing", "deviceTranscribing"]
    readonly property var cloud: ["cloudListening", "cloudNothing"].concat(Object.keys(Voice.views.en).filter(n => n.startsWith("cloud-")))
    // Each take's Listening views, then its Transcribing ones: a model on this PC, Groq, a first dictation that warns,
    // that one handed to the model on this PC as a password box does, with its warned Transcribing view or straight,
    // and Groq's handed over too, whose chip then names the table's longest model.
    readonly property var takes: ({
        device: [["deviceListening", "deviceNothing"], ["deviceTranscribing"]],
        cloud: [["cloudListening", "cloudNothing"], ["cloudTranscribing"]],
        warned: [["cloudWarned"], ["cloudWarnedTranscribing"]],
        handover: [["cloudWarned"], ["cloudWarnedTranscribing", "deviceTranscribing"]],
        password: [["cloudWarned"], ["deviceTranscribing"]],
        unwarned: [["cloudListening", "cloudNothing"], ["deviceTranscribing"]]
    })
    // The bundled fonts, so each bar is measured as the app draws it, whichever suites ran before.
    property var bundled: []

    Component { id: voiceC; FakeVoice {} }
    Component { id: barC; VoiceBar {} }
    // Every size its bar takes while watched, in whole units, so a change that lasts part of a step shows.
    Component {
        id: watchC
        Connections {
            property var seen: []
            function onSpanChanged() { note(); }
            function onTallChanged() { note(); }
            function note() { seen = seen.concat([[Math.ceil(target.span), Math.ceil(target.tall)]]); }
        }
    }

    TestCase {
        name: "voicebar_chip"
        when: windowShown

        function initTestCase() {
            root.bundled = Board.loadFonts(root);
            tryVerify(() => Board.loaded(root.bundled), 5000, "the bundled fonts load");
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // A shown bar over the hub's view `name` in language `tag`, look `lk` (Native light by default) and `place`.
        function make(name, tag, lk, place) {
            const look = lk || NL.lk;
            const voice = createTemporaryObject(voiceC, root, { page: null });
            const bar = createTemporaryObject(barC, root, {
                voice: voice, lk: look, res: Board.resOf(look, root.texts[tag]), s: 1, ar: tag === "ar",
                box: () => root.over, logical: (x, y) => ({ x: x, y: y })
            });
            Pointer.through([bar]);
            voice.page = pageOf(name, tag, place);
            tryVerify(() => bar.visible, 2000, "it shows");
            waitForRendering(bar.contentItem);
            return bar;
        }
        // A copy of the hub's view `name` in language `tag`, at `place` when given.
        function pageOf(name, tag, place) {
            const page = JSON.parse(JSON.stringify(Voice.views[tag][name]));
            if (place)
                page.place = place;
            return page;
        }
        // Shows the hub's view `name` on `bar`, as `make` does.
        function show(bar, name, tag, place) {
            bar.voice.page = pageOf(name, tag, place);
            waitForRendering(bar.contentItem);
        }
        // The shown parts readers hear as `name`.
        function heard(bar, name) {
            return Board.all(bar.contentItem, o => o.visible && !o.Accessible.ignored && o.Accessible.name === name);
        }
        // The chip's name on `bar`, shown once.
        function chipName(bar) {
            const name = bar.view.mark.name;
            const all = Board.all(bar.contentItem, o => o.visible && o.text === name && o.font !== undefined);
            compare(all.length, 1, "the chip's name shows once");
            return all[0];
        }
        // The shown icons on `bar` named `icon`.
        function icons(bar, icon) {
            return Board.all(bar.contentItem, o => o.visible && o.icon === icon && o.tint !== undefined);
        }

        function test_the_chip_names_the_model_on_this_pc_while_it_listens_or_writes_data() {
            return langs_data();
        }
        function test_the_chip_names_the_model_on_this_pc_while_it_listens_or_writes(d) {
            const fam = Board.resOf(NL.lk, root.texts[d.t]).fam;
            for (const name of root.device) {
                const bar = make(name, d.t);
                const mark = bar.view.mark;
                compare(heard(bar, mark.said).length, 1, name + ": readers hear who writes");
                const label = chipName(bar);
                verify(!label.truncated, name + ": whole, " + label.width + " px of " + label.implicitWidth);
                compare(label.font.family, d.t === "ar" ? fam.arabic : fam.latin, "in the language's own font, as the cloud mark");
                compare(icons(bar, NL.lk.icons.voice.device).length, 1, name + ": the processor's picture");
                compare(icons(bar, NL.lk.icons.voice.cloud).length, 0, name + ": no cloud");
                bar.visible = false;
            }
        }

        function test_a_long_own_file_name_is_cut_in_the_middle_data() {
            return langs_data();
        }
        // Past the theme's width, or sooner where the wave needs the room, a user's own file name keeps its start and its
        // end, while readers hear it whole.
        function test_a_long_own_file_name_is_cut_in_the_middle(d) {
            const bar = make("deviceOwn", d.t);
            const mark = bar.view.mark;
            const label = chipName(bar);
            compare(label.elide, Text.ElideMiddle);
            verify(label.truncated, "cut");
            const status = label.parent.parent.parent;
            fuzzyCompare(label.width, status.cutAt, 0.5, "where it is cut");
            verify(status.cutAt <= NL.lk.shape.voice.chip_name_px * bar.s, "at the theme's width or sooner");
            compare(heard(bar, mark.said).length, 1, "readers hear it");
            verify(mark.said.indexOf(mark.name) >= 0, "its whole name in the sentence");
            bar.visible = false;
        }

        // Each look's data row for each of `names` in English, French and Arabic.
        function every_look(names) {
            const all = [];
            for (const [tag, f] of Object.entries(root.looks))
                for (const name of names)
                    ["en", "fr", "ar"].forEach(t => all.push({ tag: tag + " " + name + " " + t, lk: f.lk, name, t }));
            return all;
        }
        // The shown buttons on `bar`.
        function buttons(bar) {
            return Board.all(bar.contentItem, o => o.visible && o.Accessible.role === Accessible.Button);
        }

        function test_the_bar_holds_the_longest_names_in_every_look_data() {
            return every_look(root.device.concat(["deviceOwn"], root.cloud));
        }
        // Every part sits inside the bar, and the wave or the shimmer keeps its least room: the narrow bar widens only for
        // it, never past its text width. A service's name shows whole; a model's is cut only there.
        function test_the_bar_holds_the_longest_names_in_every_look(d) {
            const bar = make(d.name, d.t, d.lk);
            const label = chipName(bar);
            const v = d.lk.shape.voice;
            const broad = Math.ceil(v.bar_px[1] * bar.s);
            if (root.cloud.indexOf(d.name) >= 0)
                verify(!label.truncated, "a service's name whole, " + label.width + " px of " + label.implicitWidth);
            else if (d.name !== "deviceOwn")
                verify(!label.truncated || bar.width === broad, "cut only at the text's width, " + label.width + " px");
            inside(bar, d.lk);
            verify(bar.width <= broad, "never past the text's width");
            const narrow = Math.ceil(v.bar_px[0] * bar.s);
            const room = v.wave_room_px * bar.s;
            verify(bar.width === narrow || bar.wave.width <= room + 1, "widened only for the wave, to " + bar.width);
            bar.visible = false;
        }
        // The chip and each button sit inside `bar` in look `lk`, the buttons clear of the chip, and the wave or the
        // shimmer keeps its least room.
        function inside(bar, lk) {
            const chip = chipName(bar).parent.parent;
            const at = chip.mapToItem(bar.body, 0, 0);
            verify(at.x >= 0 && at.x + chip.width <= bar.body.width + 0.5, "the chip inside the bar");
            const room = lk.shape.voice.wave_room_px * bar.s;
            verify(bar.wave.width >= room - 0.5, "the middle keeps " + room + " px, has " + bar.wave.width);
            for (const b of buttons(bar)) {
                const x = b.mapToItem(bar.body, 0, 0).x;
                verify(x >= 0 && x + b.width <= bar.body.width + 0.5, b.Accessible.id + " inside the bar");
                verify(x >= at.x + chip.width - 0.5 || x + b.width <= at.x + 0.5, b.Accessible.id + " clear of the chip");
            }
        }

        // Each look's data row for each take at each of `places`, in English, French and Arabic.
        function every_take(places) {
            const all = [];
            for (const [tag, f] of Object.entries(root.looks))
                for (const [take, views] of Object.entries(root.takes))
                    for (const place of places)
                        ["en", "fr", "ar"].forEach(t => all.push({ tag: [tag, take, place, t].join(" "), lk: f.lk, views, place, t }));
            return all;
        }
        // Where the hub put the bar's top, its size, then each shown button's place in it to two decimals.
        function spots(bar) {
            const at = b => b.mapToItem(bar.body, 0, 0);
            const r = v => Math.round(v * 100) / 100;
            const place = b => b.Accessible.id + " " + r(at(b).x) + " " + r(at(b).y);
            return [bar.y, bar.width, bar.height].concat(buttons(bar).map(place));
        }
        // The sizes among `all`, each `[w, h]`, that are not `size`.
        function others(all, size) {
            return all.filter(s => s[0] !== size[0] || s[1] !== size[1]);
        }

        function test_stop_and_cancel_keep_their_place_through_the_whole_take_data() {
            return every_take(["above", "bottom"]);
        }
        // One take holds one size from its first Listening view, with the hint, the warning or neither, to the end of
        // Transcribing, never another even for a moment, so the hub keeps the bar's bottom edge and nothing moves.
        function test_stop_and_cancel_keep_their_place_through_the_whole_take(d) {
            const [firsts, writes] = d.views;
            for (const first of firsts) {
                const bar = make(first, d.t, d.lk, d.place);
                const held = spots(bar);
                const size = [bar.width, bar.height];
                const watch = createTemporaryObject(watchC, root, { target: bar });
                const from = bar.voice.asks.length;
                for (const name of firsts.concat(writes)) {
                    show(bar, name, d.t, d.place);
                    compare(spots(bar), held, first + " to " + name + ": Stop and Cancel stay");
                }
                inside(bar, d.lk);
                compare(others(watch.seen, size), [], "never another size, even part of a step");
                const asked = bar.voice.asks.slice(from).map(a => [a.w, a.h]);
                verify(asked.length > 0, "the hub is asked where the bar goes");
                compare(others(asked, size), [], "the hub is asked for one size");
                bar.visible = false;
            }
        }

        function test_a_take_after_a_text_or_a_note_starts_afresh_data() {
            return every_take(["above"]);
        }
        // Try again shows Transcribing with no Listening before it, after a text or a note: the bar takes that view's own
        // size, as a fresh bar on it does.
        function test_a_take_after_a_text_or_a_note_starts_afresh(d) {
            const [firsts, writes] = d.views;
            const last = writes[writes.length - 1];
            const fresh = make(last, d.t, d.lk);
            const want = [fresh.width, fresh.height];
            fresh.visible = false;
            for (const after of ["text", "cloudNet"]) {
                const bar = make(firsts[0], d.t, d.lk);
                writes.forEach(name => show(bar, name, d.t));
                show(bar, after, d.t);
                show(bar, last, d.t);
                compare([bar.width, bar.height], want, after + " then " + last + ": its own size");
                bar.visible = false;
            }
        }
    }
}
