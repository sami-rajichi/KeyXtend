// The clipboard panel with its shadow (spec §7), placed and fitted beside the keyboard as every list panel is
// (PanelGlow). A press on it counts as the keyboard's.
pragma ComponentBehavior: Bound
import QtQuick

PanelGlow {
    id: panel

    pane: pop
    wide: win.lk.shape.panel.width_px

    ClipBox {
        id: pop
        ui: panel.win.ui
        lk: panel.win.lk
        res: panel.win.res
        s: panel.win.s * panel.fit
        view: panel.view
        rows: panel.win.cfg.pageRows
        rtl: !!panel.win.tongue.rtl
        aimed: !!panel.win.st.aimed
        onAiming: panel.win.redraw(panel.win.keyboard.aimPanel())
        onTouched: panel.win.touched()
        onRescaled: panel.rescaled()
        onUnitChanged: panel.relaid()
        Component.onCompleted: panel.start()
    }
}
