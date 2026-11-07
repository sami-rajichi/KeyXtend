// The keyboard window's tooltip, once the look and settings read. The window owns it, so it stays above the window when
// the guard lifts it; the bubble owns its own.
import QtQuick

Loader {
    id: tips

    // The keyboard window, whose buttons it names.
    required property Window win

    active: win.lk !== null && win.cfg !== null
    sourceComponent: Tip {
        transientParent: tips.win
        lk: tips.win.lk
        res: tips.win.res
        ms: tips.win.cfg.tipMs
        lift: t => tips.win.keyboard.liftAlone(t)
    }
}
