import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import "../components"

Item {
    id: root
    property QtObject serverManager
    property string currentId: ""

    ListModel {
        id: carsModel
    }

    Timer {
        id: pollTimer
        interval: 1000
        running: root.visible && currentId !== ""
        repeat: true
        onTriggered: {
            serverManager.poll_fetched_cars()
        }
    }

    Timer {
        id: delayedRefreshTimer
        interval: 1500
        repeat: false
        onTriggered: serverManager.fetch_cars(currentId)
    }

    onVisibleChanged: {
        if (visible && currentId !== "") {
            serverManager.fetch_cars(currentId)
        }
    }

    onCurrentIdChanged: {
        if (visible && currentId !== "") {
            serverManager.fetch_cars(currentId)
        }
    }

    Connections {
        target: serverManager
        function onCars_jsonChanged() {
            var json = serverManager.cars_json
            if (!json || json === "") return;
            try {
                var arr = JSON.parse(json)
                carsModel.clear()
                for (var i = 0; i < arr.length; i++) {
                    carsModel.append(arr[i])
                }
            } catch (e) {
                console.log("Failed to parse cars json", e)
            }
        }
    }

    Timer {
        id: uploadPollTimer
        interval: 500
        running: false
        repeat: true
        onTriggered: {
            var status = serverManager.poll_car_upload_status()
            if (status === "idle" || status === "") return;
            
            if (status.startsWith("uploading")) {
                var parts = status.split("|")
                if (parts.length > 1) {
                    var pct = parseInt(parts[1])
                    progressText.text = "Uploading... " + pct + "%"
                    progressBar.value = pct
                    progressBar.indeterminate = false
                } else {
                    progressText.text = "Uploading..."
                    progressBar.indeterminate = true
                }
            } else if (status === "unpacking") {
                progressText.text = "Unpacking files on server..."
                progressBar.value = 100
                progressBar.indeterminate = true
            } else if (status === "success") {
                progressText.text = "Car uploaded successfully!"
                progressText.color = "#4ade80"
                progressBar.value = 100
                progressBar.indeterminate = false
                progressCloseButton.visible = true
                running = false
                delayedRefreshTimer.start()
            } else if (status.startsWith("failed")) {
                var failParts = status.split("|")
                var reason = failParts.length > 1 ? failParts[1] : "Unknown error"
                progressText.text = "Upload failed:\n" + reason
                progressText.color = "#f87171"
                progressBar.value = 0
                progressBar.indeterminate = false
                progressCloseButton.visible = true
                running = false
            }
            
            var logs = serverManager.poll_car_upload_log()
            if (logs && logs.length > 0) {
                logArea.appendLog(logs)
            }
        }
    }

    Popup {
        id: progressPopup
        width: 600
        height: 500
        modal: true
        closePolicy: Popup.NoAutoClose
        anchors.centerIn: Overlay.overlay
        
        background: Rectangle {
            color: "#1e293b"
            radius: 8
            border.color: "#334155"
            border.width: 1
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 20
            spacing: 15

            Text {
                text: "Car Upload Progress"
                color: "white"
                font.pixelSize: 18
                font.bold: true
            }

            Text {
                id: progressText
                text: "Initializing..."
                color: "#cbd5e1"
                font.pixelSize: 14
                Layout.fillWidth: true
                wrapMode: Text.Wrap
            }

            ProgressBar {
                id: progressBar
                Layout.fillWidth: true
                from: 0
                to: 100
                value: 0
            }

            TerminalOutput {
                id: logArea
                Layout.fillWidth: true
                Layout.fillHeight: true
            }

            Button {
                id: progressCloseButton
                text: "Close"
                visible: false
                Layout.alignment: Qt.AlignRight
                onClicked: progressPopup.close()
            }
        }
    }

    Popup {
        id: confirmDeletePopup
        width: 400
        height: 180
        modal: true
        anchors.centerIn: Overlay.overlay
        
        property string folderToDelete: ""

        background: Rectangle {
            color: "#1e293b"
            radius: 8
            border.color: "#334155"
            border.width: 1
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 20
            spacing: 15

            Text {
                text: "Potwierdzenie usunięcia"
                color: "white"
                font.pixelSize: 18
                font.bold: true
            }

            Text {
                text: "Zostanie usunięty cały folder auta: <b>" + confirmDeletePopup.folderToDelete + "</b>"
                color: "#cbd5e1"
                font.pixelSize: 14
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                textFormat: Text.RichText
            }
            
            Item {
                Layout.fillHeight: true
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
                spacing: 10
                
                Button {
                    text: "Anuluj"
                    onClicked: confirmDeletePopup.close()
                }
                
                Button {
                    text: "Usuń"
                    onClicked: {
                        serverManager.delete_car(currentId, confirmDeletePopup.folderToDelete)
                        delayedRefreshTimer.start()
                        confirmDeletePopup.close()
                    }
                }
            }
        }
    }

    FileDialog {
        id: uploadDialog
        title: "Select Car Mod (ZIP)"
        nameFilters: ["ZIP archives (*.zip)"]
        onAccepted: {
            progressText.text = "Starting..."
            progressText.color = "#cbd5e1"
            progressBar.value = 0
            progressBar.indeterminate = true
            progressCloseButton.visible = false
            logArea.text = ""
            progressPopup.open()
            
            serverManager.upload_car(currentId, uploadDialog.selectedFile.toString())
            uploadPollTimer.start()
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.margins
        spacing: Theme.spacing

        RowLayout {
            Layout.fillWidth: true

            Text {
                text: "Car Manager"
                font.family: Theme.fontFamily
                font.pixelSize: 28
                font.bold: true
                color: Theme.textMain
                Layout.fillWidth: true
            }

            SecondaryButton {
                text: "Refresh"
                onClicked: {
                    carsModel.clear()
                    serverManager.fetch_cars(currentId)
                }
            }

            Item {
                Layout.fillWidth: true
            }

            PrimaryButton {
                text: "Upload Car"
                onClicked: uploadDialog.open()
            }
        }
        
        Text {
            text: "Loading data..."
            color: Theme.textSecondary
            font.family: Theme.fontFamily
            font.pixelSize: 18
            Layout.alignment: Qt.AlignHCenter | Qt.AlignVCenter
            Layout.fillHeight: true
            visible: carsModel.count === 0
        }

        GridView {
            Layout.fillHeight: true
            Layout.fillWidth: true
            model: carsModel
            clip: true
            cellWidth: 320
            cellHeight: 280
            visible: carsModel.count > 0

            ScrollBar.vertical: ScrollBar {
                active: true
            }

            delegate: Card {
                width: 300
                height: 260

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 10
                    spacing: 5

                    Rectangle {
                        color: Theme.bgHover
                        Layout.fillWidth: true
                        Layout.preferredHeight: 150
                        radius: Theme.radius
                        clip: true

                        Text {
                            anchors.centerIn: parent
                            text: "No preview available"
                            color: Theme.textSecondary
                            font.pixelSize: 14
                            font.bold: true
                            font.family: Theme.fontFamily
                            visible: !model.thumbnail_base64 || model.thumbnail_base64 === ""
                        }

                        Image {
                            anchors.fill: parent
                            source: model.thumbnail_base64 || ""
                            fillMode: Image.PreserveAspectCrop
                            visible: model.thumbnail_base64 && model.thumbnail_base64 !== ""
                        }
                    }

                    Text {
                        text: model.name || "Unknown Car"
                        color: Theme.textMain
                        font.bold: true
                        font.pixelSize: 16
                        font.family: Theme.fontFamily
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }

                    Text {
                        text: "Folder: " + model.car_folder
                        color: Theme.textSecondary
                        font.pixelSize: 12
                        font.family: Theme.fontFamily
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        Item { Layout.fillWidth: true }
                        DangerButton {
                            text: "Delete"
                            onClicked: {
                                var folder = model.car_folder;
                                confirmDeletePopup.folderToDelete = folder;
                                confirmDeletePopup.open();
                            }
                        }
                    }
                }
            }
        }
    }
}
