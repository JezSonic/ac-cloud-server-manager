import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Dialog {
    id: root
    title: serverManager ? serverManager.t("dialog.install_log_title") : "Logs"
    width: 600
    height: 400
    modal: true
    anchors.centerIn: parent

    property QtObject serverManager

    background: Rectangle {
        color: "#1e293b"
        radius: 8
        border.color: "#334155"
    }

    contentItem: ColumnLayout {
        spacing: 15
        
        TerminalOutput {
            id: logArea
            Layout.fillWidth: true
            Layout.fillHeight: true
            
            Timer {
                id: logTimer
                interval: 500
                running: root.visible
                repeat: true
                onTriggered: {
                    if (!serverManager) return;
                    var logs = serverManager.poll_bootstrap_log();
                    if (logs.indexOf("__DONE__") !== -1) {
                        logs = logs.replace("__DONE__", "");
                        logArea.appendLog(logs);
                        logArea.appendLog(serverManager ? serverManager.t("dialog.install_success") : "\n\n[SUCCESS]");
                        logTimer.stop();
                    } else if (logs.indexOf("__FAILED__") !== -1) {
                        logs = logs.replace("__FAILED__", "");
                        logArea.appendLog(logs);
                        logArea.appendLog(serverManager ? serverManager.t("dialog.install_error") : "\n\n[ERROR]");
                        logTimer.stop();
                    } else if (logs.length > 0) {
                        logArea.appendLog(logs);
                    }
                }
            }
        }
    }
    
    onOpened: {
        logArea.text = "";
        logTimer.start();
    }
}
