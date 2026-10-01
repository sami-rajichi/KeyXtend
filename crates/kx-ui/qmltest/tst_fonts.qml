// The shared font rule: a test that needs a font Windows ships skips, naming it, only when this system lacks it; a
// missing bundled font never skips, so a broken bundle still fails.
import QtQuick
import QtTest
import "fixtures/assets.js" as Assets
import "fonts.js" as Fonts

TestCase {
    name: "fonts"

    // Any warning fails the test, so the run stays clean.
    function init() {
        failOnWarning(/./);
    }
    // What `Fonts.need` says for `families` when Qt has `have`, or null when it does not skip.
    function said(families, have) {
        let why = null;
        Fonts.need(families, m => why = m, have);
        return why;
    }

    function test_a_missing_windows_family_skips_and_is_named() {
        const win = Assets.system[0];
        const why = said([win], []);
        verify(why !== null && why.indexOf(win) >= 0, "named: " + why);
    }

    function test_a_windows_family_qt_has_does_not_skip() {
        compare(said(Assets.system, Assets.system), null);
    }

    function test_a_missing_bundled_family_does_not_skip() {
        const bundled = "A family no system ships";
        verify(Assets.system.indexOf(bundled) < 0, "not a Windows family");
        compare(said([bundled], []), null);
    }

    function test_qt_is_asked_when_no_list_is_given() {
        const all = Qt.fontFamilies();
        compare(said(Assets.system.filter(f => all.indexOf(f) >= 0)), null);
    }
}
