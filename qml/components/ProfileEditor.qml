import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    color: "#1e2136"
    radius: 16
    border.color: "#2e324d"

    property var serverManager
    property string currentId: ""

    function loadProfile(p) {
        if (p) {
            currentId = p.id;
            nameField.text = p.name;
            hostField.text = p.host;
            userField.text = p.username;
            keyField.text = p.key_path || "";
        } else {
            clearAndShow();
        }
    }

    function clearAndShow() {
        currentId = "ac-" + Math.floor(Math.random() * 10000);
        nameField.text = "New Server";
        hostField.text = "192.168.1.100";
        userField.text = "root";
        keyField.text = "";
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.margins
        spacing: 20

        Text {
            id: editorTitle
            text: root.currentId !== "" && !root.currentId.startsWith("ac-") ? serverManager.t("profile.edit") : serverManager.t("profile.add")
            color: Theme.textMain
            font.bold: true
            font.pixelSize: 24
            font.family: Theme.fontFamily
            Layout.fillWidth: true
            elide: Text.ElideRight
        }

        GridLayout {
            columns: 2
            columnSpacing: 20
            rowSpacing: 15
            Layout.fillWidth: true

            Text { text: serverManager.t("form.name"); color: Theme.textSecondary; font.family: Theme.fontFamily }
            TextField { 
                id: nameField; 
                Layout.fillWidth: true
                color: Theme.textMain
                padding: 10
                font.family: Theme.fontFamily
                background: Rectangle { color: Theme.bgMain; radius: 6; border.color: nameField.activeFocus ? Theme.primary : Theme.borderPanel }
            }

            Text { text: serverManager.t("form.host"); color: Theme.textSecondary; font.family: Theme.fontFamily }
            TextField { 
                id: hostField; 
                Layout.fillWidth: true 
                color: Theme.textMain
                padding: 10
                font.family: Theme.fontFamily
                background: Rectangle { color: Theme.bgMain; radius: 6; border.color: hostField.activeFocus ? Theme.primary : Theme.borderPanel }
            }

            Text { text: serverManager.t("form.username"); color: Theme.textSecondary; font.family: Theme.fontFamily }
            TextField { 
                id: userField; 
                Layout.fillWidth: true 
                color: Theme.textMain
                padding: 10
                font.family: Theme.fontFamily
                background: Rectangle { color: Theme.bgMain; radius: 6; border.color: userField.activeFocus ? Theme.primary : Theme.borderPanel }
            }


            Text { text: serverManager.t("form.ssh_key"); color: Theme.textSecondary; font.family: Theme.fontFamily }
            TextField { 
                id: keyField; 
                Layout.fillWidth: true 
                color: Theme.textMain
                padding: 10
                font.family: Theme.fontFamily
                placeholderText: "~/.ssh/id_rsa"
                background: Rectangle { color: Theme.bgMain; radius: 6; border.color: keyField.activeFocus ? Theme.primary : Theme.borderPanel }
            }
        }

        Item { Layout.fillHeight: true } // Spacer

        RowLayout {
            Layout.fillWidth: true
            spacing: 20

            Button {
                text: serverManager.t("button.delete")
                visible: root.currentId !== "" && !root.currentId.startsWith("ac-")
                padding: 15
                background: Rectangle {
                    radius: Theme.radius
                    color: parent.hovered ? Theme.danger : "transparent"
                    border.color: Theme.danger
                }
                contentItem: Text {
                    text: parent.text
                    color: parent.hovered ? Theme.textMain : Theme.danger
                    horizontalAlignment: Text.AlignHCenter
                    font.bold: true
                    font.family: Theme.fontFamily
                }
                onClicked: {
                    if (root.serverManager) {
                        root.serverManager.remove_profile(root.currentId)
                        root.currentId = ""
                    }
                }
            }

            Item { Layout.fillWidth: true } // Spacer

            Button {
                text: serverManager.t("button.save_server")
                visible: root.currentId !== ""
                padding: 15
                background: Rectangle {
                    radius: Theme.radius
                    color: parent.hovered ? Theme.primaryHover : Theme.primary
                }
                contentItem: Text {
                    text: parent.text
                    color: Theme.textMain
                    horizontalAlignment: Text.AlignHCenter
                    font.bold: true
                    font.family: Theme.fontFamily
                }
                onClicked: {
                    if (root.serverManager) {
                        var profile = {
                            id: root.currentId,
                            name: nameField.text,
                            host: hostField.text,
                            username: userField.text,
                            key_path: keyField.text !== "" ? keyField.text : "~/.ssh/id_rsa"
                        };
                        root.serverManager.save_profile(JSON.stringify(profile));
                    }
                }
            }
        }
    }
}
