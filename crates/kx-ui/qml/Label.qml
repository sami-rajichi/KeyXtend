// Text in a theme font, drawn as exact curves with grayscale edges, since ClearType's colour fringes showed in our
// see-through windows; laid out unhinted as it is drawn, so it measures the same at every display scale. Readers skip
// it, since the control around it carries the name.
import QtQuick

Text {
    textFormat: Text.PlainText
    renderType: Text.CurveRendering
    font.hintingPreference: Font.PreferNoHinting
    Accessible.ignored: true
}
