// The gate-id group of the selection strip's keycaps, held once in QML: tst_selchips checks it against the strip's text
// keys, and kx-gates' P10 tests check the group their config names.
import QtQuick

QtObject {
    // A keycap's id is its chip's in this group, so it reads as its word's text key, the same in every language.
    readonly property string group: "sel"
}
