// Snip's strip over the stand-in Snip object (design/snip-parts.css .strip): Copy, Edit, Save and Close under the part
// in English, French and Arabic, the tick and done word once copied, a recorded video's length with Play, Speed, Copy,
// Save and Close (ADR-0060), Play telling readers it opens their own player, Record with the Mic and Sound switches
// before a video, a dimmed switch saying why (edge 11), a refused player's note (edge 8), a press that aims the letters
// at Snip and acts, and a key that waits for its copy at a speed showing and saying so.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "snipkit.js" as Kit
import "fixtures/snip.js" as Snip
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-reduced.js" as R

Item {
    id: root
    width: 200
    height: 100

    Component { id: snipC; FakeSnip {} }
    Component { id: appC; SnipApp {} }

    TestCase {
        name: "snipstrip"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` in language `tag` and look `lk`, once the strip shows;
        // aims counted.
        function make(name, tag, lk) {
            const snip = createTemporaryObject(snipC, root);
            const aims = [];
            const look = lk || N.lk;
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: look, res: Kit.resOf(look, tag || "en"), aim: () => aims.push(true)
            });
            snip.page = Kit.view(name);
            tryVerify(() => app.strip.strip.visible, 2000, "the strip shows");
            waitForRendering(app.strip.strip.contentItem);
            return { snip: snip, app: app, aims: aims, win: app.strip.strip };
        }
        // The words view `name`'s keys say in language `tag`.
        function words(name, tag) {
            return Snip.views[name].strip.keys.map(k => Kit.text(tag, "snip.strip-" + k.key));
        }

        function test_the_picture_strip_names_its_keys_under_the_part_in_each_language() {
            for (const tag of Kit.langs) {
                const s = make("strip", tag);
                compare(Kit.buttons(s.win).map(o => o.Accessible.name), words("strip", tag), tag);
                compare(Kit.buttons(s.win).map(o => o.Accessible.id), ["copy", "edit", "save", "close"].map(id => "snip.strip." + id));
                compare(s.win.title, Kit.text(tag, "window.snip-strip"));
                compare(s.app.strip.title, s.win.title, "its shadow shares its title");
                const [part, area] = [Snip.views.strip.strip.part, Snip.views.strip.strip.screen].map(r => s.app.box(r));
                compare(s.win.y, Math.round(part.y + part.height + N.lk.shape.snip.gap_px), "under the part");
                const centred = part.x + (part.width - s.win.width) / 2;
                compare(s.win.x, Math.round(Math.max(area.x, centred)), "centred on it, kept on its screen");
            }
        }

        function test_a_copy_shows_its_tick_and_done_word() {
            const s = make("copied", "fr");
            const copy = Kit.button(s.win, "copy");
            compare(copy.Accessible.name, Kit.text("fr", "snip.done-copy"));
            compare(copy.icon, N.lk.icons.snip.tick);
            compare(Kit.button(s.win, "save").icon, N.lk.icons.snip.strip.save, "the others keep theirs");
        }

        function test_a_videos_strip_shows_its_length_first() {
            const s = make("length");
            const len = Board.all(s.win.contentItem, o => o.Accessible.role === Accessible.StaticText && o.visible)[0];
            compare(len.Accessible.name, Kit.text("en", "snip.length") + Kit.text("en", "snip.join") + "1:15");
            verify(Kit.texts(s.win.contentItem, "1:15").length === 1, "the length reads minutes then seconds");
            verify(len.mapToItem(null, 0, 0).x < Kit.buttons(s.win)[0].mapToItem(null, 0, 0).x, "before the keys");
        }

        function test_a_recorded_videos_strip_plays_and_steps_its_speed_with_no_edit_in_each_language() {
            for (const tag of Kit.langs) {
                const s = make("length", tag);
                const ids = ["play", "speed", "copy", "save", "close"];
                compare(Kit.buttons(s.win).map(o => o.key), ids, tag);
                const [play, speed] = ["play", "speed"].map(id => Kit.button(s.win, id));
                compare(play.Accessible.name, Kit.text(tag, "snip.strip-play"));
                const say = Kit.text(tag, "snip.strip-play-say");
                compare(play.Accessible.description, say, "Play says it opens the user's own player");
                compare(speed.Accessible.description, "", "no other key says it");
                compare(play.icon, N.lk.icons.snip.strip.play);
                compare(speed.icon, N.lk.icons.snip.strip.speed);
                const times = Kit.text(tag, "snip.speed-times").replace("%1", 1);
                compare(speed.Accessible.name, Kit.text(tag, "snip.strip-speed") + Kit.text(tag, "snip.join") + times);
                compare(Kit.texts(s.win.contentItem, times).length, 1, "its speed after its word");
                mouseClick(play);
                mouseClick(speed);
                const r = Snip.views.length.round;
                compare(s.snip.heard("act"), [{ act: "strip", id: "play", arg: 0, round: r }, { act: "strip", id: "speed", arg: 0, round: r }]);
            }
        }

        function test_speed_shows_the_speed_picked_and_the_length_at_it() {
            const s = make("sped");
            const times = Kit.text("en", "snip.speed-times").replace("%1", 2);
            compare(Kit.texts(s.win.contentItem, times).length, 1, "2× on the Speed key");
            compare(Kit.texts(s.win.contentItem, "0:37").length, 1, "the length at 2×");
            const key = Kit.button(s.win, "speed");
            verify(!key.Accessible.checkable, "one key that steps, not a switch");
        }

        function test_edge_8_a_refused_player_says_so_while_the_strip_stays_on_top() {
            for (const tag of Kit.langs) {
                const s = make("play-failed", tag);
                tryCompare(s.app.hint, "said", Kit.text(tag, "snip.note-play-failed"));
                tryVerify(() => s.app.hint.visible, 2000, "the note shows by the strip");
                verify(s.win.visible && s.win.flags & Qt.WindowStaysOnTopHint, "the strip stays on top");
            }
        }

        function test_before_a_video_record_leads_and_a_missing_mic_dims_with_a_tip() {
            for (const tag of Kit.langs) {
                const s = make("ready", tag);
                compare(Kit.buttons(s.win).map(o => o.Accessible.name), words("ready", tag), tag);
                const [rec, mic, sound] = ["record", "mic", "sound"].map(id => Kit.button(s.win, id));
                verify(Qt.colorEqual(rec.fill, N.lk.palette.enter), "Record is the Enter key");
                verify(!rec.lit, "no red light, as none reaches 3:1 on Enter's colours");
                const why = Kit.text(tag, "snip.dim-no-mic");
                compare(mic.Accessible.description, why);
                compare(mic.says, Kit.text(tag, "snip.strip-mic") + "\n" + why);
                compare(mic.icon, N.lk.icons.snip.mic_off);
                verify(mic.Accessible.checkable && !mic.Accessible.checked, "the mic switch is off");
                verify(sound.Accessible.checked, "the sound switch is on");
                compare(sound.icon, N.lk.icons.snip.strip.sound);
                mouseClick(mic);
                compare(s.snip.heard("act"), [], "a dimmed switch never acts");
                mouseClick(sound);
                compare(s.snip.heard("act"), [{ act: "strip", id: "sound", arg: 0, round: Snip.views.ready.round }]);
            }
        }

        function test_a_refused_mic_no_sound_device_and_a_full_drive_each_say_why() {
            for (const tag of Kit.langs) {
                const s = make("refused", tag);
                const said = id => Kit.button(s.win, id).Accessible.description;
                compare(said("mic"), Kit.text(tag, "snip.dim-mic-refused"), tag);
                compare(said("sound"), Kit.text(tag, "snip.dim-no-sound"));
                compare(said("record"), Kit.text(tag, "snip.dim-disk-low"));
                tryCompare(s.app.hint, "said", Kit.text(tag, "snip.note-disk-low"));
                tryVerify(() => s.app.hint.visible, 2000, "the note shows by the strip");
            }
        }

        function test_a_press_on_the_strip_aims_the_letters_and_acts() {
            const s = make("strip");
            mouseClick(Kit.button(s.win, "edit"));
            compare(s.aims.length, 1, "the press aims the letters at Snip");
            compare(s.snip.heard("act"), [{ act: "strip", id: "edit", arg: 0, round: Snip.views.strip.round }]);
            verify(!(s.win.flags & Qt.WindowTransparentForInput) && s.win.flags & Qt.WindowDoesNotAcceptFocus);
        }

        // While Play's copy at 2× is written, Play alone shows it waits with a line sliding along its foot, and says
        // so to readers and in its tip in each language; the line goes once the copy is written.
        function test_a_key_waiting_for_its_copy_at_a_speed_shows_and_says_it_waits() {
            for (const tag of Kit.langs) {
                const s = make("waiting", tag);
                const [play, copy] = ["play", "copy"].map(id => Kit.button(s.win, id));
                const wait = Kit.text(tag, "snip.strip-wait");
                verify(play.cue.visible && !copy.cue.visible, tag + ": only Play waits");
                compare([play.Accessible.description, play.says], [wait, wait], tag);
                const part = play.cue.children[0];
                const x = part.x;
                tryVerify(() => part.x !== x, 2000, "its line slides");
                s.snip.page = Kit.view("sped");
                tryVerify(() => !Kit.button(s.win, "play").cue.visible, 2000, "gone once written");
            }
        }

        // A tip over Play follows its words as it starts waiting for its copy, and goes once the copy is written.
        function test_a_tip_over_play_follows_it_as_it_waits_and_once_its_copy_is_written() {
            const s = make("length");
            const tips = Board.tipOf(s.win);
            mouseMove(Kit.button(s.win, "play"));
            s.snip.page = Kit.view("waiting");
            tryCompare(tips, "name", Kit.text("en", "snip.strip-wait"));
            s.snip.page = Kit.view("sped");
            tryCompare(tips, "name", "");
        }

        // Reduced motion draws the waiting line whole, dimmed and still (voice-bar.css .still .prog.run).
        function test_with_reduced_motion_the_waiting_line_stands_still() {
            const s = make("waiting", "en", R.lk);
            const cue = Kit.button(s.win, "play").cue;
            const part = cue.children[0];
            verify(cue.visible);
            compare([part.x, part.width, part.opacity], [0, cue.width, R.lk.shape.voice.run_share[1]]);
        }
    }
}
