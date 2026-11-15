// The app's root: the hub's Ui, Keyboard, QuickFill and Voice objects under the app's windows; it quits once the UI has
// ended.
import QtQuick
import KeyXtend.Ui

App {
    ui: Ui {}
    keyboard: Keyboard {}
    fill: QuickFill {}
    voice: Voice {}
    onDone: Qt.quit()
}
