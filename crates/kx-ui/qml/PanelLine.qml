// A list panel's line, for search and Quick-fill's name, values and own labels: never focused, and its editor only draws
// (ADR-0013). It takes the letters clicked on our keyboard only while aimed at, which a press on its panel asks for
// (ADR-0040). It shows its text, the mask or its hint, which hides once it takes the letters, with a caret, a ring-coloured
// line under it and a ring around it while it takes them, as WinUI's focused TextBox draws its accent line.
import QtQuick

Rectangle {
    id: line

    // Its text (`{text, rtl, arabic}`, or null for the mask), its hint while empty, a caption before it and the share
    // of the width it may take, its icon, whether it takes the letters now, whether a click may aim them at it (no while
    // a question waits), the mask, the characters under it (0 draws the mask as it is) and its letter spacing (a share
    // of its size), whether the keyboard's language reads right to left, the look, fonts, icons and size, its gate id
    // and the name readers hear.
    required property var said
    required property string hint
    property string caption: ""
    property real share: 1
    property string icon: ""
    property bool takes: false
    property bool aims: true
    property string mask: ""
    property int count: 0
    property real maskEm: 0
    // A search line keeps its hint beside the caret until a letter comes, as that hint alone says what the line does.
    property bool search: false
    required property bool rtl
    required property var lk
    required property var res
    required property real s
    property string uid: ""
    property string name: caption || hint
    // Its text is a secret that shows on screen: readers hear the mask while it holds any.
    property bool secret: false
    // Where the module puts the caret while it takes the letters: UTF-16 units of what it draws, or dots under a mask.
    property int at: 0
    // How far the caret stands from the side the text hugs, and how far the text has moved under it from that side.
    property real from: 0
    property real moved: 0
    // The caret a press under the pointer found as it turned this line on, or -1: the text stays under the pointer until
    // the click's own place comes, so the click lands on the letter seen there.
    property int kept: -1

    // A click picked this line, then says which place in what it draws.
    signal clicked
    signal placed(int at)

    // A reader's press, which has no pointer: its panel's window, where one can, asks the keyboard for the aim, as a
    // press there does, then its page hears which line.
    function aim() {
        const panel = line.Window.window;
        if (panel && panel.aiming)
            panel.aiming();
        line.clicked();
    }
    // Moves the text under the caret only as far as keeps the caret in sight, as a text box does. Qt's own scroll jumps
    // at each new text, so the text is laid out whole and this moves it.
    function follow() {
        const wide = field.width;
        const place = Math.min(takes ? at : drawn.length, field.length);
        const x = masked ? place * wide / Math.max(1, field.length) : field.positionToRectangle(place).x;
        from = wrote ? wide - x : x;
        if (at !== kept)
            kept = -1;
        const most = Math.max(0, wide - room);
        moved = kept < 0 ? Math.max(0, Math.min(most, from, Math.max(moved, from - room))) : Math.min(moved, most);
    }
    // The place in what it draws at `x` in the editor. Dots are alike and count from the left, as Left goes back on them,
    // however Qt lays out what holds no direction.
    function placeAt(x) {
        if (!masked)
            return field.positionAt(x, field.height / 2);
        return Math.max(0, Math.min(drawn.length, Math.round(x * drawn.length / Math.max(1, field.width))));
    }

    readonly property var f: lk.shape.panel.field_px
    readonly property var ln: lk.shape.panel.line_px
    readonly property var p: lk.palette
    readonly property bool masked: said === null
    // One dot per character on New and Edit, in the mask's first character; readers only ever hear the mask itself.
    readonly property string dots: count > 0 ? mask.charAt(0).repeat(count) : mask
    readonly property string drawn: masked ? dots : said.text
    readonly property bool typed: masked || said.text.length > 0
    // A mask starts on the keyboard language's side, so the caret an empty hidden line waits with stays put.
    readonly property bool wrote: masked ? rtl : said.rtl
    // The room the text may fill, as the caret keeps its own width clear.
    readonly property real room: box.width - caret.width
    readonly property real pad: f[2] * s
    // The hint keeps clear of the caret, which waits on the language's start side.
    readonly property real clear: (ln[1] + ln[2]) * s
    // Under the pointer it takes pop_hover, Highlight in high contrast, so every ink there is sel_ink (as ClipRow's).
    readonly property bool hot: aims && area.containsMouse
    readonly property color ink: hot ? lk.sel_ink : p.pop_ink
    readonly property color muted: hot ? lk.sel_ink : p.pop_muted
    readonly property color mark: hot ? lk.sel_ink : p.ring

    height: f[0] * s
    radius: f[1] * s
    color: "transparent"
    border.width: takes ? lk.shape.press.ring_px * s : lk.shape.line.px
    border.color: takes ? p.ring : p.pop_line
    Accessible.role: Accessible.EditableText
    Accessible.name: name
    Accessible.description: masked || (secret && typed) ? mask : said.text
    Accessible.id: uid
    Accessible.onPressAction: if (line.aims) line.aim()
    onAtChanged: follow()
    // Keeps the caret in sight, or the end a line that does not take the letters shows; a press under the pointer keeps
    // the text instead.
    onTakesChanged: {
        kept = takes && area.containsMouse ? at : -1;
        follow();
    }
    onWroteChanged: follow()
    onRoomChanged: follow()
    Component.onCompleted: follow()

    Hover {
        lk: line.lk
        radius: line.radius
        visible: line.hot
        tint: line.p.pop_hover
    }
    Row {
        id: lead
        x: line.pad
        anchors.verticalCenter: parent.verticalCenter
        spacing: line.pad
        Icon {
            visible: !!line.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: line.icon
            base: line.res.icons
            ext: line.res.ext
            px: line.ln[0] * line.s
            tint: line.muted
        }
        Label {
            visible: !!line.caption
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(implicitWidth, line.width * line.share)
            text: line.caption
            elide: Text.ElideRight
            font.family: line.res.fam.ui
            font.pixelSize: line.lk.shape.panel.hint_px[0] * line.s
            color: line.muted
        }
    }
    Item {
        id: box
        anchors.left: lead.visibleChildren.length > 0 ? lead.right : parent.left
        anchors.leftMargin: line.pad
        anchors.right: parent.right
        anchors.rightMargin: line.pad
        height: parent.height
        clip: true

        Label {
            visible: !line.typed && (line.search || !line.takes)
            x: line.takes && !line.rtl ? line.clear : 0
            width: parent.width - (line.takes ? line.clear : 0)
            anchors.verticalCenter: parent.verticalCenter
            text: line.hint
            elide: Text.ElideRight
            font.family: line.res.fam.ui
            font.pixelSize: line.f[3] * line.s
            color: line.muted
        }
        // While it does not take the letters, the newest stay in sight: the text gives up its start, and Qt elides in the
        // text's own order, so that is ElideLeft in either direction.
        Label {
            id: words
            visible: line.typed && !line.takes
            width: parent.width
            anchors.verticalCenter: parent.verticalCenter
            text: line.drawn
            horizontalAlignment: line.wrote ? Text.AlignRight : Text.AlignLeft
            elide: Text.ElideLeft
            font.family: line.said && line.said.arabic ? line.res.fam.arabic : line.res.fam.latin
            font.pixelSize: line.f[3] * line.s
            font.letterSpacing: line.masked ? line.maskEm * line.f[3] * line.s : 0
            color: line.masked ? line.muted : line.ink
        }
        // Only Qt's text layout places a caret exactly in joined Arabic and mixed text, and only an editor lets one read
        // it; this one only draws, and never takes focus or a press (ADR-0013).
        TextEdit {
            id: field
            visible: line.typed && line.takes
            x: line.wrote ? (line.masked ? line.room : box.width) - width + line.moved : -line.moved
            anchors.verticalCenter: parent.verticalCenter
            text: line.drawn
            textFormat: Text.PlainText
            horizontalAlignment: line.wrote ? TextEdit.AlignRight : TextEdit.AlignLeft
            font: words.font
            color: words.color
            renderType: TextEdit.CurveRendering
            readOnly: true
            enabled: false
            activeFocusOnPress: false
            activeFocusOnTab: false
            cursorVisible: false
            selectByMouse: false
            Accessible.ignored: true
            onTextChanged: line.follow()
            onContentSizeChanged: line.follow()
        }
        Rectangle {
            id: caret
            // Marks the caret for the tests.
            readonly property bool caret: true
            // Where the layout puts its place, on the side the text reads to; on the hint's start side while nothing is typed.
            readonly property real inset: line.from - line.moved
            visible: line.takes
            x: !line.typed ? (line.rtl ? box.width - width : 0) : line.wrote ? line.room - inset : inset
            anchors.verticalCenter: parent.verticalCenter
            width: line.ln[1] * line.s
            height: line.f[3] * line.s
            color: line.mark
        }
    }
    Rectangle {
        visible: line.takes
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        width: parent.width - 2 * line.radius
        height: line.ln[3] * line.s
        color: line.mark
    }
    Act {
        id: area
        anchors.fill: parent
        enabled: line.aims
        hoverEnabled: true
        onActed: at => {
            const place = line.placeAt(area.mapToItem(field, at).x);
            line.clicked();
            line.placed(place);
        }
    }
}
