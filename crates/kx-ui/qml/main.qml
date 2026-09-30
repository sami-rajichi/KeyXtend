// The app's root: the hub's Ui and Keyboard objects under the app's windows; it quits once the UI has ended.
import QtQuick
import KeyXtend.Ui

App {
    ui: Ui {}
    keyboard: Keyboard {}
    onDone: Qt.quit()
}
