import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Window {
    id: root
    title: serverManager.t("dialog.dep_title")
    width: 600
    height: 400
    color: "#1e1e2e"
    flags: Qt.Dialog | Qt.Window
    modality: Qt.ApplicationModal

    property var serverManager
    property string currentId
    property var depsList: []
    property bool isInstalling: false
    property string rawLog: ""

    onVisibleChanged: {
        if (visible) {
            isInstalling = false;
            rawLog = "";
            terminalOutput.text = "";
            logPoller.stop();
            
            try {
                let parsed = JSON.parse(serverManager.missing_dependencies_json);
                if (parsed && parsed.length > 0) {
                    depsList = parsed;
                }
            } catch(e) {
                depsList = [];
            }
        }
    }

    Timer {
        id: logPoller
        interval: 100
        repeat: true
        running: false
        onTriggered: {
            if (root.serverManager) {
                let chunk = root.serverManager.poll_bootstrap_log();
                if (chunk !== "") {
                    let doneIndex = chunk.indexOf("__DONE__");
                    let failIndex = chunk.indexOf("__FAILED__");
                    
                    let isDone = doneIndex !== -1 || failIndex !== -1;
                    if (isDone) {
                        logPoller.stop();
                        chunk = chunk.replace("__DONE__", "").replace("__FAILED__", "");
                    }
                    if (chunk !== "") {
                        terminalOutput.appendLog(chunk);
                    }
                    
                    if (isDone && terminalOutput.text === "") {
                        terminalOutput.appendLog(failIndex !== -1 ? "[ERROR] Process failed without output." : "[SUCCESS] Process finished silently.");
                    }
                }
            }
        }
    }

    StackLayout {
        anchors.fill: parent
        anchors.margins: 20
        currentIndex: isInstalling ? 1 : 0

        // State 0: Editing Commands
        ColumnLayout {
            spacing: 15

            Text {
                text: serverManager.t("dialog.dep_desc")
                color: "#f8fafc"
                wrapMode: Text.WordWrap
                font.pixelSize: 16
                Layout.fillWidth: true
            }
            
            ListView {
                id: listView
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: root.depsList
                spacing: 10

                delegate: ColumnLayout {
                    width: listView.width
                    spacing: 5
                    
                    Text {
                        text: "- " + modelData.name
                        color: "#ef4444"
                        font.bold: true
                    }
                    TextField {
                        id: cmdField
                        Layout.fillWidth: true
                        text: modelData.cmd
                        color: "#fff"
                        padding: 10
                        background: Rectangle { 
                            color: "#0f111a"; 
                            radius: 6; 
                            border.color: cmdField.activeFocus ? "#3b82f6" : "#2e324d" 
                        }
                        onTextChanged: {
                            root.depsList[index].cmd = text;
                        }
                    }
                }
            }

            Text {
                text: serverManager.t("dialog.dep_hint")
                color: "#94a3b8"
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignRight
                spacing: 15

                Button {
                    text: serverManager.t("button.cancel")
                    padding: 10
                    background: Rectangle {
                        radius: 6
                        color: parent.hovered ? "#ef4444" : "transparent"
                        border.color: "#ef4444"
                    }
                    contentItem: Text {
                        text: parent.text
                        color: parent.hovered ? "#ffffff" : "#ef4444"
                    }
                    onClicked: {
                        if (root.serverManager) {
                            root.serverManager.missing_dependencies_json = "[]";
                            root.serverManager.status = "Bootstrap cancelled by user.";
                        }
                        root.close();
                    }
                }

                Button {
                    text: serverManager.t("button.accept_bootstrap")
                    padding: 10
                    background: Rectangle {
                        radius: 6
                        color: parent.hovered ? "#3b82f6" : "#2563eb"
                    }
                    contentItem: Text {
                        text: parent.text
                        color: "#ffffff"
                        font.bold: true
                    }
                    onClicked: {
                        isInstalling = true;
                        logPoller.start();
                        if (root.serverManager) {
                            root.serverManager.approve_bootstrap(root.currentId, JSON.stringify(root.depsList));
                        }
                    }
                }
            }
        }

        // State 1: Terminal Log
        ColumnLayout {
            spacing: 15

            Text {
                text: serverManager.t("dialog.dep_output")
                color: "#f8fafc"
                font.bold: true
                font.pixelSize: 16
                Layout.fillWidth: true
            }

            TerminalOutput {
                id: terminalOutput
                Layout.fillWidth: true
                Layout.fillHeight: true
            }

            Button {
                text: logPoller.running ? serverManager.t("button.installing") : serverManager.t("button.close")
                Layout.alignment: Qt.AlignRight
                enabled: !logPoller.running
                padding: 10
                background: Rectangle {
                    radius: 6
                    color: parent.enabled ? (parent.hovered ? "#3b82f6" : "#2563eb") : "#475569"
                }
                contentItem: Text {
                    text: parent.text
                    color: parent.enabled ? "#ffffff" : "#94a3b8"
                    font.bold: true
                }
                onClicked: {
                    root.close();
                }
            }
        }
    }
    
    // Prevent accidental closing while installing
    onClosing: (close) => {
        if (isInstalling && logPoller.running) {
            close.accepted = false;
        }
    }
}
