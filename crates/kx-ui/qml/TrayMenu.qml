// The tray icon's menu (mock-up .ctx, in the theme's panel colours): Show keyboard and Quit, in the system's language
// (ADR-0020). It opens by the tray icon, never takes focus, and closes on a pick, another click on the icon, a press on
// our windows or a new front window.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: menu

    // The look, fonts, icons and texts (`res.tray`), and where a `w` by `h` menu goes for a click at physical
    // (x, y): a Qt point or null.
    required property var lk
    required property var res
    required property var spot
    // How long after it opens a new front window leaves it open, in ms (`[ui] tray_grace_ms`), and the clock in ms.
    required property int graceMs
    property var now: () => Date.now()
    property real openedAt: 0

    // An item was picked; the menu has closed.
    signal showKeyboard
    signal quitApp
    // The menu shows (true) or hides (false); its shadow follows.
    signal showing(bool on)

    // The box whose fade and scale the shadow follows.
    readonly property alias body: body
    readonly property var sh: lk.shape.menu
    readonly property var p: lk.palette

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: res.tray("window.menu")
    color: "transparent"
    visible: false
    // Whole units up, as a window's size is whole, so the widest item always fits.
    width: rescale.keep(Math.ceil(Math.max(sh.width_px, Math.max(showItem.implicitWidth, quitItem.implicitWidth) + 2 * sh.pad_px)))
    height: rescale.keep(Math.ceil(items.implicitHeight + 2 * sh.pad_px))

    // Keeps the size above across display scale changes.
    Rescale {
        id: rescale
    }

    // A click on the tray icon at physical (px, py): opens the menu there, or closes it when it shows.
    function toggle(px, py) {
        if (visible) {
            shut();
            return;
        }
        const at = spot(px, py, width, height);
        if (!at)
            return;
        x = at.x;
        y = at.y;
        openedAt = now();
        showing(true);
        visible = true;
        grow.restart();
    }
    // Another program's window came to the front: it closes, unless that came within the grace after it opened, as
    // the hidden icons' pop-up closing can bring one.
    function away() {
        const since = now() - openedAt;
        if (since < 0 || since >= graceMs)
            shut();
    }
    function shut() {
        if (!visible)
            return;
        visible = false;
        showing(false);
    }
    function picked(what) {
        shut();
        what();
    }

    Rectangle {
        id: body
        width: menu.width
        height: menu.height
        radius: menu.sh.radius_px
        color: menu.p.pop_bg
        border.width: menu.lk.contrast ? menu.lk.shape.line.px : 0
        border.color: menu.p.pop_line
        // It grows from the corner by the icon (.ctx transform-origin), which is its bottom left.
        transformOrigin: Item.BottomLeft
        Accessible.role: Accessible.PopupMenu
        Accessible.name: menu.title

        Column {
            id: items
            x: menu.sh.pad_px
            y: menu.sh.pad_px
            width: menu.width - 2 * menu.sh.pad_px
            MenuItem {
                id: showItem
                part: "show"
                text: menu.res.tray("tray.show")
                onClicked: menu.picked(menu.showKeyboard)
            }
            // The rule between items (.ctx hr).
            Item {
                width: parent.width
                height: 2 * menu.sh.rule_px[0] + menu.lk.shape.line.px
                Rectangle {
                    x: menu.sh.rule_px[1]
                    y: menu.sh.rule_px[0]
                    width: parent.width - 2 * menu.sh.rule_px[1]
                    height: menu.lk.shape.line.px
                    color: menu.p.pop_line
                }
            }
            MenuItem {
                id: quitItem
                part: "quit"
                text: menu.res.tray("tray.quit")
                onClicked: menu.picked(menu.quitApp)
            }
        }
    }

    // One item (.ctx button): its icon from `icons.toml [tray]` and its text.
    component MenuItem: Item {
        id: it
        required property string part
        required property string text
        signal clicked
        readonly property bool hot: area.containsMouse
        // Hover text sits on pop_hover, which is Highlight in high contrast, so it takes sel_ink there.
        readonly property color ink: hot ? menu.lk.sel_ink : menu.p.pop_ink
        width: parent ? parent.width : 0
        implicitWidth: row.implicitWidth + 2 * menu.sh.item_pad_px[1]
        height: row.implicitHeight + 2 * menu.sh.item_pad_px[0]
        Accessible.role: Accessible.MenuItem
        Accessible.name: text
        Accessible.onPressAction: it.clicked()
        Hover {
            lk: menu.lk
            radius: menu.sh.item_radius_px
            visible: it.hot
            tint: menu.p.pop_hover
        }
        Row {
            id: row
            anchors.left: parent.left
            anchors.leftMargin: menu.sh.item_pad_px[1]
            anchors.verticalCenter: parent.verticalCenter
            spacing: menu.sh.gap_px
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                icon: menu.lk.icons.tray[it.part]
                base: menu.res.icons
                ext: menu.res.ext
                px: menu.sh.icon_px
                tint: it.ink
                opacity: menu.sh.icon_share
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: it.text
                font.family: menu.res.fam.tray
                font.pixelSize: menu.sh.font_px
                color: it.ink
            }
        }
        MouseArea {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onClicked: it.clicked()
        }
    }

    // It opens as the mock-up's menu does (ctxIn): a fade and a grow.
    ParallelAnimation {
        id: grow
        Tween {
            target: body
            property: "opacity"
            from: 0
            to: 1
            move: menu.lk.motion.moves.menu
        }
        Tween {
            target: body
            property: "scale"
            from: menu.lk.motion.amount.menu_scale
            to: 1
            move: menu.lk.motion.moves.menu
        }
    }
}
