// The selection strip over the stand-in Ui (design 4, "Keycaps"): the keyboard's own caps in each theme, light and dark,
// one key row tall at each size, icons or words alone, Arabic words in the Arabic font in the same order, a click sent
// with its round and chip at the press, sizes measured in physical pixels once and again after a reword, a ticked
// strip that loses its room or its key row still ending its round, no focus, and each keycap found and pressed by its
// accessible id and name.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/sel.js" as Sel
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-dark.js" as Nd
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-et66-dark.js" as Ed
import "../../kx-theme/fixtures/look-dolch-light.js" as D
import "../../kx-theme/fixtures/look-dolch-dark.js" as Dd
import "../../kx-theme/fixtures/look-native-light-contrast.js" as H

Item {
    id: root
    width: 200
    height: 100

    // One key row at size 1: a character key's height on the golden board.
    readonly property real row: Face.board.keys.find(k => !k.side && k.kind === "char").h
    // Every look with the cap kind it builds: flat, convex and sculpted, light and dark, and high contrast.
    readonly property var looks: [[N.lk, "flat"], [Nd.lk, "flat"], [E.lk, "convex"], [Ed.lk, "convex"], [D.lk, "skirt"], [Dd.lk, "skirt"], [H.lk, "flat"]]
    // The boxes the strip keeps clear of, as the app would hand them.
    readonly property var boxes: [{ x: 10, y: 20, w: 30, h: 40 }]

    Component { id: uiC; FakeUi {} }
    Component { id: stripC; SelChips {} }
    Component { id: namesC; SelNames {} }

    TestCase {
        name: "selchips"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A copy of the hub's view `name` in language `tag`, with `more` laid over it.
        function view(tag, name, more) {
            return Object.assign(JSON.parse(JSON.stringify(Sel.views[tag][name])), more || {});
        }
        // A strip over a stand-in Ui holding `v`, in look `lk` at size `s` (Native light at 1 by default).
        function make(v, props) {
            const p = Object.assign({ lk: N.lk, s: 1, ar: false, texts: En }, props || {});
            const ui = createTemporaryObject(uiC, root, { selView: v });
            const strip = createTemporaryObject(stripC, root, {
                ui: ui, lk: p.lk, res: Board.resOf(p.lk, p.texts), s: p.s, row: root.row * p.s, ar: p.ar,
                clear: () => root.boxes, logical: p.logical || ((x, y) => ({ x: x, y: y }))
            });
            tryVerify(() => strip.visible, 2000, "it shows");
            return strip;
        }
        // The strip's keycaps as a reader finds them, its buttons, left to right.
        function keys(strip) {
            return Board.all(strip.contentItem, o => o.Accessible.role === Accessible.Button).sort((a, b) => a.x - b.x);
        }
        // The keycap a reader or a gate finds by accessible id `uid`, the same in every language.
        function withId(strip, uid) {
            return keys(strip).find(k => k.Accessible.id === uid);
        }
        // The keycap a reader finds by its name, its word or its done word.
        function named(strip, name) {
            return keys(strip).find(k => k.Accessible.name === name);
        }
        // The first item under keycap `k` for which `ok` holds.
        function inside(k, ok) {
            return Board.all(k, ok)[0];
        }
        function face(k) {
            return inside(k, o => o.convex !== undefined && o.grad !== undefined);
        }
        function glyph(k) {
            return inside(k, o => o.base !== undefined && o.px !== undefined);
        }
        function word(k) {
            return inside(k, o => o.font !== undefined && o.renderType !== undefined);
        }
        // The keycap of chip `id`, found by its accessible id.
        function key(strip, id) {
            return withId(strip, Sel.ids[view("en", "both").chips.findIndex(c => c.id === id)]);
        }

        function test_each_theme_draws_the_keyboards_own_caps_on_its_plate() {
            for (const [lk, kind] of root.looks) {
                const strip = make(view("en", "both"), { lk: lk });
                const where = lk.look.id + (lk.dark ? " dark" : "") + (lk.contrast ? " contrast" : "");
                verify(Qt.colorEqual(strip.body.color, lk.palette.plate), where + ": the plate");
                compare(strip.body.radius, lk.look.window_radius_px, where);
                compare(strip.body.border.width, lk.contrast ? lk.shape.line.px : 0, where + ": an edge in high contrast");
                for (const k of keys(strip)) {
                    const f = face(k);
                    compare(f.cap.kind, kind, where);
                    verify(Qt.colorEqual(f.topColour, lk.palette.key), where + ": a character key's colour");
                    verify(Qt.colorEqual(word(k).color, lk.palette.legend), where + ": its legend");
                    compare(word(k).font.weight, lk.look.legend_weight, where + ": the theme's weight");
                }
            }
        }

        // Each keycap is one key row tall at the keyboard's size, inset by half the theme's gap as a key is.
        function test_each_keycap_is_one_key_row_tall_at_every_size() {
            let widths = [];
            for (const s of [1, 1.25, 1.5]) {
                const strip = make(view("en", "both"), { s: s });
                const caps = keys(strip);
                compare(caps.length, 4, "every chip");
                for (const k of caps) {
                    compare(k.height, root.row * s, s + ": one key row");
                    compare(face(k).parent.y, N.lk.look.gap_px * s / 2, s + ": half a gap in");
                }
                compare(strip.height, Math.ceil(root.row * s + 2 * N.lk.shape.chips.pad_px * s), s + ": with the plate's padding");
                widths.push(strip.width);
            }
            verify(widths[0] < widths[1] && widths[1] < widths[2], "it grows with the keyboard: " + widths);
        }

        function test_icons_or_words_alone_show_as_the_view_says() {
            const both = keys(make(view("en", "both")));
            const icons = keys(make(view("en", "icons")));
            const words = keys(make(view("en", "words")));
            for (let i = 0; i < 4; i++) {
                verify(glyph(icons[i]).visible && !word(icons[i]).visible, "icons alone");
                verify(!glyph(words[i]).visible && word(words[i]).visible, "words alone");
                verify(glyph(both[i]).visible && word(both[i]).visible, "both");
                verify(icons[i].width < both[i].width && words[i].width < both[i].width, "as wide as its legend");
            }
            compare(glyph(both[0]).icon, N.lk.icons.chips.copy, "the theme's icon by chip");
            compare(glyph(both[3]).icon, N.lk.icons.chips.translate);
        }

        // Arabic flips the text only, so Copy, Cut, Paste and Translate stay left to right (ADR-0020).
        function test_arabic_words_take_the_arabic_font_and_keep_the_order() {
            const v = view("ar", "both");
            const latin = word(keys(make(view("en", "both")))[0]);
            compare(latin.font.family, Board.resOf(N.lk, En).fam.latin, "Latin in the Latin font");
            for (const lk of [N.lk, E.lk, D.lk]) {
                const strip = make(v, { lk: lk, ar: true, texts: Ar });
                const caps = keys(strip);
                compare(caps.map(k => k.Accessible.id), Sel.ids, lk.look.id + ": the order kept");
                compare(caps.map(k => word(k).text), v.chips.map(c => c.word), lk.look.id);
                verify(caps.every((k, i) => i === 0 || k.x > caps[i - 1].x), lk.look.id + ": left to right");
                for (const k of caps) {
                    compare(word(k).font.family, strip.res.fam.arabic, lk.look.id + ": the Arabic font");
                    // A font's pixel size is whole, so the theme's Arabic scale shows within a pixel.
                    fuzzyCompare(word(k).font.pixelSize, lk.shape.chips.font_px * lk.look.arabic_scale, 1, lk.look.id);
                    verify(word(k).font.pixelSize > latin.font.pixelSize, lk.look.id + ": larger than Latin");
                }
            }
        }

        // A keycap acts at the press, as a key does, with its round and chip; a ticked strip takes no click.
        function test_a_click_calls_the_hub_with_its_round_and_chip() {
            const strip = make(view("en", "both", { round: 9 }));
            waitForRendering(strip.contentItem);
            mousePress(key(strip, "cut"));
            compare(strip.ui.selCalls, ["selAct 9 cut"], "at the press");
            compare(inside(key(strip, "cut"), o => o.hovered !== undefined && o.sink !== undefined).sink, 1, "its cap sinks at once");
            mouseRelease(key(strip, "cut"));
            compare(strip.ui.selCalls.length, 1, "the let-go adds nothing");
            strip.ui.selView = view("en", "ticked", { round: 9, acted: "cut" });
            compare(word(key(strip, "cut")).text, view("en", "both").chips[1].done, "its done word");
            compare(glyph(key(strip, "cut")).icon, N.lk.icons.chips.tick, "and the tick");
            compare(glyph(key(strip, "copy")).icon, N.lk.icons.chips.copy, "the others stay");
            mouseClick(key(strip, "copy"));
            compare(strip.ui.selCalls.length, 1, "a ticked strip takes no click");
        }

        // A reader or a gate finds each keycap by its id, the same in every language, and by its name, in English and
        // Arabic; a press on the keycap found either way asks the hub for that chip in that round.
        function test_each_keycap_found_by_its_id_or_its_name_asks_for_its_chip() {
            for (const [tag, texts] of [["en", En], ["ar", Ar]]) {
                const v = view(tag, "both", { round: 6 });
                const strip = make(v, { ar: tag === "ar", texts: texts });
                waitForRendering(strip.contentItem);
                const asked = [];
                v.chips.forEach((c, i) => {
                    const [byId, byName] = [withId(strip, Sel.ids[i]), named(strip, c.word)];
                    verify(byId !== undefined && byId === byName, tag + " " + c.id + ": one keycap by its id and name");
                    const k = i % 2 ? byName : byId;
                    mousePress(k);
                    mouseRelease(k);
                    asked.push("selAct 6 " + c.id);
                    compare(strip.ui.selCalls, asked, tag + " " + c.id + ": its own ask");
                });
            }
        }

        function test_it_takes_no_focus_and_readers_find_it() {
            const v = view("en", "both");
            const strip = make(v);
            verify(strip.flags & Qt.WindowDoesNotAcceptFocus, "never takes focus");
            verify(strip.flags & Qt.FramelessWindowHint, "no title bar");
            compare(strip.title, v.name, "the title is its reader name");
            compare([strip.body.Accessible.role, strip.body.Accessible.name], [Accessible.ToolBar, v.name]);
            const caps = keys(strip);
            compare(caps.map(k => k.Accessible.role), caps.map(() => Accessible.Button));
            compare(caps.map(k => k.Accessible.name), v.chips.map(c => c.word), "each named by its word");
            compare(caps.map(k => k.Accessible.id), Sel.ids, "each its word's text key, the same in every language");
            compare(createTemporaryObject(namesC, root).group, Sel.group, "the group the gates find");
            strip.ui.selView = view("en", "ticked", { acted: "copy" });
            const said = v.chips.map(c => c.id === "copy" ? c.done : c.word);
            compare(caps.map(k => k.Accessible.name), said, "a ticked keycap reads its done word");
            compare(caps.map(k => k.Accessible.id), Sel.ids, "and keeps its id");
        }

        // `selAt` takes physical pixels: the size times the selection screen's scale, once, with the keyboard's size for
        // the gap; its answer comes back to Qt.
        function test_the_size_goes_to_the_hub_in_physical_pixels_once() {
            const half = (x, y) => ({ x: x / 2, y: y / 2 });
            const strip = make(view("en", "both", { round: 3, scale: 1.5 }), { s: 1.25, logical: half });
            const ask = strip.ui.selAsks[strip.ui.selAsks.length - 1];
            compare([ask.round, ask.w, ask.h, ask.size], [3, strip.width * 1.5, strip.height * 1.5, 1.25]);
            compare(JSON.stringify(ask.clear), JSON.stringify(root.boxes), "clear of our windows");
            compare([strip.x, strip.y], [strip.ui.selSpot.x / 2, strip.ui.selSpot.y / 2], "back in Qt units");
        }

        // A reword of the same round, as a new keyboard language, measures again without a new pop.
        function test_a_reword_measures_the_strip_again() {
            const strip = make(view("en", "both"));
            const [asks, wide] = [strip.ui.selAsks.length, strip.width];
            strip.ui.selView = view("fr", "both");
            verify(strip.width > wide, "the French words are longer");
            verify(strip.ui.selAsks.length > asks, "measured again");
            compare(strip.ui.selAsks[strip.ui.selAsks.length - 1].w, strip.width, "at its new width");
            compare(word(keys(strip)[0]).text, view("fr", "both").chips[0].word);
        }

        // A ticked strip that loses its room, in its tick or its fade, still ends its round, so a wake never shows it again.
        function test_a_ticked_strip_with_no_room_still_ends_its_round() {
            for (const fading of [false, true]) {
                const strip = make(view("en", "both", { round: 7 }));
                strip.ui.selView = view("en", "ticked", { round: 7 });
                verify(strip.done.running, "the tick shows");
                if (fading)
                    strip.fadeOut();
                strip.ui.selSpot = null;
                strip.follow();
                verify(!strip.visible, fading + ": hidden");
                compare(strip.ui.selCalls, ["selFaded 7"], fading + ": the hub hears it once");
                verify(!strip.done.running, fading + ": no fade after");
                strip.ui.selSpot = { x: 30, y: 40 };
                strip.ui.settings = Object.assign({}, strip.ui.settings);
                verify(!strip.visible, fading + ": a wake shows nothing");
            }
        }

        // A key row that goes, as when the keyboard module stops, hides the strip as no room does: a live one shows
        // again with the row, and a ticked one ends its round.
        function test_a_strip_whose_key_row_goes_hides() {
            const live = make(view("en", "both", { round: 4 }));
            live.row = 0;
            tryVerify(() => !live.visible, 1000, "no key row, so it hides");
            compare(live.ui.selCalls, [], "a live strip ends no round");
            live.row = root.row;
            tryVerify(() => live.visible, 1000, "the row came back");
            const ticked = make(view("en", "both", { round: 8 }));
            ticked.ui.selView = view("en", "ticked", { round: 8 });
            ticked.row = 0;
            tryVerify(() => !ticked.visible, 1000, "a ticked strip hides too");
            compare(ticked.ui.selCalls, ["selFaded 8"], "and ends its round");
        }

        // With no room the hub answers null and nothing shows; a hidden view hides the strip.
        function test_no_room_or_no_view_shows_nothing() {
            const strip = make(view("en", "both"));
            strip.ui.selView = null;
            verify(!strip.visible, "hidden");
            strip.ui.selSpot = null;
            strip.ui.selView = view("en", "both", { round: 5 });
            verify(!strip.visible, "no room");
            strip.ui.selSpot = { x: 30, y: 40 };
            strip.follow();
            verify(strip.visible, "room again, so it shows");
        }
    }
}
