// The pointer badge (spec §3.2, §5.3, mock-up .badge): the mode's icon, the word while Grab carries, the scroll pad's
// arrow and speed after a dot, its widest width, its pill, colours and own shadow, and the keyboard's language, whose
// Arabic words read right to left with no part mirrored. tst_overlay checks it beside the pointer.
pragma ComponentBehavior: Bound
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NC

Item {
    id: root
    width: 200
    height: 100

    // The bundled fonts, from where the texts test lays them out.
    property var bundled: []

    Component { id: badgeC; Badge {} }
    // A twin of a drawn text with one part as a link, so `linkAt` finds where Qt lays that part out (tst_order).
    Component {
        id: twinC
        Text {
            textFormat: Text.StyledText
            opacity: 0
        }
    }

    TestCase {
        name: "badge"
        when: windowShown

        // The badge's tokens.
        readonly property var sh: N.lk.shape.badge

        function initTestCase() {
            root.bundled = Board.loadFonts(root);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function find(item, ok) {
            return Board.all(item, ok)[0] || null;
        }
        function badge(ui, mode, grab, scroll) {
            const res = Board.resOf(N.lk, ui);
            return createTemporaryObject(badgeC, root, { lk: N.lk, res: res, mode: mode, grab: grab, scroll: scroll || null, shown: true });
        }
        // The badge's text `key` in language `ui`, its icon `name`, and its pill.
        function word(b, ui, key) {
            return find(b, o => o.font !== undefined && o.text === ui.texts[key]);
        }
        function icon(b, name) {
            return find(b, o => o.icon === name && o.tint !== undefined);
        }
        function pill(b) {
            return find(b, o => o.radius !== undefined && o.color !== undefined && Qt.colorEqual(o.color, N.lk.palette.badge_bg));
        }
        function dot(b) {
            return find(b, o => o.radius !== undefined && o.color !== undefined && Qt.colorEqual(o.color, N.lk.palette.badge_ink));
        }
        // Its widest: the mode's icon and word, the dot, the arrow and the wider speed word.
        function widest(b, ui) {
            const w = k => word(b, ui, k).implicitWidth;
            const parts = 2 * sh.icon_px + 4 * sh.gap_px + sh.sep_px + w("badge.holding") + Math.max(w("badge.slow"), w("badge.fast"));
            return 2 * sh.pad_px[1] + parts;
        }

        function test_each_mode_shows_its_icon_and_carrying_adds_the_word() {
            for (const [grab, mode, said] of [["off", "rclick", false], ["armed", "grab", false], ["carrying", "grab", true]]) {
                const b = badge(En, mode, grab);
                const i = icon(b, N.lk.icons.keys[mode]);
                verify(i && i.visible, grab);
                verify(Qt.colorEqual(i.tint, N.lk.palette.badge_ink), grab);
                compare(word(b, En, "badge.holding").visible, said, grab);
                compare(b.Accessible.name, En.texts["label." + mode], "readers hear the mode");
                compare(b.Accessible.description, said ? En.texts["badge.holding"] : "");
                const wide = said ? sh.gap_px + word(b, En, "badge.holding").implicitWidth : 0;
                fuzzyCompare(b.width, 2 * sh.pad_px[1] + sh.icon_px + wide, 0.01, grab);
                fuzzyCompare(b.widest, widest(b, En), 0.01, "the widest");
                compare(b.height, sh.icon_px + 2 * sh.pad_px[0], grab);
            }
        }

        // While the pad scrolls, the badge shows its arrow and slow or fast, alone or after a dot beside the mode.
        function test_the_scroll_part_shows_the_arrow_and_the_speed() {
            for (const [dir, fast] of [["down", false], ["left", true]]) {
                const b = badge(En, "", "off", { dir: dir, fast: fast });
                const key = fast ? "badge.fast" : "badge.slow", w = word(b, En, key);
                verify(icon(b, N.lk.icons.dpad[dir]).visible && w.visible, dir);
                verify(!word(b, En, fast ? "badge.slow" : "badge.fast").visible, "one speed");
                verify(!dot(b).visible, "no dot with no mode");
                compare(b.opacity, 1, "it shows with no mode");
                fuzzyCompare(b.width, 2 * sh.pad_px[1] + sh.icon_px + sh.gap_px + w.implicitWidth, 0.01, dir);
                compare(b.Accessible.name, b.res.dpad(dir));
                compare(b.Accessible.description, En.texts[key]);
            }
        }

        function test_beside_a_mode_a_faded_dot_parts_the_two() {
            const b = badge(En, "grab", "carrying", { dir: "up", fast: true });
            const parts = [icon(b, N.lk.icons.keys.grab), word(b, En, "badge.holding"), dot(b), icon(b, N.lk.icons.dpad.up), word(b, En, "badge.fast")];
            verify(parts.every(p => p.visible), "every part shows");
            for (let i = 1; i < parts.length; i++)
                fuzzyCompare(parts[i].x, parts[i - 1].x + parts[i - 1].width + sh.gap_px, 0.01, "part " + i + " after the last");
            const d = dot(b);
            compare([d.width, d.height, d.radius, d.opacity], [sh.sep_px, sh.sep_px, sh.sep_px / 2, sh.sep_share]);
            const wider = Math.max(...["badge.slow", "badge.fast"].map(k => word(b, En, k).implicitWidth));
            fuzzyCompare(b.width - parts[4].implicitWidth + wider, b.widest, 0.01, "the widest it gets, with the wider speed");
            fuzzyCompare(b.widest, widest(b, En), 0.01);
            const join = En.texts["list.join"];
            compare(b.Accessible.description, [En.texts["badge.holding"], b.res.dpad("up"), En.texts["badge.fast"]].join(join));
        }

        // Arabic words read right to left inside their part, and the parts keep their places, as nothing mirrors.
        function test_arabic_words_read_right_to_left() {
            tryVerify(() => Board.loaded(root.bundled), 5000, "the bundle is laid out by the texts test");
            const b = badge(Ar, "grab", "carrying", { dir: "left", fast: false });
            const [hold, slow] = [word(b, Ar, "badge.holding"), word(b, Ar, "badge.slow")];
            verify(icon(b, N.lk.icons.dpad.left).x < slow.x && hold.x < slow.x, "the parts keep their places");
            const holding = Ar.texts["badge.holding"].split(" ");
            verify(placeOf(hold, holding[0]) > placeOf(hold, holding[holding.length - 1]), "its first word on the right");
            const letters = Array.from(Ar.texts["badge.slow"]);
            verify(placeOf(slow, letters[0]) > placeOf(slow, letters[letters.length - 1]), "its first letter on the right");
            compare(slow.font.family, Board.resOf(N.lk, Ar).fam.ui);
        }
        // Where drawn text `label` lays out `part`: the middle of its pixels on its one line.
        function placeOf(label, part) {
            const at = label.text.indexOf(part);
            verify(at >= 0, part + " is drawn");
            const linked = label.text.slice(0, at) + "<a href=\"part\">" + part + "</a>" + label.text.slice(at + part.length);
            const twin = createTemporaryObject(twinC, root, { text: linked, font: label.font });
            const xs = [];
            for (let x = 0; x < twin.width; x++)
                if (twin.linkAt(x, twin.height / 2) === "part")
                    xs.push(x);
            verify(xs.length > 0, part + " is laid out");
            return (xs[0] + xs[xs.length - 1]) / 2;
        }

        // A pill (mock-up .badge, border-radius 999px) in the badge colours; its word is in the keyboard's language.
        function test_the_badge_is_a_pill_in_its_colours_and_speaks_the_keyboards_language() {
            const b = badge(Ar, "grab", "carrying");
            compare(pill(b).radius, b.height / 2);
            const w = word(b, Ar, "badge.holding");
            compare(w.font.family, Board.resOf(N.lk, Ar).fam.ui);
            verify(Qt.colorEqual(w.color, N.lk.palette.badge_ink));
            b.shown = false;
            tryCompare(b, "opacity", 0, 2000, "it fades out");
        }

        // Its own small shadow (mock-up .badge box-shadow), not the windows' wide one; high contrast drops it.
        function test_the_badge_casts_its_own_small_shadow() {
            const fields = l => ["inset", "x", "y", "blur", "spread", "color"].map(k => String(l[k])).join(" ");
            for (const look of [N.lk, NC.lk]) {
                const b = createTemporaryObject(badgeC, root, { lk: look, res: Board.resOf(look, En), mode: "rclick", grab: "off", scroll: null, shown: true });
                const shades = Board.all(b, o => o.shadow !== undefined && o.modelData !== undefined).map(o => fields(o.shadow));
                compare(shades, look.palette.badge_shadow.map(fields), look.contrast ? "contrast" : "native");
            }
            verify(N.lk.palette.badge_shadow.length > 0 && NC.lk.palette.badge_shadow.length === 0);
        }
    }
}
