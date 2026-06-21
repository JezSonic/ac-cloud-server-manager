import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    Layout.fillWidth: true
    Layout.preferredHeight: 60

    property string label: ""
    property var model: []
    property string currentValue: ""
    property string textRole: ""
    property string valueRole: ""

    signal valueChanged(string newValue)

    ColumnLayout {
        anchors.fill: parent
        spacing: 5

        Text {
            text: root.label
            color: Theme.textSecondary
            font.pixelSize: 12
            font.bold: true
            font.family: Theme.fontFamily
        }

        ComboBox {
            id: combo
            Layout.fillWidth: true
            Layout.preferredHeight: 35
            
            model: root.model
            textRole: root.textRole
            valueRole: root.valueRole

            font.pixelSize: 14
            font.family: Theme.fontFamily

            background: Rectangle {
                color: Theme.bgHover
                radius: Theme.radius
                border.color: combo.activeFocus ? Theme.accent : Theme.borderPanel
                border.width: 1
            }

            onCurrentIndexChanged: {
                if (currentIndex >= 0 && currentIndex < count) {
                    var val = "";
                    if (valueRole !== "") {
                        val = combo.model.get ? combo.model.get(currentIndex)[valueRole] : combo.model[currentIndex][valueRole];
                    } else if (textRole !== "") {
                        val = combo.model.get ? combo.model.get(currentIndex)[textRole] : combo.model[currentIndex][textRole];
                    } else {
                        val = combo.model[currentIndex];
                    }
                    if (val !== undefined && val !== root.currentValue) {
                        root.valueChanged(val)
                    }
                }
            }

            Component.onCompleted: syncValue()
            onModelChanged: syncValue()
            
            Connections {
                target: root
                function onCurrentValueChanged() {
                    syncValue()
                }
            }

            function syncValue() {
                if (!root.model) return;
                var found = false;
                for (var i = 0; i < count; i++) {
                    var val = "";
                    if (valueRole !== "") {
                        val = combo.model.get ? combo.model.get(i)[valueRole] : combo.model[i][valueRole];
                    } else if (textRole !== "") {
                        val = combo.model.get ? combo.model.get(i)[textRole] : combo.model[i][textRole];
                    } else {
                        val = combo.model[i];
                    }
                    
                    if (val === root.currentValue) {
                        if (currentIndex !== i) {
                            currentIndex = i;
                        }
                        found = true;
                        break;
                    }
                }
                
                if (!found && root.currentValue !== "") {
                    currentIndex = -1;
                }
            }
        }
    }
}
