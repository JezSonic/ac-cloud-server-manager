import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    color: "#1e2136"
    radius: 16
    border.color: "#2e324d"

    property var serverManager
    signal editRequested(var profile)
    signal connectRequested(string id)

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.margins
        spacing: 20

        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: 50
            Text {
                id: titleText
                text: serverManager.t("title.servers")
                color: Theme.textMain
                font.bold: true
                font.pixelSize: 24
                font.family: Theme.fontFamily
                Layout.fillWidth: true
            }

            PrimaryButton {
                text: serverManager.t("button.add_server")
                Layout.preferredHeight: 40
                Layout.preferredWidth: 150
                onClicked: {
                    root.editRequested(null)
                }
            }
        }

        ListView {
            id: profilesList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: root.serverManager ? JSON.parse(root.serverManager.profiles_json) : []
            spacing: 10
            
            ScrollBar.vertical: ScrollBar {
                active: true
            }

            delegate: Item {
                width: profilesList.width
                height: 80
                
                Rectangle {
                    anchors.fill: parent
                    radius: Theme.radius
                    color: profileMouseArea.containsMouse ? Theme.bgHover : Theme.bgMain
                    border.color: Theme.primary
                    border.width: profileMouseArea.containsMouse ? 1 : 0
                    
                    Behavior on color { ColorAnimation { duration: 150 } }

                    MouseArea {
                        id: profileMouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: { root.editRequested(modelData) }
                        z: 0
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 10
                        z: 1

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2
                            clip: true
                            Text {
                                text: modelData.name
                                color: Theme.textMain
                                font.bold: true
                                font.pixelSize: 15
                                font.family: Theme.fontFamily
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }
                            Text {
                                text: modelData.username + "@" + modelData.host
                                color: Theme.textSecondary
                                font.pixelSize: 12
                                font.family: Theme.fontFamily
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }
                        }

                        PrimaryButton {
                            text: serverManager.t("button.edit")
                            onClicked: {
                                root.editRequested(modelData)
                            }
                        }

                        SuccessButton {
                            text: serverManager.t("button.connect")
                            onClicked: {
                                root.connectRequested(modelData.id)
                                if (root.serverManager) {
                                    root.serverManager.connect_to_server(modelData.id)
                                }
                            }
                        }
                    }
                }
            }
        }

    }
}
