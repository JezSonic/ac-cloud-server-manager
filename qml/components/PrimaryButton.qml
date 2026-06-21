import QtQuick
import QtQuick.Controls

Button {
    id: control
    padding: 10
    leftPadding: 20
    rightPadding: 20
    
    background: Rectangle {
        radius: Theme.radius
        color: control.down ? Theme.primaryHover : Theme.primary
        
        Behavior on color { ColorAnimation { duration: 150 } }
    }
    
    contentItem: Text {
        text: control.text
        color: Theme.textMain
        font.family: Theme.fontFamily
        font.bold: true
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
    }
}
