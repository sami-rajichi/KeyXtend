// A soft shadow: Qt's RectangularShadow, which takes Qt's default material as its own at once, as Qt 6.11 never frees
// that material (cpp/shadows.h serves `shadows`).
import QtQuick
import QtQuick.Effects

RectangularShadow {
    id: soft

    Component.onCompleted: shadows.adopt(soft)
}
