import QtQuick

Item {
    id: root
    property real value: 0
    property real maxValue: 100
    property string label: ""
    property string subLabel: ""
    property color color: "#3b82f6"
    property alias textColor: valueText.color

    width: 200
    height: 200

    Canvas {
        id: canvas
        anchors.fill: parent
        antialiasing: true

        onPaint: {
            var ctx = getContext("2d");
            ctx.clearRect(0, 0, width, height);

            var centerX = width / 2;
            var centerY = height / 2;
            var radius = Math.min(width, height) / 2 - 10;

            // Background arc
            ctx.beginPath();
            ctx.arc(centerX, centerY, radius, 0, 2 * Math.PI);
            ctx.strokeStyle = "#334155";
            ctx.lineWidth = 12;
            ctx.stroke();

            // Value arc
            var startAngle = -Math.PI / 2;
            var endAngle = startAngle + (2 * Math.PI * (value / maxValue));
            
            ctx.beginPath();
            ctx.arc(centerX, centerY, radius, startAngle, endAngle);
            ctx.strokeStyle = color;
            ctx.lineWidth = 12;
            ctx.lineCap = "round";
            ctx.stroke();
        }

    }

    onValueChanged: canvas.requestPaint()

    Column {
        anchors.centerIn: parent
        spacing: 4
        
        Text {
            id: valueText
            text: label
            font.pixelSize: 18
            font.bold: true
            color: "#ffffff"
            anchors.horizontalCenter: parent.horizontalCenter
        }
        
        Text {
            text: subLabel
            font.pixelSize: 12
            color: "#94a3b8"
            anchors.horizontalCenter: parent.horizontalCenter
            visible: subLabel !== ""
        }
    }
}
