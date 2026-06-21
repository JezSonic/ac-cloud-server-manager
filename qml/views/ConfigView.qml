import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Item {
    id: configView

    property QtObject serverManager
    property string currentId: ""
    property string validationMessage: ""

    ListModel {
        id: trackModel
    }

    Timer {
        id: pollTimer
        interval: 1000
        running: configView.visible && currentId !== ""
        repeat: true
        onTriggered: {
            serverManager.poll_fetched_configs()
            serverManager.poll_fetched_tracks()
            var status = serverManager.poll_config_save_status()
            if (status !== "") {
                validationMessage = status
            }
        }
    }

    onVisibleChanged: {
        if (visible && currentId !== "") {
            serverManager.fetch_server_configs(currentId)
            serverManager.fetch_tracks(currentId)
        }
    }

    onCurrentIdChanged: {
        if (visible && currentId !== "") {
            serverManager.fetch_server_configs(currentId)
            serverManager.fetch_tracks(currentId)
        }
    }

    Connections {
        target: serverManager
        function onTracks_jsonChanged() {
            var json = serverManager.tracks_json
            if (!json || json === "") return;
            try {
                var arr = JSON.parse(json)
                trackModel.clear()
                for (var i = 0; i < arr.length; i++) {
                    var name = arr[i].name || "Unknown"
                    if (arr[i].layout) {
                        name += " (" + arr[i].layout + ")"
                    }
                    trackModel.append({
                        "displayName": name,
                        "track_folder": arr[i].track_folder,
                        "layout": arr[i].layout || "",
                        "thumbnail_base64": arr[i].thumbnail_base64 || ""
                    })
                }
            } catch (e) {
                console.log("Failed to parse tracks in config", e)
            }
            syncTrackComboWithConfig()
        }
    }

    Timer {
        id: clearMessageTimer
        interval: 3000
        repeat: false
        onTriggered: {
            if (validationMessage.indexOf("Configs saved") !== -1) {
                validationMessage = ""
            }
        }
    }

    onValidationMessageChanged: {
        if (validationMessage.indexOf("Configs saved") !== -1) {
            clearMessageTimer.start()
        }
    }

    function updateTrackInConfig(track_folder, layout) {
        var lines = serverCfgArea.text.split("\n")
        var trackUpdated = false
        var layoutUpdated = false
        for (var i = 0; i < lines.length; i++) {
            if (lines[i].startsWith("TRACK=")) {
                lines[i] = "TRACK=" + track_folder
                trackUpdated = true
            } else if (lines[i].startsWith("CONFIG_TRACK=")) {
                lines[i] = "CONFIG_TRACK=" + layout
                layoutUpdated = true
            }
        }

        // If they didn't exist, append to [SERVER] section
        if (!trackUpdated || !layoutUpdated) {
            var serverIdx = -1
            for (var j = 0; j < lines.length; j++) {
                if (lines[j].trim() === "[SERVER]") {
                    serverIdx = j; break;
                }
            }
            if (serverIdx !== -1) {
                if (!trackUpdated) lines.splice(serverIdx + 1, 0, "TRACK=" + track_folder)
                if (!layoutUpdated) lines.splice(serverIdx + 2, 0, "CONFIG_TRACK=" + layout)
            }
        }
        serverCfgArea.text = lines.join("\n")
    }

    function syncTrackComboWithConfig() {
        if (!serverManager.server_cfg_content || trackModel.count === 0) return;
        var lines = serverManager.server_cfg_content.split("\n")
        var currentTrack = ""
        var currentLayout = ""
        for (var i = 0; i < lines.length; i++) {
            var line = lines[i].trim()
            if (line.startsWith("TRACK=")) {
                currentTrack = line.substring(6).trim()
            } else if (line.startsWith("CONFIG_TRACK=")) {
                currentLayout = line.substring(13).trim()
            }
        }
        
        for (var j = 0; j < trackModel.count; j++) {
            var track = trackModel.get(j)
            if (track.track_folder === currentTrack && track.layout === currentLayout) {
                trackCombo.currentIndex = j
                return
            }
        }
    }

    Connections {
        target: serverManager
        function onServer_cfg_contentChanged() {
            syncTrackComboWithConfig()
            if (serverCfgArea.text !== serverManager.server_cfg_content) {
                serverCfgArea.text = serverManager.server_cfg_content
            }
        }
        function onEntry_list_contentChanged() {
            if (entryListArea.text !== serverManager.entry_list_content) {
                entryListArea.text = serverManager.entry_list_content
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.margins
        spacing: Theme.spacing

        Text {
            text: serverManager.t("sidebar.config") || "Server Configuration"
            font.family: Theme.fontFamily
            font.pixelSize: 28
            font.bold: true
            color: Theme.textMain
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            Text {
                text: "Select Track:"
                color: "white"
                font.bold: true
            }

            ComboBox {
                id: trackCombo
                Layout.preferredWidth: 350
                model: trackModel
                textRole: "displayName"

                delegate: ItemDelegate {
                    width: trackCombo.width
                    height: 50
                    contentItem: RowLayout {
                        spacing: 10
                        Image {
                            source: model.thumbnail_base64
                            Layout.preferredWidth: 60
                            Layout.preferredHeight: 40
                            fillMode: Image.PreserveAspectCrop
                            clip: true
                        }
                        Text {
                            text: model.displayName
                            color: "white"
                            font.bold: true
                            Layout.fillWidth: true
                            elide: Text.ElideRight
                        }
                    }
                    background: Rectangle {
                        color: parent.highlighted ? "#334155" : "#1e293b"
                    }
                }

                onActivated: function(index) {
                    var track = trackModel.get(index)
                    updateTrackInConfig(track.track_folder, track.layout)
                }
            }

            Item { Layout.fillWidth: true }

            SecondaryButton {
                text: "Reload from Server"
                onClicked: {
                    serverManager.fetch_server_configs(currentId)
                    serverManager.fetch_tracks(currentId)
                }
            }

            PrimaryButton {
                text: "Validate"
                onClicked: {
                    validationMessage = serverManager.validate_server_configs(serverCfgArea.text, entryListArea.text)
                }
            }

            SuccessButton {
                text: "Save & Restart Server"
                onClicked: {
                    var validation = serverManager.validate_server_configs(serverCfgArea.text, entryListArea.text)
                    if (validation === "Valid") {
                        validationMessage = "Saving..."
                        serverManager.save_server_configs(currentId, serverCfgArea.text, entryListArea.text)
                    } else {
                        validationMessage = "Fix errors before saving: " + validation
                    }
                }
            }
        }

        Text {
            text: validationMessage
            color: (validationMessage === "Valid" || validationMessage.indexOf("Configs saved") !== -1) ? "#22c55e" : (validationMessage === "" || validationMessage === "Saving..." ? "white" : "#ef4444")
            font.pixelSize: 14
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }

        SplitView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            orientation: Qt.Horizontal

            ColumnLayout {
                SplitView.preferredWidth: parent.width / 2
                spacing: 5

                Text {
                    text: "server_cfg.ini"
                    color: "white"
                    font.bold: true
                }

                ScrollView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true

                    TextArea {
                        id: serverCfgArea
                        text: serverManager.server_cfg_content
                        color: "#e2e8f0"
                        background: Rectangle { color: "#1e293b"; radius: 4 }
                        font.family: "monospace"
                        wrapMode: TextArea.NoWrap
                    }
                }
            }

            ColumnLayout {
                SplitView.preferredWidth: parent.width / 2
                spacing: 5

                Text {
                    text: "entry_list.ini"
                    color: "white"
                    font.bold: true
                }

                ScrollView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true

                    TextArea {
                        id: entryListArea
                        text: serverManager.entry_list_content
                        color: "#e2e8f0"
                        background: Rectangle { color: "#1e293b"; radius: 4 }
                        font.family: "monospace"
                        wrapMode: TextArea.NoWrap
                    }
                }
            }
        }
    }
}
