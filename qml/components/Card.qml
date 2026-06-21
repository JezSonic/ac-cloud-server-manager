import QtQuick

Rectangle {
    id: root
    color: Theme.bgPanel
    radius: Theme.radius
    border.color: Theme.borderPanel
    border.width: 0 // Optional, mostly zero
    
    // Default padding logic can be handled by anchors in children,
    // but having a unified look is nice
}
