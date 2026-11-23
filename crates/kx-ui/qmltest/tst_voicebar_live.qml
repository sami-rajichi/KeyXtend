// The dictation bar as it lives over the stand-in Voice object: the wave follows the smoothed level and ripples, reduced
// motion draws a plain meter instead, the timer counts on at one width, the start and stop sounds play as it
// starts and stops listening unless sounds are off, the pointer coming over the bar and leaving is told once each, and
// the bar is placed where the hub says by the keyboard's box: above it, at the bottom or top for those places, at the
// bottom centre while the keyboard hides (edge 15, Stop still acting), and it stays while the panel is open.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/voice.js" as Voice
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-reduced.js" as R

Item {
    id: root
    width: 200
    height: 100

    // The keyboard's box the app hands the bar, or "null" while the keyboard hides; and the bar's clock in ms.
    property string over: JSON.stringify({ x: 100, y: 600, w: 900, h: 300 })
    property real clock: 1000

    Component { id: voiceC; FakeVoice {} }
    Component { id: barC; VoiceBar {} }

    TestCase {
        name: "voicebar_live"
        when: windowShown

        // Any warning fails the test, so the run stays clean; each test starts by the keyboard.
        function init() {
            failOnWarning(/./);
            root.over = JSON.stringify({ x: 100, y: 600, w: 900, h: 300 });
            root.clock = 1000;
        }
        // A copy of the hub's English view `name`, with `more` laid over it.
        function view(name, more) {
            return Object.assign(JSON.parse(JSON.stringify(Voice.views.en[name])), more || {});
        }
        // A bar over a stand-in Voice object showing `v`, in look `lk` (Native light by default), on the root's clock.
        function make(v, lk) {
            const look = lk || N.lk;
            const voice = createTemporaryObject(voiceC, root, { page: v });
            const bar = createTemporaryObject(barC, root, {
                voice: voice, lk: look, res: Board.resOf(look, En), s: 1, ar: false,
                box: () => root.over, logical: (x, y) => ({ x: x, y: y }), now: () => root.clock
            });
            tryVerify(() => bar.visible, 2000, "it shows");
            return bar;
        }
        // The wave's bars' heights.
        function heights(bar) {
            return Board.all(bar.wave, o => o.bar !== undefined).map(o => o.height);
        }
        function withId(bar, uid) {
            return Board.all(bar.contentItem, o => o.Accessible.role === Accessible.Button && o.Accessible.id === uid)[0];
        }
        // The level `level` comes after `ms` on the clock.
        function hear(bar, level, ms) {
            root.clock += ms;
            bar.voice.heard = level;
        }

        // The wave smooths the level over `waveMs`: at rest with silence, taller with speech, and back.
        function test_the_wave_follows_the_level() {
            const bar = make(view("listening"));
            const rest = N.lk.shape.voice.wave_px[2];
            const full = N.lk.shape.voice.wave_px[3];
            verify(bar.wave.wave.visible && !bar.wave.meter.visible, "a wave, not a meter");
            verify(heights(bar).length > 1, "bars across the room left");
            verify(heights(bar).every(h => h === rest), "at rest in silence");
            hear(bar, 1, 1);
            const soon = Math.max(...heights(bar));
            verify(soon > rest && soon < full, "a sudden level eases in: " + soon);
            hear(bar, 0.99, 100 * bar.view.bar.waveMs);
            verify(Math.max(...heights(bar)) > soon, "and rises to it");
            verify(heights(bar).every(h => h <= full), "never past full");
            hear(bar, 0, 100 * bar.view.bar.waveMs);
            verify(heights(bar).every(h => Math.abs(h - rest) < 0.01), "back at rest");
            compare(bar.voice.reads, 3, "one read per level");
        }

        function test_the_wave_ripples_across_its_bars_while_it_listens() {
            const bar = make(view("listening"));
            verify(bar.wave.rippling, "it ripples");
            hear(bar, 1, 100 * bar.view.bar.waveMs);
            const before = heights(bar);
            bar.wave.phase += 1;
            verify(JSON.stringify(heights(bar)) !== JSON.stringify(before), "the ripple moves the bars");
            const differ = new Set(before.map(h => h.toFixed(3))).size > 1;
            verify(differ, "each bar its own height");
            bar.voice.page = view("transcribing");
            verify(!bar.wave.rippling, "it stops once it stops listening");
        }

        function test_reduced_motion_draws_a_plain_meter() {
            const bar = make(view("listening"), R.lk);
            verify(bar.wave.meter.visible && !bar.wave.wave.visible, "a meter, not a wave");
            verify(!bar.wave.rippling, "nothing ripples");
            hear(bar, 0.5, 100 * bar.view.bar.waveMs);
            compare(bar.wave.fill.width, bar.wave.meter.width * 0.5, "filled to the level");
            const shim = make(view("transcribing"), R.lk);
            verify(!Board.all(shim.contentItem, o => o.running !== undefined && o.span !== undefined)[0].running, "a still shimmer");
        }

        function test_the_timer_counts_on_from_the_modules_time() {
            const bar = make(view("listening"));
            const timer = Board.all(bar.contentItem, o => o.elapsed !== undefined && o.font !== undefined)[0];
            compare(timer.text, "0:01", "1.2 s in");
            root.clock += 60 * 1000;
            timer.tick.triggered();
            compare(timer.text, "1:01");
            compare(timer.tick.interval, 800, "the next whole second");
        }

        // The minutes are measured two digits wide, so ten minutes in widens nothing.
        function test_the_timer_holds_its_width_past_ten_minutes() {
            const bar = make(view("listening"));
            const timer = Board.all(bar.contentItem, o => o.elapsed !== undefined && o.font !== undefined)[0];
            const width = timer.width;
            root.clock += 10 * 60 * 1000;
            timer.tick.triggered();
            compare(timer.text, "10:01");
            compare(timer.width, width, "the same width");
        }

        // The minutes and seconds go where the language's words put them.
        function test_the_timer_takes_its_order_from_the_words() {
            const v = view("listening");
            v.words.timer = "%2 · %1";
            const timer = Board.all(make(v).contentItem, o => o.elapsed !== undefined && o.font !== undefined)[0];
            compare(timer.text, "01 · 0");
        }

        function test_the_sounds_play_as_it_starts_and_stops_listening() {
            const voice = createTemporaryObject(voiceC, root, { page: view("hidden") });
            const live = createTemporaryObject(barC, root, {
                voice: voice, lk: N.lk, res: Board.resOf(N.lk, En), s: 1, ar: false,
                box: () => root.over, logical: (x, y) => ({ x: x, y: y })
            });
            voice.page = view("listening");
            voice.page = view("listening", { round: 99 });
            voice.page = view("transcribing");
            voice.page = view("text");
            compare(voice.calls, ["chime true", "chime false"], "once each");
            voice.calls = [];
            voice.page = view("listening", { sounds: false });
            voice.page = view("transcribing", { sounds: false });
            compare(voice.calls, [], "sounds off plays none (edge 40)");
            verify(live.visible);
        }

        // The pointer coming over the bar and leaving it is told once each; a bar that hides forgets it, as the hub
        // tells the module itself.
        function test_the_pointer_over_the_bar_is_told_once_each_way() {
            const v = view("listening");
            const bar = make(v);
            mouseMove(bar.body, 10, 10);
            mouseMove(bar.body, 20, 10);
            tryCompare(bar, "over", true);
            // The pointer goes off the bar.
            mouseMove(bar.body, bar.body.width + 10, bar.body.height / 2);
            tryCompare(bar, "over", false);
            compare(bar.voice.calls, ["act over  1 " + v.round, "act over  0 " + v.round]);
            mouseMove(bar.body, 10, 10);
            tryCompare(bar, "over", true);
            bar.voice.page = view("hidden");
            compare(bar.voice.calls.length, 3, "a hide sends nothing");
        }

        // The hub places the bar by the keyboard's box and its size; a new place moves it (edge 40).
        function test_it_goes_where_the_hub_places_it() {
            const bar = make(view("listening"));
            const ask = bar.voice.asks[bar.voice.asks.length - 1];
            compare(JSON.stringify(ask.over), root.over, "by the keyboard");
            compare([ask.w, ask.h], [bar.width, bar.height], "its own size");
            compare([bar.x, bar.y], [300, 500 - bar.height], "above it, by its bottom edge");
            bar.voice.page = view("listening", { place: "bottom" });
            compare([bar.x, bar.y], [300, 900 - bar.height], "the bottom centre");
            bar.voice.page = view("listening", { place: "top" });
            compare([bar.x, bar.y], [300, 10], "the top centre");
            bar.voice.page = view("text");
            const wide = bar.voice.asks[bar.voice.asks.length - 1];
            compare(wide.w, N.lk.shape.voice.bar_px[1], "placed again at its new width");
        }

        // Edge 15: the keyboard hides while it listens; the bar stays, at the bottom centre, and Stop still acts.
        function test_it_stays_when_the_keyboard_hides_and_stop_still_acts() {
            const v = view("listening");
            const bar = make(v);
            root.over = "null";
            bar.follow();
            verify(bar.visible, "it stays");
            compare(bar.voice.asks[bar.voice.asks.length - 1].over, null, "with no keyboard");
            compare([bar.x, bar.y], [300, 900 - bar.height], "at the bottom centre");
            mouseClick(withId(bar, "voice.stop"));
            compare(bar.voice.calls, ["act bar stop 0 " + v.round]);
        }

        function test_it_stays_while_the_panel_is_open() {
            const bar = make(view("listening"));
            bar.voice.page = view("listening", { round: 50, panel: { lang: { name: "English", rtl: false, arabic: false } } });
            verify(bar.visible, "it stays below the panel");
            bar.voice.page = view("text", { round: 51, panel: { lang: { name: "English", rtl: false, arabic: false } } });
            verify(bar.visible);
        }
    }
}
