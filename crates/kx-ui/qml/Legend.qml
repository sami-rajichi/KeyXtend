// A key's one-line text in the theme's Latin, Arabic or mark font.
import QtQuick

Label {
    // The look's font families, and whether the text is Arabic, or marks on their dotted circle.
    required property var fam
    property bool ar: false
    property bool mark: false

    font.family: mark ? fam.mark : ar ? fam.arabic : fam.latin
}
