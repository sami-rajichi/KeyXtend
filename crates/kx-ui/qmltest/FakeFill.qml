// Stands in for the hub's QuickFill object: it answers with a view from the Quick-fill fixtures and records each click.
// qmltestrunner cannot load KeyXtend.Ui, which is linked into Rust binaries.
import QtQuick

QtObject {
    // The panel's view as the hub writes it, null while closed; setting it sends `viewChanged`.
    property var shown: null
    // Each click as `kind id arg` (`kind arg` with no id), and the round each named, oldest first.
    property var calls: []
    property var rounds: []

    // The panel shows a new view or closes: read `view` again.
    signal viewChanged

    onShownChanged: viewChanged()

    function view() {
        return JSON.stringify(shown);
    }
    function act(kind, id, arg, round) {
        rounds = rounds.concat([round]);
        calls = calls.concat([[kind].concat(id ? [id] : [], [arg]).join(" ")]);
    }
}
