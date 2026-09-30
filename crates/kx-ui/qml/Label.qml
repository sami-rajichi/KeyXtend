// Text in a theme font with grayscale edges: ClearType's colour fringes showed on thin text in our see-through,
// non-activating windows. Readers skip it, since the control around it carries the name.
import QtQuick

Text {
    textFormat: Text.PlainText
    renderType: Text.CurveRendering
    Accessible.ignored: true
}
