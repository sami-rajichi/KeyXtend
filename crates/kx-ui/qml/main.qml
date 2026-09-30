// The app's root: the Ui object, which hears the app's commands, and the keyboard's bridge.
import QtQuick
import KeyXtend.Ui

QtObject {
    id: root

    property Ui ui: Ui {
        onQuitAsked: Qt.quit()
    }
    property Keyboard keyboard: Keyboard {}
}
