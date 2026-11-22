// What the bar does while it listens or writes (design/voice-bar.css .st, .lc): the red dot and Listening, or Nothing
// heard yet in its place, then the language's chip; or the audio mark and Transcribing; then the chip naming what takes
// the voice, a cloud service or the model on this PC. Words follow the keyboard's language, the chip the language's own
// script; while it listens the word takes the wider of both texts' room, so one take holds one width.
import QtQuick

Row {
    id: st

    // The bar it sits on, whose view, look, fonts and size it reads.
    required property var host

    readonly property var shown: host.shown
    readonly property var cap: host.cap
    readonly property var chip: host.lk.shape.chip
    readonly property real scale: host.ar ? host.lk.look.arabic_scale : 1
    readonly property var lang: shown && shown.lang ? shown.lang : null
    readonly property var mark: host.view && host.view.mark ? host.view.mark : null
    // Where the chip's name is cut: a model's past the theme's width, and any name only where the lead would pass its
    // room; the room counts all before the name, none of which follows its width.
    readonly property real cutAt: Math.min(nameCap, nameRoom)
    readonly property real nameCap: mark && mark.cut ? host.lk.shape.voice.chip_name_px * host.s : Infinity
    readonly property real nameRoom: host.leadRoom - x - status.width - (tongue.visible ? spacing + tongue.width : 0) -
        spacing - 2 * chip.pad_px * host.s - picture.width - where.spacing

    spacing: host.gap

    // A status word in the bar's font: the one shown, and the two it may show, measured.
    component Word: Label {
        font.family: st.host.res.fam.ui
        font.pixelSize: st.cap.status_px * st.host.s * st.scale
        font.weight: st.cap.status_weight
        color: st.host.p.pop_ink
    }

    Row {
        id: status
        anchors.verticalCenter: parent.verticalCenter
        spacing: st.host.gap
        leftPadding: st.cap.inset_px * st.host.s
        rightPadding: leftPadding
        VoiceDot {
            visible: st.host.listens
            anchors.verticalCenter: parent.verticalCenter
            lk: st.host.lk
            s: st.host.s
        }
        Icon {
            visible: st.host.writes
            anchors.verticalCenter: parent.verticalCenter
            icon: st.host.lk.icons.voice.transcribing
            base: st.host.res.icons
            ext: st.host.res.ext
            px: st.cap.icon_px * st.host.s
            tint: st.host.p.pop_muted
        }
        Word {
            anchors.verticalCenter: parent.verticalCenter
            width: st.host.listens ? Math.max(word.implicitWidth, quiet.implicitWidth) : implicitWidth
            horizontalAlignment: Text.AlignLeft
            text: !st.shown || !st.shown.word ? "" : (st.shown.hint || st.shown.word)
        }
        Word {
            id: word
            visible: false
            text: st.shown && st.shown.word ? st.shown.word : ""
        }
        Word {
            id: quiet
            visible: false
            text: st.shown && st.shown.quiet ? st.shown.quiet : ""
        }
    }
    // High contrast draws a chip's word in ButtonText, as on the chip lane (drawn/voice.rs).
    Rectangle {
        id: tongue
        visible: !!st.lang
        anchors.verticalCenter: parent.verticalCenter
        width: name.width + 2 * st.chip.pad_px * st.host.s
        height: st.chip.height_px * st.host.s
        radius: height / 2
        color: st.host.p.chip_bg
        border.width: st.host.lk.shape.line.px
        border.color: st.host.p.pop_line
        Label {
            id: name
            anchors.centerIn: parent
            text: st.lang ? st.lang.name : ""
            font.family: st.lang && st.lang.arabic ? st.host.res.fam.arabic : st.host.res.fam.latin
            font.pixelSize: st.chip.font_px[st.lang && st.lang.arabic ? 1 : 0] * st.host.s
            font.weight: st.chip.weight
            color: st.host.lk.contrast ? st.host.p.legend : st.host.p.pop_ink
        }
    }
    // The chip: the service the voice goes to at Stop or the model on this PC that writes it, by its kind's picture and
    // its name, cut in the middle as `cutAt` says; readers hear it whole in a sentence.
    Rectangle {
        visible: !!st.mark
        anchors.verticalCenter: parent.verticalCenter
        width: where.width + 2 * st.chip.pad_px * st.host.s
        height: st.chip.height_px * st.host.s
        radius: height / 2
        color: st.host.p.chip_bg
        border.width: st.host.lk.shape.line.px
        border.color: st.host.p.pop_line
        Accessible.role: Accessible.StaticText
        Accessible.name: st.mark ? st.mark.said : ""
        Row {
            id: where
            anchors.centerIn: parent
            spacing: st.chip.gap_px * st.host.s
            Icon {
                id: picture
                anchors.verticalCenter: parent.verticalCenter
                icon: st.mark ? st.host.lk.icons.voice[st.mark.kind] || "" : ""
                base: st.host.res.icons
                ext: st.host.res.ext
                px: st.chip.icon_px * st.host.s
                tint: st.host.lk.contrast ? st.host.p.legend : st.host.p.ring
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(0, Math.min(implicitWidth, st.cutAt))
                elide: Text.ElideMiddle
                text: st.mark ? st.mark.name : ""
                font.family: st.host.ar ? st.host.res.fam.arabic : st.host.res.fam.latin
                font.pixelSize: st.chip.font_px[st.host.ar ? 1 : 0] * st.host.s
                font.weight: st.chip.weight
                color: st.host.lk.contrast ? st.host.p.legend : st.host.p.pop_ink
            }
        }
    }
}
