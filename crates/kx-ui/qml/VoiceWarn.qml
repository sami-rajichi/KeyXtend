// The card's warning under the dictation bar's row on the first dictation to its service: a reminder in the warning
// tone, in the keyboard's language, wrapped in full so its account switch is never cut.
import QtQuick

Label {
    // The bar it sits in, whose warning, look, fonts and size it reads.
    required property var host

    visible: host.warns
    x: host.pad + host.cap.inset_px * host.s
    width: parent.width - 2 * x
    bottomPadding: host.cap.words_pad_px * host.s
    text: host.warning
    wrapMode: Text.Wrap
    horizontalAlignment: host.ar ? Text.AlignRight : Text.AlignLeft
    font.family: host.res.fam.ui
    font.pixelSize: host.lk.shape.notice.font_px * host.s * (host.ar ? host.lk.look.arabic_scale : 1)
    color: host.p.warn_ink
}
