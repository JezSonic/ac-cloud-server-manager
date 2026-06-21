import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root
    title: serverManager ? serverManager.t("dialog.steam_2fa_title") : "Steam Guard (2FA)"
    modal: true
    anchors.centerIn: parent
    standardButtons: Dialog.Ok | Dialog.Cancel

    property QtObject serverManager

    background: Rectangle {
        color: "#1e293b"
        radius: 8
        border.color: "#334155"
    }

    contentItem: ColumnLayout {
        spacing: 15
        
        Text {
            text: serverManager ? serverManager.t("dialog.steam_2fa_desc") : ""
            color: "#94a3b8"
            font.pixelSize: 14
            Layout.fillWidth: true
        }

        TextField {
            id: codeField
            placeholderText: serverManager ? serverManager.t("dialog.steam_2fa_code") : ""
            Layout.fillWidth: true
            color: "white"
            background: Rectangle { color: "#0f111a"; radius: 4 }
        }
    }

    onAccepted: {
        serverManager.submit_steam_2fa(codeField.text);
        codeField.text = "";
    }
}
