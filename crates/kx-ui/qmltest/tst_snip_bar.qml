// Snip's picking windows over the stand-in Snip object (design "What the user sees"): the click kinds and capture
// windows are the hub's, each capture window is shielded with its title before it shows, a snip sends the theme's sizes
// first, the capture bar names its keys in English, French and Arabic, a dimmed video way says why (edges 14 and Bare),
// a press aims the letters at Snip and acts, the hint tells how to pick, a frozen screen draws its picture uncached and
// takes a press as a spot, a system dialog drops the windows from the top, and none takes focus.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "snipkit.js" as Kit
import "fixtures/snip.js" as Snip
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-contrast.js" as H

Item {
    id: root
    width: 200
    height: 100

    Component { id: snipC; FakeSnip {} }
    Component { id: appC; SnipApp {} }
    Component { id: namesC; SnipNames {} }

    TestCase {
        name: "snipbar"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` in language `tag` and look `lk`, with the aims counted.
        function make(name, tag, lk) {
            const snip = createTemporaryObject(snipC, root);
            const look = lk || N.lk;
            const aims = [];
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: look, res: Kit.resOf(look, tag || "en"), aim: () => aims.push(true)
            });
            snip.page = Kit.view(name);
            tryVerify(() => app.shown === app.stage, 2000, "the stage shows");
            return { snip: snip, app: app, aims: aims, tag: tag || "en" };
        }
        // Every window Snip's windows make, the frozen screens' included.
        function windows(app) {
            const sides = [app.edge.top, app.edge.bottom, app.edge.left, app.edge.right];
            const shown = [app.bar, app.bar.bar, app.hint, app.strip, app.strip.strip, app.count, app.record, app.record.bar];
            const frozen = [];
            for (let i = 0; i < app.overlays.count; i++)
                frozen.push(app.overlays.objectAt(i));
            return shown.concat(sides, frozen);
        }

        function test_the_click_kinds_and_capture_windows_are_the_hubs() {
            const names = createTemporaryObject(namesC, root);
            compare(names.acts, Snip.acts);
            compare(names.capture, Snip.capture);
        }

        function test_each_capture_window_is_shielded_with_its_title_before_it_shows() {
            const s = make("idle");
            const shields = s.snip.heard("shield");
            compare(shields.map(c => c.shield).sort(), Object.keys(Snip.capture).sort());
            for (const c of shields)
                compare(c.title, Kit.text("en", "window.snip-" + c.shield), c.shield);
            verify(windows(s.app).every(w => !w.visible), "nothing shows while idle");
            compare(s.app.bar.title, s.app.bar.bar.title, "the bar's shadow shares its title");
            compare(s.app.record.title, s.app.record.bar.title, "the record bar's shadow shares its title");
            compare(s.app.edge.top.title, Kit.text("en", "window.snip-edge"));
        }

        function test_a_snip_that_starts_sends_the_themes_sizes_first() {
            const s = make("region");
            const fits = s.snip.heard("fit");
            compare(fits.length, 1);
            const sh = N.lk.shape.snip;
            const f = fits[0].fit;
            compare([f.strip, f.bar, f.gap, f.slop], [sh.key_px + 2 * sh.plate_pad_px, sh.rec_px, sh.gap_px, sh.slop_px]);
            compare(f.handle, sh.handle_hit_px, "a frame handle's click area");
            compare(f.thick.map(t => t.pen), sh.pen_px);
            compare(f.text.map(t => t.height), sh.text_px);
            verify(f.text.every(t => t.letter >= 1 && t.letter < t.height), JSON.stringify(f.text));
            s.snip.page = Kit.view("corner");
            compare(s.snip.heard("fit").length, 1, "only a snip's start sends them");
        }

        function test_the_bar_names_its_keys_in_each_language() {
            for (const tag of Kit.langs) {
                const s = make("region", tag);
                const bar = s.app.bar.bar;
                tryVerify(() => bar.visible, 2000, tag);
                const ids = Snip.views.region.pick.keys.map(k => k.key);
                compare(Kit.buttons(bar).map(o => o.Accessible.name), ids.map(id => Kit.text(tag, "snip.way-" + id)), tag);
                compare(Kit.buttons(bar).map(o => o.Accessible.id), ids.map(id => "snip.bar." + id));
                compare(bar.title, Kit.text(tag, "window.snip-bar"));
                verify(Kit.button(bar, "region").Accessible.checked, "the picked way is on");
                const ways = ids.filter(id => Kit.button(bar, id).Accessible.role === Accessible.RadioButton);
                compare(ways, ["region", "window", "full", "video-region", "video-full"], "one way of many");
                verify(ways.every(id => Kit.button(bar, id).Accessible.checkable), "each says whether it is picked");
                compare(Kit.button(bar, "close").Accessible.role, Accessible.Button);
                compare(Kit.button(bar, "close").says, Kit.text(tag, "snip.way-close"), "an icon alone names itself in its tip");
                compare(Kit.button(bar, "region").says, "", "a worded key needs no tip");
            }
        }

        function test_a_dimmed_video_way_says_why_and_never_acts() {
            for (const [name, why] of [["no-video", "snip.dim-no-video"], ["bare", "snip.dim-bare"]]) {
                const s = make(name);
                const bar = s.app.bar.bar;
                tryVerify(() => bar.visible);
                waitForRendering(bar.contentItem);
                for (const id of ["video-region", "video-full"]) {
                    const k = Kit.button(bar, id);
                    compare(k.Accessible.description, Kit.text("en", why), id);
                    compare(k.says, Kit.text("en", "snip.way-" + id) + "\n" + Kit.text("en", why));
                    mouseClick(k);
                }
                compare(s.snip.heard("act"), [], name + ": a dimmed way never acts");
                mouseClick(Kit.button(bar, "window"));
                compare(s.snip.heard("act").map(a => a.id), ["window"], "while a live way there does");
            }
        }

        function test_the_picked_way_rings_and_a_video_way_lights_in_colours_that_read_on_its_cap() {
            for (const lk of [N.lk, H.lk]) {
                const bar = make("region", "en", lk).app.bar.bar;
                tryVerify(() => bar.visible);
                const [way, video] = [Kit.button(bar, "region"), Kit.button(bar, "video-region")];
                verify(way.on && video.lit && !Kit.button(bar, "window").lit, "only the video ways light");
                verify(Qt.colorEqual(way.ringInk, lk.palette.ring_act), "the side keycap's ring");
                verify(Qt.colorEqual(video.lightInk, lk.palette.rec_light), "the side keycap's red light");
            }
            const bar = make("region", "en", H.lk).app.bar.bar;
            tryVerify(() => bar.visible);
            waitForRendering(bar.contentItem);
            const video = Kit.button(bar, "video-region");
            mousePress(video);
            verify(video.down, "held down");
            verify(Qt.colorEqual(video.lightInk, H.lk.palette.press_ink), "pressed, the press text, as a legend");
            mouseRelease(video);
        }

        function test_a_press_on_the_bar_aims_the_letters_and_acts() {
            const s = make("region");
            const bar = s.app.bar.bar;
            tryVerify(() => bar.visible);
            waitForRendering(bar.contentItem);
            mouseClick(Kit.button(bar, "window"));
            compare(s.aims.length, 1, "the press aims the letters at Snip");
            compare(s.snip.heard("act"), [{ act: "bar", id: "window", arg: 0, round: Snip.views.region.round }]);
        }

        function test_the_hint_tells_how_to_pick_then_a_note() {
            const s = make("region", "ar");
            const pill = () => Board.all(s.app.hint.contentItem, o => o.Accessible.name === s.app.hint.said)[0];
            tryVerify(() => s.app.hint.visible);
            compare(s.app.hint.said, Kit.text("ar", "snip.hint-region"));
            compare(pill().Accessible.role, Accessible.StaticText, "a hint is read when asked");
            s.snip.page = Kit.view("corner");
            tryCompare(s.app.hint, "said", Kit.text("ar", "snip.hint-corner"));
            s.snip.page = Kit.view("note");
            tryCompare(s.app.hint, "said", Kit.text("ar", "snip.note-too-big"));
            tryVerify(() => s.app.hint.visible, 2000, "a note shows once the snip ended");
            compare(pill().Accessible.role, Accessible.AlertMessage, "a note is read out as it shows");
            verify(s.app.hint.flags & Qt.WindowTransparentForInput, "every click goes through it");
        }

        function test_a_frozen_screen_draws_its_picture_uncached_and_a_press_is_a_spot() {
            const s = make("region");
            const ov = s.app.overlays.objectAt(0);
            tryVerify(() => ov.visible);
            const image = Board.all(ov.contentItem, o => o.cache !== undefined && o.source !== undefined)[0];
            verify(!image.cache, "never cached, so the pixels go with the view");
            compare(image.source.toString(), Kit.picture);
            compare(ov.title, Kit.text("en", "window.snip-screen"));
            mouseClick(ov.contentItem, 50, 40);
            compare(s.aims.length, 0, "a frozen screen shows no line, so it never aims the letters");
            compare(s.snip.heard("spot"), [{ spot: 0, x: 50, y: 40, round: Snip.views.region.round }]);
        }

        function test_a_system_dialog_drops_snips_windows_from_the_top() {
            const s = make("low");
            tryVerify(() => s.app.strip.strip.visible);
            verify(!(s.app.strip.strip.flags & Qt.WindowStaysOnTopHint), "the strip leaves the top");
            verify(!(s.app.overlays.objectAt(0).flags & Qt.WindowStaysOnTopHint), "the frozen screen too");
            s.snip.page = Kit.view("strip");
            tryVerify(() => s.app.strip.strip.flags & Qt.WindowStaysOnTopHint, 2000, "back on top once it closed");
        }

        function test_no_window_takes_focus_and_high_contrast_draws_the_tag_opaque() {
            const s = make("region", "en", H.lk);
            verify(windows(s.app).every(w => w.flags & Qt.WindowDoesNotAcceptFocus), "no window takes focus");
            compare(s.app.tag.a, 1, "the tag is opaque in high contrast");
            fuzzyCompare(make("region").app.tag.a, Qt.tint(N.lk.common.snip_tag, "transparent").a, 0.01);
        }
    }
}
