// The app's root: the hub's Ui, Keyboard and QuickFill objects under the app's windows; it quits once the UI has ended.
import QtQuick
import KeyXtend.Ui

App {
    ui: Ui {}
    keyboard: Keyboard {}
    fill: QuickFill {}
    onDone: Qt.quit()
}
