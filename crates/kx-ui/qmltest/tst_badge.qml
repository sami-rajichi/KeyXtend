// The mode badge beside the pointer (spec §5.3, mock-up .badge): the mode's icon, the word while Grab carries, its
// widest width, its pill, colours and own shadow, and the keyboard's language. tst_overlay checks it beside the pointer.
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

    Component { id: badgeC; Badge {} }

    TestCase {
        name: "badge"
        when: windowShown

        // The badge's tokens.
        readonly property var sh: N.lk.shape.badge

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function find(item, ok) {
            return Board.all(item, ok)[0] || null;
        }
        function badge(ui, mode, grab) {
            const res = Board.resOf(N.lk, ui);
            return createTemporaryObject(badgeC, root, { lk: N.lk, res: res, mode: mode, grab: grab, shown: true });
        }
        // The badge's text and its pill.
        function word(b) {
            return find(b, o => o.font !== undefined && o.text !== undefined);
        }
        function pill(b) {
            return find(b, o => o.radius !== undefined && o.color !== undefined && Qt.colorEqual(o.color, N.lk.palette.badge_bg));
        }

        function test_each_mode_shows_its_icon_and_carrying_adds_the_word() {
            for (const [grab, mode, said] of [["off", "rclick", false], ["armed", "grab", false], ["carrying", "grab", true]]) {
                const b = badge(En, mode, grab);
                const icon = find(b, o => o.icon !== undefined && o.tint !== undefined);
                compare(icon.icon, N.lk.icons.keys[mode], grab);
                verify(Qt.colorEqual(icon.tint, N.lk.palette.badge_ink), grab);
                compare(word(b).visible, said, grab);
                compare(b.Accessible.name, En.texts["label." + mode], "readers hear the mode");
                compare(b.Accessible.description, said ? En.texts["badge.holding"] : "");
                const wide = said ? sh.gap_px + word(b).implicitWidth : 0;
                fuzzyCompare(b.width, 2 * sh.pad_px[1] + sh.icon_px + wide, 0.01, grab);
                fuzzyCompare(b.widest, 2 * sh.pad_px[1] + sh.icon_px + sh.gap_px + word(b).implicitWidth, 0.01, "the widest");
                compare(b.height, sh.icon_px + 2 * sh.pad_px[0], grab);
            }
        }

        // A pill (mock-up .badge, border-radius 999px) in the badge colours; its word is in the keyboard's language.
        function test_the_badge_is_a_pill_in_its_colours_and_speaks_the_keyboards_language() {
            const b = badge(Ar, "grab", "carrying");
            compare(pill(b).radius, b.height / 2);
            compare(word(b).text, Ar.texts["badge.holding"]);
            compare(word(b).font.family, Board.resOf(N.lk, Ar).fam.ui);
            verify(Qt.colorEqual(word(b).color, N.lk.palette.badge_ink));
            b.shown = false;
            tryCompare(b, "opacity", 0, 2000, "it fades out");
        }

        // Its own small shadow (mock-up .badge box-shadow), not the windows' wide one; high contrast drops it.
        function test_the_badge_casts_its_own_small_shadow() {
            const fields = l => ["inset", "x", "y", "blur", "spread", "color"].map(k => String(l[k])).join(" ");
            for (const look of [N.lk, NC.lk]) {
                const b = createTemporaryObject(badgeC, root, { lk: look, res: Board.resOf(look, En), mode: "rclick", grab: "off", shown: true });
                const shades = Board.all(b, o => o.shadow !== undefined && o.modelData !== undefined).map(o => fields(o.shadow));
                compare(shades, look.palette.badge_shadow.map(fields), look.contrast ? "contrast" : "native");
            }
            verify(N.lk.palette.badge_shadow.length > 0 && NC.lk.palette.badge_shadow.length === 0);
        }
    }
}
