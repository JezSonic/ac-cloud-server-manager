import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root
    title: serverManager ? serverManager.t("dialog.steam_login_title") : "Steam Login"
    modal: true
    anchors.centerIn: parent
    standardButtons: Dialog.Ok | Dialog.Cancel

    property QtObject serverManager
    property string currentId

    background: Rectangle {
        color: "#1e293b"
        radius: 8
        border.color: "#334155"
    }

    contentItem: ColumnLayout {
        spacing: 15
        
        Text {
            text: serverManager ? serverManager.t("dialog.steam_login_desc") : ""
            color: "#94a3b8"
            font.pixelSize: 14
            Layout.fillWidth: true
        }

        TextField {
            id: usernameField
            placeholderText: serverManager ? serverManager.t("dialog.steam_username") : ""
            Layout.fillWidth: true
            color: "white"
            background: Rectangle { color: "#0f111a"; radius: 4 }
        }

        TextField {
            id: passwordField
            placeholderText: serverManager ? serverManager.t("dialog.steam_password") : ""
            echoMode: TextInput.Password
            Layout.fillWidth: true
            color: "white"
            background: Rectangle { color: "#0f111a"; radius: 4 }
        }
    }

    onAccepted: {
        console.log("Accepted");
        serverManager.install_ac_server(currentId, usernameField.text, passwordField.text);
    }
}
