import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: root
    property string label: ""
    property alias text: textField.text
    property alias placeholderText: textField.placeholderText
    signal textEdited(string text)
    spacing: 5

    Text {
        text: label
        color: Theme.textSecondary
        font.family: Theme.fontFamily
        font.pixelSize: 14
    }

    TextField {
        id: textField
        onTextEdited: root.textEdited(text)
        Layout.fillWidth: true
        color: Theme.textMain
        font.family: Theme.fontFamily
        background: Rectangle {
            color: Theme.bgHover
            radius: Theme.radius
            border.color: textField.activeFocus ? Theme.primary : "transparent"
            border.width: 1
        }
        padding: 10
    }
}
