// The dictation bar over the stand-in Voice object (design "The bar"): each state's words and buttons in one order in
// English, French and Arabic, the gear before the last button, Stop dark and dimmed while it writes, Arabic keeping
// Cancel left and Stop right with its text read right to left, a long text cut to its first lines, every icon loaded in
// its tint, the red dot beside any note while the microphone listens, Copy turning to Copied, and a window that takes
// clicks but never focus, each button found by its accessible id.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/voice.js" as Voice
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-contrast.js" as H

Item {
    id: root
    width: 200
    height: 100

    // The keyboard's box the bar is placed by, as the app hands it (JSON, physical pixels).
    readonly property string over: JSON.stringify({ x: 100, y: 600, w: 900, h: 300 })
    readonly property var texts: ({ en: En, fr: Fr, ar: Ar })

    Component { id: voiceC; FakeVoice {} }
    Component { id: barC; VoiceBar {} }
    Component { id: namesC; VoiceNames {} }

    TestCase {
        name: "voicebar"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A copy of the hub's view `name` in language `tag`, with `more` laid over it.
        function view(tag, name, more) {
            return Object.assign(JSON.parse(JSON.stringify(Voice.views[tag][name])), more || {});
        }
        // A bar over a stand-in Voice object showing `v`, in language `tag` and look `lk` (Native light by default).
        function make(v, tag, lk) {
            const t = tag || "en";
            const look = lk || N.lk;
            const voice = createTemporaryObject(voiceC, root, { page: v });
            const bar = createTemporaryObject(barC, root, {
                voice: voice, lk: look, res: Board.resOf(look, root.texts[t]), s: 1, ar: t === "ar",
                box: () => root.over, logical: (x, y) => ({ x: x, y: y })
            });
            if (v.bar.state !== "hidden") {
                tryVerify(() => bar.visible, 2000, "it shows");
                waitForRendering(bar.contentItem);
            }
            return bar;
        }
        // The bar's shown buttons as a reader finds them, left to right on screen.
        function buttons(bar) {
            const all = Board.all(bar.contentItem, o => o.Accessible.role === Accessible.Button && o.visible);
            return all.sort((a, b) => a.mapToItem(null, 0, 0).x - b.mapToItem(null, 0, 0).x);
        }
        function names(bar) {
            return buttons(bar).map(o => o.Accessible.name);
        }
        function withId(bar, uid) {
            return buttons(bar).find(o => o.Accessible.id === uid);
        }
        // The shown text item under `bar` reading `text`.
        function label(bar, text) {
            return Board.all(bar.contentItem, o => o.text === text && o.font !== undefined && o.visible)[0];
        }
        // The red dot, when it shows.
        function dot(bar) {
            return Board.all(bar.contentItem, o => o.pulsing !== undefined && o.visible)[0];
        }
        // The text, dots or note under the bar.
        function note(bar) {
            return Board.all(bar.contentItem, o => o.words !== undefined && o.hint !== undefined)[0];
        }

        function test_the_click_kinds_states_and_ids_are_the_hubs() {
            const names = createTemporaryObject(namesC, root);
            compare(names.acts, Voice.acts);
            const views = Object.values(Voice.views.en);
            const states = [...new Set(views.map(v => v.bar.state))].sort();
            compare(states, Object.values(names.states).sort(), "every state the hub writes");
            verify(Voice.views.en.text.bar.buttons.some(b => b.id === names.copy), "Copy's id");
            verify(Voice.views.en.listening.bar.buttons.some(b => b.tone === names.plain), "the plain tone");
            verify(views.every(v => v.bar.buttons === undefined || v.bar.buttons.every(b => b.id !== names.gear)), "the gear's own id");
        }

        function test_each_state_shows_its_words_and_buttons_in_order_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        function test_each_state_shows_its_words_and_buttons_in_order(d) {
            const by = name => Voice.views[d.t][name];
            const gear = by("listening").words.gear;
            const named = (name, id) => by(name).bar.buttons.find(b => b.id === id).label;
            const states = [
                ["listening", ["cancel", "", "stop"], by("listening").bar.word],
                ["transcribing", ["cancel", "", "stop"], by("transcribing").bar.word],
                ["text", ["copy", "undo", "", "close"], by("text").bar.text.text],
                ["password", ["undo", "", "close"], by("password").bar.hint.text],
                ["heardNothing", ["listen", "", "close"], by("heardNothing").bar.hint.text],
                ["cancelled", ["anyway", "", "close"], by("cancelled").bar.hint.text],
                ["noModel", ["download", "", "close"], by("noModel").bar.hint.text],
                ["fault", ["", "close"], by("fault").bar.hint.text]
            ];
            for (const [name, ids, words] of states) {
                const bar = make(view(d.t, name), d.t);
                const want = ids.map(id => id === "" ? gear : named(name, id));
                compare(names(bar), want, name + ": the gear before the last button");
                verify(!!label(bar, words), name + " reads " + words);
                compare(bar.title, by(name).words.window, name);
                compare(bar.body.Accessible.name, by(name).words.title);
            }
            const live = make(view(d.t, "listening"), d.t);
            verify(!!label(live, by("listening").bar.lang.name), "the language's chip");
            verify(!!label(live, "0:01"), "the timer, from 1.2 s in");
        }

        function test_stop_is_dark_and_stays_in_place_dimmed_while_it_writes() {
            const bar = make(view("en", "transcribing"));
            const stop = withId(bar, "voice.stop");
            verify(!stop.enabled, "dimmed");
            compare(stop.opacity, N.lk.shape.panel.off_share);
            verify(Qt.colorEqual(stop.fill, N.lk.palette.pop_ink), "drawn dark (owner, 2026-11-14)");
            verify(Qt.colorEqual(stop.ink, N.lk.palette.pop_bg), "its icon in the panel colour");
            mouseClick(stop);
            compare(bar.voice.calls, [], "a dimmed Stop sends nothing");
        }

        // Arabic flips text only (ADR-0020): Cancel stays left and Stop right; the text reads right to left.
        function test_arabic_keeps_cancel_left_and_stop_right_and_reads_its_text_right_to_left() {
            const bar = make(view("ar", "listening"), "ar");
            const all = buttons(bar);
            compare(all[0].Accessible.id, "voice.cancel");
            compare(all[all.length - 1].Accessible.id, "voice.stop");
            const text = make(view("ar", "text"), "ar");
            const words = note(text).words;
            compare(words.effectiveHorizontalAlignment, Text.AlignRight, "right to left");
            compare(words.font.family, text.res.fam.arabic, "the theme's Arabic font");
            verify(withId(text, "voice.close") === buttons(text)[buttons(text).length - 1], "Close at the right end");
        }

        function test_a_long_text_shows_its_first_lines_data() {
            return ["en", "ar"].map(t => ({ tag: t, t: t }));
        }
        function test_a_long_text_shows_its_first_lines(d) {
            const bar = make(view(d.t, "long"), d.t);
            const words = note(bar).words;
            verify(words.truncated, "cut");
            compare(words.lineCount, N.lk.shape.voice.text_lines, "its first lines");
            compare(bar.width, N.lk.shape.voice.bar_px[1], "the text's width");
            const short = make(view(d.t, "listening"), d.t);
            compare(short.width, N.lk.shape.voice.bar_px[0]);
        }

        // Each line a text can carry shows under it, and Copy goes on after a refused copy.
        function test_a_texts_lines_show_under_it() {
            for (const name of ["stopped", "copyFailed", "password"]) {
                const v = view("fr", name);
                const bar = make(v, "fr");
                compare(note(bar).hint.text, v.bar.hint.text, name);
                verify(note(bar).hint.visible, name);
            }
            verify(!!withId(make(view("en", "copyFailed")), "voice.copy"), "Copy may be tried again");
            compare(note(make(view("en", "password"))).words.text, Voice.views.en.password.bar.text.text, "dots");
        }

        // A note's line takes the status weight, as the mock-up's .msg keeps .vbar's; a text is plain (.vtx).
        function test_a_notes_line_takes_the_status_weight_and_a_text_is_plain() {
            const line = note(make(view("en", "fault"))).words;
            compare(line.font.weight, N.lk.shape.caption.status_weight);
            compare(note(make(view("en", "text"))).words.font.weight, Font.Normal);
        }

        function test_every_icon_loads_in_its_tint_data() {
            return [{ tag: "light", lk: N.lk }, { tag: "contrast", lk: H.lk }];
        }
        function test_every_icon_loads_in_its_tint(d) {
            for (const name of ["listening", "transcribing", "text", "password", "noModel", "checking", "cancelled"]) {
                const bar = make(view("en", name), "en", d.lk);
                const icons = Board.all(bar.contentItem, o => o.base !== undefined && o.px !== undefined && o.visible);
                verify(icons.length > 0, name);
                for (const i of icons) {
                    verify(i.icon.length > 0, name + ": an icon name");
                    tryCompare(i, "status", Image.Ready);
                }
            }
        }

        // The microphone listening shows the red dot, even beside a hint or a note.
        function test_a_note_never_hides_the_red_dot() {
            verify(!!dot(make(view("en", "listening"))), "listening");
            const nothing = make(view("en", "nothing"));
            verify(!!dot(nothing), "beside Nothing heard yet");
            verify(!!label(nothing, Voice.views.en.nothing.bar.hint), "the hint shows");
            const checking = make(view("en", "checking"));
            verify(checking.view.bar.mic, "the microphone is open");
            verify(!!dot(checking), "beside a note");
            compare(dot(make(view("en", "heardNothing"))), undefined, "no dot once it stopped");
            compare(dot(make(view("en", "text"))), undefined);
        }

        function test_copy_turns_to_copied_until_the_next_view() {
            const v = view("en", "text");
            const bar = make(v);
            mouseClick(withId(bar, "voice.copy"));
            compare(bar.voice.calls, ["act bar copy 0 " + v.round]);
            compare(withId(bar, "voice.copy").Accessible.name, v.words.copied);
            bar.voice.page = view("en", "undone", { round: v.round + 1 });
            compare(withId(bar, "voice.copy").Accessible.name, v.bar.buttons[0].label, "a new view");
        }

        // It never takes focus, so the app in front keeps its caret; each button acts by name and id at let-go.
        function test_the_window_takes_clicks_but_never_focus() {
            const v = view("en", "listening");
            const bar = make(v);
            verify(bar.flags & Qt.WindowDoesNotAcceptFocus, "never takes focus");
            mouseClick(withId(bar, "voice.stop"));
            withId(bar, "voice.cancel").Accessible.pressAction();
            mouseClick(withId(bar, "voice.gear"));
            const r = v.round;
            compare(bar.voice.calls, ["act bar stop 0 " + r, "act bar cancel 0 " + r, "act gear gear 0 " + r]);
        }

        function test_a_hidden_bar_shows_nothing() {
            const bar = make(view("en", "hidden"));
            verify(!bar.visible);
            bar.voice.page = view("en", "listening");
            tryVerify(() => bar.visible, 2000, "a view shows it");
            bar.voice.page = view("en", "hidden");
            verify(!bar.visible, "and hides it");
        }
    }
}
