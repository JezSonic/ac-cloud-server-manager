import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Popup {
    id: root
    width: 600
    height: 500
    modal: true
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: Overlay.overlay
    
    property string title: "Progress"
    property string text: "Initializing..."
    property int progressValue: 0
    property bool indeterminate: true
    property bool closeVisible: false
    property color textColor: Theme.textMuted
    
    // Expose log area for appending
    property alias logArea: logOutput
    
    background: Rectangle {
        color: Theme.bgPanel
        radius: Theme.radius
        border.color: Theme.borderPanel
        border.width: 1
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.margins
        spacing: 15

        Text {
            text: root.title
            color: Theme.textMain
            font.pixelSize: 18
            font.bold: true
            font.family: Theme.fontFamily
        }

        Text {
            id: progressText
            text: root.text
            color: root.textColor
            font.pixelSize: 14
            font.family: Theme.fontFamily
            Layout.fillWidth: true
            wrapMode: Text.Wrap
        }

        ProgressBar {
            id: progressBar
            Layout.fillWidth: true
            from: 0
            to: 100
            value: root.progressValue
            indeterminate: root.indeterminate
        }

        TerminalOutput {
            id: logOutput
            Layout.fillWidth: true
            Layout.fillHeight: true
        }

        PrimaryButton {
            id: progressCloseButton
            text: "Close"
            visible: root.closeVisible
            Layout.alignment: Qt.AlignRight
            onClicked: root.close()
        }
    }
}
