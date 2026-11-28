// Snip's video windows over the stand-in Snip object (design/snip-parts.css .count, .soon, .edge, .recbar): the
// countdown with its seconds and Cancel, the dashed then red edge round the part, the record bar with its time,
// switches, Stop and Cancel in English, French and Arabic, the dot that stops pulsing with reduced motion and takes
// Highlight in high contrast (edge 23), a switch under the pointer in sel_ink, windows a refused shield keeps hidden,
// and no press here that aims the letters.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "snipkit.js" as Kit
import "fixtures/snip.js" as Snip
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-contrast.js" as H
import "../../kx-theme/fixtures/look-native-light-reduced.js" as R

Item {
    id: root
    width: 200
    height: 100

    Component { id: snipC; FakeSnip {} }
    Component { id: appC; SnipApp {} }

    TestCase {
        name: "sniprec"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` (with `more` laid over it) in language `tag` and look `lk`.
        function make(name, tag, lk, more) {
            const snip = createTemporaryObject(snipC, root);
            const look = lk || N.lk;
            const aims = [];
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: look, res: Kit.resOf(look, tag || "en"), aim: () => aims.push(true)
            });
            snip.page = Kit.view(name, more);
            tryVerify(() => app.shown === app.stage, 2000, "the stage shows");
            return { snip: snip, app: app, aims: aims };
        }
        // The part edge's four sides.
        function sides(app) {
            return [app.edge.top, app.edge.bottom, app.edge.left, app.edge.right];
        }

        function test_the_countdown_shows_the_seconds_and_cancel_in_each_language() {
            for (const tag of Kit.langs) {
                const s = make("count", tag);
                const win = s.app.count;
                tryVerify(() => win.visible, 2000, tag);
                waitForRendering(win.contentItem);
                compare(win.title, Kit.text(tag, "window.snip-count"));
                const disc = Board.all(win.contentItem, o => o.Accessible.role === Accessible.StaticText)[0];
                compare(disc.Accessible.name, Kit.text(tag, "snip.count") + Kit.text(tag, "snip.join") + "3");
                verify(Kit.texts(win.contentItem, "3").length === 1, "the seconds left");
                const part = s.app.box(Snip.views.count.count.part);
                compare(win.x, Math.round(part.x + (part.width - win.width) / 2), "centred on the part");
                const cancel = Kit.buttons(win)[0];
                compare(cancel.Accessible.name, Kit.text(tag, "snip.rec-cancel"));
                mouseClick(cancel);
                compare(s.snip.heard("act"), [{ act: "rec", id: "cancel", arg: 0, round: Snip.views.count.round }]);
                compare(s.aims.length, 0, "the countdown never aims the letters");
            }
        }

        function test_the_edge_dashes_round_the_part_then_turns_red() {
            const s = make("count");
            const sh = N.lk.shape.snip;
            const part = s.app.box(Snip.views.count.count.part);
            tryVerify(() => sides(s.app).every(w => w.visible), 2000, "four sides");
            verify(sides(s.app).every(w => !w.rec && w.title === Kit.text("en", "window.snip-edge")));
            verify(sides(s.app).every(w => w.flags & Qt.WindowTransparentForInput), "every click goes through");
            compare([s.app.edge.top.y, s.app.edge.top.height], [Math.round(part.y - sh.soon_px), Math.round(sh.soon_px)]);
            compare(s.app.edge.top.near, "bottom", "the line lies by the part");
            s.snip.page = Kit.view("record");
            tryVerify(() => sides(s.app).every(w => w.rec && w.visible), 2000, "red while it records");
            compare(s.app.edge.left.width, Math.round(sh.edge_px + sh.glow_px), "the line and its glow");
            compare(s.app.edge.left.x, Math.round(part.x - sh.edge_px - sh.glow_px), "just outside the part");
        }

        function test_a_whole_screen_keeps_its_edge_inside_the_screen() {
            const s = make("top");
            tryVerify(() => s.app.edge.top.visible);
            const area = s.app.box(Snip.views.top.record.part);
            compare(s.app.edge.top.y, Math.round(area.y), "inside, at the screen's top");
            compare(s.app.edge.top.near, "top");
        }

        function test_the_record_bar_shows_the_time_switches_stop_and_cancel_in_each_language() {
            for (const tag of Kit.langs) {
                const s = make("record", tag);
                const win = s.app.record.bar;
                tryVerify(() => win.visible, 2000, tag);
                waitForRendering(win.contentItem);
                const ids = ["mic", "sound", "stop", "cancel"];
                compare(Kit.buttons(win).map(o => o.Accessible.name), ids.map(id => Kit.text(tag, "snip.rec-" + id)), tag);
                compare(Kit.buttons(win).map(o => o.Accessible.id), ids.map(id => "snip.rec." + id));
                const [mic, sound, stop] = Kit.buttons(win);
                verify(mic.Accessible.checked && !sound.Accessible.checked, "the mic records, the sound does not");
                compare(sound.icon, N.lk.icons.snip.sound_off);
                verify(Kit.texts(win.contentItem, Kit.text(tag, "snip.rec-stop")).length === 1, "Stop says its word");
                verify(Kit.texts(win.contentItem, "1:05").length === 1, "the time, minutes then seconds");
                const live = Kit.text(tag, "snip.live") + Kit.text(tag, "snip.join") + "1:05";
                verify(Board.all(win.contentItem, o => o.Accessible.name === live).length === 1, "readers hear the time");
                const sh = N.lk.shape.snip;
                const seps = Board.all(win.contentItem, o => o.visible && o.height === sh.rec_px - 2 * sh.rec_sep_px);
                compare(seps.length, 2, "a line before the switches and before Stop, clear of the bar's top and bottom");
                compare(win.title, Kit.text(tag, "window.snip-record"));
                const part = s.app.box(Snip.views.record.record.part);
                compare(win.y, Math.round(part.y - N.lk.shape.snip.gap_px - win.height), "above the part");
                mouseClick(stop);
                compare(s.snip.heard("act"), [{ act: "rec", id: "stop", arg: 0, round: Snip.views.record.round }]);
                compare(s.aims.length, 0, "the record bar never aims the letters");
            }
        }

        function test_a_refused_mic_and_no_sound_device_fade_say_why_and_never_act() {
            for (const tag of Kit.langs) {
                const s = make("rec-refused", tag);
                const win = s.app.record.bar;
                tryVerify(() => win.visible, 2000, tag);
                waitForRendering(win.contentItem);
                const [mic, sound, stop] = Kit.buttons(win);
                for (const [k, why] of [[mic, "snip.dim-mic-refused"], [sound, "snip.dim-no-sound"]]) {
                    compare(k.Accessible.description, Kit.text(tag, why), tag);
                    compare(k.says, Kit.text(tag, "snip.rec-" + k.key) + "\n" + Kit.text(tag, why));
                    const row = Kit.named(k, "row");
                    compare(row.opacity, N.lk.shape.panel.off_share, "faded as the panels' dimmed buttons");
                    mouseClick(k);
                }
                compare(s.snip.heard("act"), [], tag + ": a dimmed switch never acts");
                compare(Kit.named(stop, "row").opacity, 1, "Stop stays clear");
                mouseClick(stop);
                compare(s.snip.heard("act").map(a => a.id), ["stop"], "and acts in the same bar");
            }
        }

        function test_the_edge_closes_its_corners_and_leaves_the_top_under_a_dialog() {
            for (const name of ["count", "record"]) {
                const s = make(name);
                tryVerify(() => sides(s.app).every(w => w.visible), 2000, name);
                const [top, bottom, left, right] = sides(s.app);
                for (const w of [top, bottom])
                    compare([w.x, w.x + w.width], [left.x, right.x + right.width], name + ": the ring closes");
                compare(top.line.x, top.at.lead - top.px, name + ": the line reaches the side's line, the glow the rest");
                verify(sides(s.app).every(w => w.flags & Qt.WindowStaysOnTopHint), name + ": on top");
            }
            const low = make("record", "en", N.lk, { low: true });
            tryVerify(() => low.app.edge.top.visible);
            verify(sides(low.app).every(w => !(w.flags & Qt.WindowStaysOnTopHint)), "a dialog takes the top");
        }

        function test_a_note_in_or_right_after_a_countdown_shows_on_the_parts_screen() {
            const screens = [
                { virtualX: 0, virtualY: 0, width: 800, height: 600, devicePixelRatio: 1 },
                { virtualX: 800, virtualY: 0, width: 640, height: 480, devicePixelRatio: 1.5 }
            ];
            const snip = createTemporaryObject(snipC, root);
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: N.lk, res: Kit.resOf(N.lk, "en"), aim: () => {}, screens: screens
            });
            const part = { x: 900, y: 100, w: 300, h: 200 };
            snip.page = Kit.view("count", { note: "bare", count: { part: part, left_s: 3 } });
            tryVerify(() => app.hint.visible, 2000, "the note shows");
            const flat = r => [r.x, r.y, r.width, r.height];
            const there = flat(app.box({ x: 800, y: 0, w: 960, h: 720 }));
            compare(there, [800, 0, 640, 480], "the second screen, in Qt units");
            compare(flat(app.hint.screenBox), there, "on the part's screen, not the first");
            snip.page = Kit.view("idle", { note: "bare" });
            tryVerify(() => app.hint.visible, 2000, "the note stays once the countdown ends");
            compare(flat(app.hint.screenBox), there, "still where the part was");
        }

        function test_the_time_restarts_only_on_a_new_time_or_a_new_recording() {
            const s = make("record");
            const rec = s.app.record;
            const at = rec.heard;
            const timed = (round, ms) => Object.assign(Kit.view("record", { round: round }), {
                record: Object.assign({}, Snip.views.record.record, { elapsed_ms: ms })
            });
            rec.now = () => at + 5000;
            s.snip.page = timed(97, Snip.views.record.record.elapsed_ms);
            compare(rec.heard, at, "a view for another reason keeps the clock counting");
            s.snip.page = timed(98, 70000);
            compare([rec.heard, rec.elapsed], [at + 5000, 70000], "a new time sets it");
            s.snip.page = Kit.view("idle");
            rec.now = () => at + 9000;
            s.snip.page = timed(99, 70000);
            compare(rec.heard, at + 9000, "a new recording starts it afresh");
        }

        function test_the_clock_holds_at_nought_until_the_first_frame_and_never_steps_back() {
            const s = make("record");
            const rec = s.app.record;
            const t0 = rec.heard;
            const view = (round, live, ms) => Object.assign(Kit.view("record", { round: round }), {
                record: Object.assign({}, Snip.views.record.record, { live: live, elapsed_ms: ms })
            });
            const timer = () => Board.all(rec.bar.contentItem, o => o.tick !== undefined)[0];
            const said = () => timer().Accessible.name;
            const at = time => Kit.text("en", "snip.live") + Kit.text("en", "snip.join") + time;
            s.snip.page = Kit.view("idle");
            rec.now = () => t0;
            s.snip.page = view(90, false, 0);
            tryVerify(() => rec.bar.visible);
            rec.now = () => t0 + 2500;
            timer().count();
            compare(said(), at("0:00"), "held while the worker starts");
            s.snip.page = view(91, true, 0);
            compare(said(), at("0:00"), "the first frame starts it at nought");
            rec.now = () => t0 + 3400;
            timer().count();
            compare(said(), at("0:00"), "under a second since the first frame");
            rec.now = () => t0 + 3600;
            s.snip.page = view(92, true, 1000);
            compare(said(), at("0:01"), "the recorder's second, never a step back");
        }

        function test_with_no_room_above_the_record_bar_sits_at_the_top() {
            const s = make("top");
            const win = s.app.record.bar;
            tryVerify(() => win.visible);
            const area = s.app.box(Snip.views.top.record.screen);
            compare(win.y, Math.round(area.y + N.lk.shape.snip.bar_top_px));
        }

        function test_the_dot_pulses_stops_with_reduced_motion_and_takes_highlight_in_high_contrast() {
            const dot = lk => {
                const s = make("record", "en", lk);
                tryVerify(() => s.app.record.bar.visible);
                return Board.all(s.app.record.bar.contentItem, o => o.pulsing !== undefined)[0];
            };
            verify(dot(N.lk).pulsing, "it pulses");
            verify(!dot(R.lk).pulsing, "reduced motion stops it");
            verify(Qt.colorEqual(dot(H.lk).color, H.lk.palette.ring), "Highlight in high contrast");
            compare(dot(N.lk).width, N.lk.shape.snip.rec_dot_px);
        }

        function test_a_switch_under_the_pointer_takes_sel_ink_in_high_contrast() {
            const s = make("record", "en", H.lk);
            const win = s.app.record.bar;
            tryVerify(() => win.visible);
            waitForRendering(win.contentItem);
            const [mic, sound] = Kit.buttons(win);
            verify(Qt.colorEqual(mic.ink, H.lk.palette.ring), "a recording mic takes Highlight");
            mouseMove(mic);
            tryVerify(() => Qt.colorEqual(mic.ink, H.lk.sel_ink), 2000, "not Highlight on Highlight");
            mouseMove(sound);
            tryVerify(() => Qt.colorEqual(sound.ink, H.lk.sel_ink), 2000, "nor the muted ink");
            verify(Qt.colorEqual(mic.ink, H.lk.palette.ring), "the mic is back once the pointer leaves");
        }

        function test_a_refused_shield_keeps_its_window_hidden() {
            const s = make("record", "en", N.lk, { hide: ["record", "edge"] });
            verify(!s.app.record.bar.visible && !s.app.record.visible, "no record bar nor its shadow");
            verify(sides(s.app).every(w => !w.visible), "no edge");
            const c = make("count", "en", N.lk, { hide: ["count"] });
            verify(!c.app.count.visible, "no countdown");
        }

        function test_no_video_window_takes_focus() {
            const s = make("record");
            const wins = [s.app.record, s.app.record.bar, s.app.count].concat(sides(s.app));
            verify(wins.every(w => w.flags & Qt.WindowDoesNotAcceptFocus));
        }
    }
}
