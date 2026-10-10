// The notice banner above the keyboard (mock-up .toast, with the panel's .btn buttons): one notice at a time, with Try
// again when the app can restart what failed, and OK; it counts the notices still waiting. It never takes focus.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: note

    // The app's Ui object, the look, fonts, icons and texts, the keyboard size, its widest width in Qt units, and
    // where a pop-up of a given size goes: `spot(w, h)` gives a Qt point or null. It never mirrors (ADR-0020).
    required property var ui
    required property var lk
    required property var res
    required property real s
    required property real widest
    required property var spot

    // The notice shown, then those waiting: `{id, text, retry}` each.
    property var queue: []

    // The banner shows (true) or hides (false); its shadow follows.
    signal showing(bool on)
    // A press landed on the banner, which closes the tray menu; the press still reaches the button.
    signal touched

    // The box whose fade and drop the shadow follows.
    readonly property alias body: body
    readonly property var shown: queue.length > 0 ? queue[0] : null
    // Read again when the texts change, as `res` does.
    readonly property string more: queue.length > 1 && res ? ui.moreText(queue.length - 1) : ""
    readonly property var sh: lk.shape.notice
    readonly property var p: lk.palette
    readonly property real radius: lk.look.window_radius_px * sh.radius_share
    readonly property real side: sh.pad_px[1] * s
    readonly property real gap: sh.gap_px * s
    // What the icon, the buttons, the padding and the gaps take; the words get the rest, up to the widest width.
    readonly property real fixed: 2 * side + icon.width + buttons.width + 2 * gap
    readonly property real wordsW: Math.max(0, Math.min(Math.ceil(Math.max(msgSize.advanceWidth, moreSize.advanceWidth)), widest - fixed))

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    title: res.tr("window.notice")
    color: "transparent"
    visible: false
    // Whole units up, as a window's size is whole.
    width: rescale.keep(Math.ceil(fixed + wordsW))
    height: rescale.keep(Math.ceil(Math.max(words.height, buttons.height, icon.height) + 2 * sh.pad_px[0] * s))

    // Keeps the size above across display scale changes, and the place over the keyboard on a new scale.
    Rescale {
        id: rescale
        onRatioChanged: Qt.callLater(note.follow)
    }

    Connections {
        target: note.ui
        function onNoticesWaiting() {
            note.take();
        }
        function onTextsChanged() {
            note.retell();
        }
        function onSettingsChanged() {
            note.retell();
        }
    }
    Component.onCompleted: take()
    onWidthChanged: follow()
    onHeightChanged: follow()

    // Takes the notices waiting in the hub and shows the banner.
    function take() {
        const got = JSON.parse(ui.takeNotices()) || [];
        if (got.length === 0)
            return;
        queue = queue.concat(got);
        if (!visible)
            Qt.callLater(open);
    }
    // The keyboard's language changed: each notice in the queue reads again in it.
    function retell() {
        const now = JSON.parse(ui.shownNotices()) || [];
        queue = queue.map(n => now.find(m => m.id === n.id) || n);
    }
    // Places the banner over the keyboard and drops it in.
    function open() {
        if (!shown)
            return;
        place();
        showing(true);
        visible = true;
        drop.restart();
        tell();
    }
    // Places the shown banner again, as the keyboard or the banner moved or changed size.
    function follow() {
        if (visible)
            place();
    }
    // Screen readers hear each notice as it shows, since the banner never takes focus.
    function tell() {
        if (shown)
            body.Accessible.announce(shown.text);
    }
    function place() {
        const at = spot(width, height);
        if (at) {
            x = at.x;
            y = at.y;
        }
    }
    // Closes the shown notice: Try again asks the app to restart what failed, OK only closes it.
    function answer(retry) {
        if (!shown)
            return;
        if (retry)
            ui.retry(shown.id);
        else
            ui.dismiss(shown.id);
        queue = queue.slice(1);
        if (shown) {
            Qt.callLater(place);
            drop.restart();
            tell();
        } else {
            visible = false;
            showing(false);
        }
    }

    TextMetrics {
        id: msgSize
        font: msg.font
        text: msg.text
    }
    TextMetrics {
        id: moreSize
        font: count.font
        text: count.text
    }

    Rectangle {
        id: body
        width: note.width
        height: note.height
        radius: note.radius
        color: note.p.pop_bg
        border.width: note.lk.contrast ? note.lk.shape.line.px : 0
        border.color: note.p.pop_line
        Accessible.role: Accessible.AlertMessage
        Accessible.name: note.shown ? note.shown.text : ""
        Accessible.description: note.more

        Icon {
            id: icon
            anchors.left: parent.left
            anchors.leftMargin: note.side
            anchors.verticalCenter: parent.verticalCenter
            icon: note.lk.icons.notice.icon
            base: note.res.icons
            ext: note.res.ext
            px: note.sh.icon_px * note.s
            tint: note.p.pop_ink
        }
        Column {
            id: words
            anchors.left: icon.right
            anchors.leftMargin: note.gap
            anchors.verticalCenter: parent.verticalCenter
            width: note.wordsW
            Label {
                id: msg
                width: parent.width
                text: note.shown ? note.shown.text : ""
                wrapMode: Text.Wrap
                font.family: note.res.fam.ui
                font.pixelSize: note.sh.font_px * note.s
                color: note.p.pop_ink
            }
            Label {
                id: count
                visible: text.length > 0
                width: parent.width
                text: note.more
                font.family: note.res.fam.ui
                font.pixelSize: note.lk.shape.panel.count_px * note.s
                font.weight: note.lk.shape.panel.count_weight
                color: note.p.pop_muted
            }
        }
        Row {
            id: buttons
            anchors.right: parent.right
            anchors.rightMargin: note.side
            anchors.verticalCenter: parent.verticalCenter
            spacing: note.gap
            NoticeButton {
                visible: !!note.shown && note.shown.retry
                text: note.res.tr("button.retry")
                main: true
                onClicked: note.answer(true)
            }
            NoticeButton {
                text: note.res.tr("button.ok")
                main: !(note.shown && note.shown.retry)
                onClicked: note.answer(false)
            }
        }
        PressWatch {
            onTouched: note.touched()
        }
    }

    // A banner button (mock-up .btn): the main one is ink on the panel, the other a ghost on the hover tint.
    component NoticeButton: Rectangle {
        id: b
        required property string text
        property bool main: false
        signal clicked
        readonly property color ink: main ? note.p.pop_bg : note.lk.sel_ink
        width: label.implicitWidth + 2 * note.sh.button_pad_px[1] * note.s
        height: label.implicitHeight + 2 * note.sh.button_pad_px[0] * note.s
        radius: note.sh.button_radius_px * note.s
        color: main ? note.p.pop_ink : note.p.pop_hover
        Accessible.role: Accessible.Button
        Accessible.name: text
        Accessible.onPressAction: b.clicked()
        Hover {
            lk: note.lk
            radius: b.radius
            visible: area.containsMouse
        }
        Label {
            id: label
            anchors.centerIn: parent
            text: b.text
            font.family: note.res.fam.ui
            font.pixelSize: note.sh.button_px * note.s
            font.weight: note.sh.button_weight
            color: b.ink
        }
        MouseArea {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onClicked: b.clicked()
            // The second click of a double click answers nothing, so a shaky click never skips the next notice.
            onDoubleClicked: {}
        }
    }

    // The banner drops in (mock-up toastIn).
    ParallelAnimation {
        id: drop
        Tween {
            target: body
            property: "opacity"
            from: 0
            to: 1
            move: note.lk.motion.moves.notice
        }
        Tween {
            target: body
            property: "y"
            from: -note.lk.motion.amount.pop_rise_px
            to: 0
            move: note.lk.motion.moves.notice
        }
    }
}
