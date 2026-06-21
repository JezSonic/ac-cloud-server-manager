import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Popup {
    id: root
    width: 400
    height: 200
    modal: true
    anchors.centerIn: Overlay.overlay
    
    property string title: "Potwierdzenie usunięcia"
    property string message: ""
    
    signal confirmed()

    background: Rectangle {
        color: Theme.bgPanel
        radius: Theme.radius
        border.color: Theme.borderPanel
        border.width: 1
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 15

        Text {
            text: root.title
            color: Theme.textMain
            font.pixelSize: 18
            font.bold: true
            font.family: Theme.fontFamily
        }

        Text {
            text: root.message
            color: Theme.textMuted
            font.pixelSize: 14
            font.family: Theme.fontFamily
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
            
            SecondaryButton {
                text: "Anuluj"
                onClicked: root.close()
            }
            
            DangerButton {
                text: "Usuń"
                onClicked: {
                    root.confirmed()
                    root.close()
                }
            }
        }
    }
}
