import QtQuick
import QtQuick.Controls

Rectangle {
    id: root
    color: "#1e1e1e" // Dark gray background
    radius: 6
    border.color: "#2e324d"
    
    property alias text: textArea.text
    property alias cursorPosition: textArea.cursorPosition

    function appendLog(chunk) {
        if (!chunk || chunk === "") return;
        
        // Strip ANSI escape codes safely without breaking QML parser
        let esc = String.fromCharCode(27);
        let ansiRegex = new RegExp(esc + "\\[[0-9;]*[a-zA-Z]", "g");
        let cleanLog = chunk.replace(ansiRegex, "");
        
        let escaped = cleanLog.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
        // Manually preserve spaces and newlines without <pre>
        escaped = escaped.replace(/ /g, "&nbsp;").replace(/\n/g, "<br>");

        let styled = escaped.replace(/(error:)/ig, "<font color='#ef4444'><b>$1</b></font>")
            .replace(/(err:)/ig, "<font color='#ef4444'><b>$1</b></font>")
            .replace(/(E:)/ig, "<font color='#ef4444'><b>$1</b></font>")
            .replace(/(warning:)/ig, "<font color='#eab308'><b>$1</b></font>")
            .replace(/(W:)/ig, "<font color='#eab308'><b>$1</b></font>")
            .replace(/(warn:)/ig, "<font color='#eab308'><b>$1</b></font>")
            .replace(/(\[SUCCESS\])/ig, "<font color='#22c55e'><b>$1</b></font>")
            .replace(/(\[ERROR\])/ig, "<font color='#ef4444'><b>$1</b></font>");

        textArea.append("<font color='#ffffff' face='monospace'>" + styled + "</font>");
        textArea.cursorPosition = textArea.length;
    }

    ScrollView {
        anchors.fill: parent
        anchors.margins: 10
        
        TextArea {
            id: textArea
            color: "#ffffff" // White text
            textFormat: TextEdit.RichText
            font.family: "Monospace"
            font.pixelSize: 13
            readOnly: true
            selectByMouse: true
            background: null
            wrapMode: TextEdit.Wrap
        }
    }
}
