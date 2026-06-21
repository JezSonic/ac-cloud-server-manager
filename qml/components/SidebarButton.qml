import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    Layout.fillWidth: true
    height: 50
    color: isActive ? Theme.primary : (hoverHandler.hovered ? Theme.bgHover : "transparent")
    
    property string text: ""
    property string iconSource: ""
    property bool isActive: false
    signal clicked()

    HoverHandler {
        id: hoverHandler
    }

    TapHandler {
        onTapped: root.clicked()
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 20
        spacing: 15

        Text {
            text: root.text
            color: root.isActive ? Theme.textMain : Theme.textSecondary
            font.family: Theme.fontFamily
            font.pixelSize: 15
            font.bold: root.isActive
            Layout.fillWidth: true
        }
    }

    // Left active indicator
    Rectangle {
        width: 4
        height: parent.height
        anchors.left: parent.left
        color: Theme.primaryHover
        visible: root.isActive
    }
    
    Behavior on color {
        ColorAnimation { duration: 150 }
    }
}
