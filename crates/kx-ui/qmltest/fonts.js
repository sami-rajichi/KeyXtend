.pragma library
// The one rule for a test that needs a font Windows ships: on a system without it, such as a CI image, the test skips
// and names the font, so it never passes unseen.
.import "fixtures/assets.js" as Assets

// Calls `skip` with the reason when Qt's families (`have`, or Qt's own list) lack a Windows family that `families`
// names; a missing bundled family never skips, so a broken bundle still fails.
function need(families, skip, have) {
    const known = have || Qt.fontFamilies();
    const gone = families.filter(f => Assets.system.indexOf(f) >= 0 && known.indexOf(f) < 0);
    if (gone.length > 0)
        skip("this system lacks " + gone.join(", ") + ", a font Windows 11 ships");
}
