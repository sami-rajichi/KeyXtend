// Where things go for the keyboard window: Qt units and physical pixels on the screen that holds them, and a pop-up's
// spot above the keyboard.
import QtQuick

QtObject {
    // The keyboard window, whose place, plate size and Ui object the spots read.
    required property Window win
    // The screens, each with its corner in physical pixels and size in Qt units; tests give scaled ones.
    property var screens: Qt.application.screens

    // Where a `w` by `h` pop-up goes, in Qt units: above the shown keyboard with its left edge at Qt x `left`, or
    // centred when `left` is NaN; with the keyboard hidden, at the bottom centre of the main screen. Null with no screen.
    function popSpot(w, h, left) {
        const p = JSON.parse(win.ui.popAt(...beside(left), w, h));
        return p ? logical(p.x, p.y) : null;
    }
    // The same for a pop-up panel, with the share of its size it takes there (`{x, y, fit}`): one too tall for the
    // room above or below the keyboard shrinks, never below the theme's least fit (spec §3.4).
    function popFit(w, h, left) {
        const p = JSON.parse(win.ui.popFit(...beside(left), w, h));
        return p ? Object.assign(logical(p.x, p.y), { fit: p.fit }) : null;
    }
    // The shown keyboard's box in physical pixels (JSON, null while hidden), the pop-up's left edge and whether it is
    // centred, as `popAt` and `popFit` take them.
    function beside(left) {
        const a = physical(win.at.x, win.at.y);
        const over = win.visible ? { x: a.x, y: a.y, w: win.drawn.width * a.dpr, h: win.drawn.height * a.dpr } : null;
        const centred = isNaN(left);
        return [JSON.stringify(over), centred ? 0 : physical(left, win.at.y).x, centred];
    }

    // Qt units for physical pixel (px, py), using the scale of the screen that holds it; Qt keeps each screen's corner unscaled.
    function logical(px, py) {
        const sc = screenOf(px, py, true);
        if (!sc)
            return { x: px, y: py };
        const d = sc.devicePixelRatio;
        return { x: sc.virtualX + (px - sc.virtualX) / d, y: sc.virtualY + (py - sc.virtualY) / d };
    }
    // Physical pixels for Qt point (lx, ly), and that screen's scale.
    function physical(lx, ly) {
        const sc = screenOf(lx, ly, false);
        if (!sc)
            return { x: lx, y: ly, dpr: 1 };
        const d = sc.devicePixelRatio;
        return { x: sc.virtualX + (lx - sc.virtualX) * d, y: sc.virtualY + (ly - sc.virtualY) * d, dpr: d };
    }
    // The screen holding point (x, y), in physical pixels or Qt units; null when none does.
    function screenOf(x, y, phys) {
        const holds = sc => {
            const d = phys ? sc.devicePixelRatio : 1;
            const inX = x >= sc.virtualX && x < sc.virtualX + sc.width * d;
            return inX && y >= sc.virtualY && y < sc.virtualY + sc.height * d;
        };
        return Array.from(screens).find(holds) || null;
    }
}
