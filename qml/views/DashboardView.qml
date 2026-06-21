import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtCharts
import "../components"
import "../dialogs" as Dialogs

Item {
    id: dashboardView
    property QtObject serverManager
    property string currentId: ""
    property var currentStats: null
    property int historyLimit: 60
    property int tickCount: 0

    Timer {
        interval: 1000
        running: dashboardView.visible
        repeat: true
        onTriggered: {
            serverManager.poll_2fa_request();
            var jsonStr = serverManager.get_latest_stats();
            if (jsonStr && jsonStr !== "{}" && jsonStr !== "") {
                currentStats = JSON.parse(jsonStr);
                updateCharts();
            }
        }
    }

    function formatBytes(bytes) {
        if (!bytes && bytes !== 0) return '0 B';
        if (bytes === 0) return '0 B';
        var k = 1024;
        var sizes = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
        var i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }

    function updateCharts() {
        if (!currentStats) return;
        tickCount++;

        // Update charts history
        if (currentStats.cpu_cores) {
            for (var i = 0; i < currentStats.cpu_cores.length; i++) {
                var core = currentStats.cpu_cores[i];
                var series = cpuDetailedChart.series(serverManager.t("chart.core_prefix") + " " + (i+1));
                if (!series) {
                    series = cpuDetailedChart.createSeries(ChartView.SeriesTypeLine, serverManager.t("chart.core_prefix") + " " + (i+1), cpuDetailedAxisX, cpuDetailedAxisY);
                    series.width = 1;
                }
                series.append(tickCount, core.usage_percent);
                if (series.count > historyLimit) series.remove(0);
            }
            cpuDetailedAxisX.min = Math.max(0, tickCount - historyLimit);
            cpuDetailedAxisX.max = Math.max(historyLimit, tickCount);
        }

        netDetailedRxSeries.append(tickCount, currentStats.net_rx_kbps || 0);
        netDetailedTxSeries.append(tickCount, currentStats.net_tx_kbps || 0);
        if (netDetailedRxSeries.count > historyLimit) { netDetailedRxSeries.remove(0); netDetailedTxSeries.remove(0); }
        netDetailedAxisX.min = Math.max(0, tickCount - historyLimit);
        netDetailedAxisX.max = Math.max(historyLimit, tickCount);
        
        memHistorySeries.append(tickCount, currentStats.ram_usage_percent || 0);
        if (memHistorySeries.count > historyLimit) memHistorySeries.remove(0);
        memHistoryAxisX.min = Math.max(0, tickCount - historyLimit);
        memHistoryAxisX.max = Math.max(historyLimit, tickCount);

        swapHistorySeries.append(tickCount, currentStats.swap_usage_percent || 0);
        if (swapHistorySeries.count > historyLimit) swapHistorySeries.remove(0);
        swapHistoryAxisX.min = Math.max(0, tickCount - historyLimit);
        swapHistoryAxisX.max = Math.max(historyLimit, tickCount);
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.margins
        spacing: Theme.spacing

        // Fixed Header
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 60
            color: "transparent"

            RowLayout {
                anchors.fill: parent

                Text {
                    text: serverManager.t("title.dashboard")
                    font.family: Theme.fontFamily
                    font.pixelSize: 28
                    font.bold: true
                    color: Theme.textMain
                    Layout.fillWidth: true
                }

                RowLayout {
                    spacing: 15
                    DangerButton {
                        text: serverManager.t("button.reboot")
                        onClicked: serverManager.reboot_host(dashboardView.currentId)
                    }

                    DangerButton {
                        text: serverManager.t("button.shutdown")
                        onClicked: serverManager.shutdown_host(dashboardView.currentId)
                    }
                }
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            Text {
                text: "Loading data from server..."
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: 18
                anchors.centerIn: parent
                visible: !currentStats
            }

            ScrollView {
                anchors.fill: parent
                contentWidth: availableWidth
                clip: true
                visible: currentStats !== null
                
                ColumnLayout {
                    width: parent.width
                    spacing: Theme.spacing

                    // AC Server Controls
                    Card {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 70

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 15
                            spacing: 15

                            Text {
                                text: "AC Server"
                                color: Theme.textMain
                                font.bold: true
                                font.pixelSize: 18
                                font.family: Theme.fontFamily
                            }
                            
                            Item { Layout.fillWidth: true }
                            
                            Text {
                                visible: currentStats && currentStats.ac_server_status
                                text: {
                                    if (!currentStats) return "";
                                    var str = "Status: " + currentStats.ac_server_status;
                                    if (currentStats.ac_tcp_port && currentStats.ac_server_status !== "UNINSTALLED") {
                                        str += " | Port: " + currentStats.ac_tcp_port;
                                    }
                                    return str;
                                }
                                color: currentStats && currentStats.ac_server_status === "ON" ? Theme.success : Theme.textSecondary
                                font.family: Theme.fontFamily
                            }
                            
                            SuccessButton {
                                text: serverManager.t("button.ac_start")
                                onClicked: serverManager.start_ac_server(dashboardView.currentId)
                            }

                            DangerButton {
                                text: serverManager.t("button.ac_stop")
                                onClicked: serverManager.stop_ac_server(dashboardView.currentId)
                            }

                            PrimaryButton {
                                text: serverManager.t("button.ac_install")
                                onClicked: steamLoginDialog.open()
                            }

                            SecondaryButton {
                                text: serverManager.t("button.ac_uninstall")
                                onClicked: serverManager.uninstall_ac_server(dashboardView.currentId)
                            }
                        }
                    }

                    // Top Row: Gauges
                    GridLayout {
                        Layout.fillWidth: true
                        columns: dashboardView.width < 1400 ? 1 : 2
                        columnSpacing: Theme.spacing
                        rowSpacing: Theme.spacing
                        
                        // Procesor & GPU
                        Card {
                            Layout.fillWidth: true
                            Layout.preferredHeight: dashboardView.width < 1200 ? 500 : 250
                            
                            GridLayout {
                                anchors.fill: parent
                                anchors.margins: 15
                                columns: dashboardView.width < 1200 ? 1 : 2
                                columnSpacing: 15
                                rowSpacing: 15
                                
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    Text { text: serverManager.t("chart.proc"); color: Theme.textMain; font.bold: true; Layout.alignment: Qt.AlignHCenter; font.family: Theme.fontFamily }
                                    
                                    RowLayout {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        
                                        CircularGauge {
                                            Layout.preferredWidth: 150; Layout.preferredHeight: 150
                                            value: currentStats ? currentStats.cpu_usage_total : 0
                                            label: (currentStats ? currentStats.cpu_usage_total.toFixed(1) : "0.0") + "%"
                                            color: Theme.primary
                                            Layout.alignment: Qt.AlignVCenter
                                        }

                                        ColumnLayout {
                                            Layout.fillWidth: true; Layout.fillHeight: true
                                            Text { text: serverManager.t("chart.cpu_temp"); color: Theme.textMain; font.bold: true; Layout.alignment: Qt.AlignHCenter; font.family: Theme.fontFamily }
                                            ChartView {
                                                Layout.fillWidth: true; Layout.fillHeight: true
                                                Layout.minimumHeight: 100
                                                backgroundColor: "transparent"; legend.visible: false; antialiasing: true
                                                ValueAxis { id: cpuTempY; min: 0; max: 100; labelsColor: Theme.textSecondary; tickCount: 5 }
                                                BarSeries {
                                                    axisY: cpuTempY
                                                    BarSet { 
                                                        values: [currentStats && currentStats.cpu_cores && currentStats.cpu_cores.length > 0 ? currentStats.cpu_cores[0].temperature : 0]
                                                        color: Theme.primary
                                                    }
                                                }
                                                Text { 
                                                    anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter
                                                    text: (currentStats && currentStats.cpu_cores && currentStats.cpu_cores.length > 0 ? currentStats.cpu_cores[0].temperature.toFixed(1) : "0.0") + "°C"
                                                    color: Theme.textMain; font.pixelSize: 12; font.bold: true; font.family: Theme.fontFamily
                                                }
                                            }
                                        }
                                    }
                                }

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    visible: currentStats && currentStats.has_gpu
                                    Text { text: serverManager.t("chart.gpu"); color: Theme.textMain; font.bold: true; Layout.alignment: Qt.AlignHCenter; font.family: Theme.fontFamily }
                                    
                                    RowLayout {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        
                                        CircularGauge {
                                            Layout.preferredWidth: 150; Layout.preferredHeight: 150
                                            value: currentStats ? currentStats.gpu_usage_percent : 0
                                            label: (currentStats ? currentStats.gpu_usage_percent.toFixed(0) : "0") + "%"
                                            color: Theme.primary
                                            Layout.alignment: Qt.AlignVCenter
                                        }

                                        ColumnLayout {
                                            Layout.fillWidth: true; Layout.fillHeight: true
                                            Text { text: serverManager.t("chart.gpu_temp"); color: Theme.textMain; font.bold: true; Layout.alignment: Qt.AlignHCenter; font.family: Theme.fontFamily }
                                            ChartView {
                                                Layout.fillWidth: true; Layout.fillHeight: true
                                                Layout.minimumHeight: 100
                                                backgroundColor: "transparent"; legend.visible: false; antialiasing: true
                                                ValueAxis { id: gpuTempY; min: 0; max: 100; labelsColor: Theme.textSecondary; tickCount: 5 }
                                                BarSeries {
                                                    axisY: gpuTempY
                                                    BarSet { 
                                                        values: [currentStats ? currentStats.gpu_temp : 0]
                                                        color: Theme.primaryHover
                                                    }
                                                }
                                                Text { 
                                                    anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter
                                                    text: (currentStats ? currentStats.gpu_temp.toFixed(1) : "0.0") + "°C"
                                                    color: Theme.textMain; font.pixelSize: 12; font.bold: true; font.family: Theme.fontFamily
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                Text {
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                    visible: currentStats && !currentStats.has_gpu
                                    text: "No GPU detected on the host"
                                    color: Theme.textSecondary
                                    font.family: Theme.fontFamily
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }
                        }

                        // Pamięć
                        Card {
                            Layout.fillWidth: true
                            Layout.preferredHeight: dashboardView.width < 1200 ? 500 : 250
                            
                            GridLayout {
                                anchors.fill: parent
                                anchors.margins: 15
                                columns: dashboardView.width < 1200 ? 1 : 2
                                columnSpacing: 15
                                rowSpacing: 15
                                
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    Text { text: serverManager.t("chart.mem"); color: Theme.textMain; font.bold: true; Layout.alignment: Qt.AlignHCenter; font.family: Theme.fontFamily }
                                    
                                    RowLayout {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        
                                        CircularGauge {
                                            Layout.preferredWidth: 120; Layout.preferredHeight: 120
                                            value: currentStats ? currentStats.ram_usage_percent : 0
                                            label: (currentStats ? currentStats.ram_usage_percent.toFixed(1) : "0.0") + "%"
                                            subLabel: (currentStats ? (currentStats.ram_used_mb / 1024).toFixed(1) : "0.0") + "G / " + 
                                                      (currentStats ? (currentStats.ram_total_mb / 1024).toFixed(1) : "0.0") + "G"
                                            color: Theme.primary
                                            Layout.alignment: Qt.AlignVCenter
                                        }
                                        
                                        ChartView {
                                            Layout.fillWidth: true; Layout.fillHeight: true
                                            Layout.minimumHeight: 150
                                            backgroundColor: "transparent"; legend.visible: false; antialiasing: true
                                            ValueAxis { id: memHistoryAxisX; min: 0; max: historyLimit; visible: false }
                                            ValueAxis { id: memHistoryAxisY; min: 0; max: 100; labelsColor: Theme.textSecondary; titleText: "%" }
                                            LineSeries { id: memHistorySeries; axisX: memHistoryAxisX; axisY: memHistoryAxisY; color: Theme.primary; width: 2 }
                                        }
                                    }
                                }

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    Text { text: serverManager.t("chart.swap"); color: Theme.textMain; font.bold: true; Layout.alignment: Qt.AlignHCenter; font.family: Theme.fontFamily }
                                    
                                    RowLayout {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        
                                        CircularGauge {
                                            Layout.preferredWidth: 120; Layout.preferredHeight: 120
                                            value: currentStats ? currentStats.swap_usage_percent : 0
                                            label: (currentStats ? currentStats.swap_usage_percent.toFixed(1) : "0.0") + "%"
                                            subLabel: (currentStats ? (currentStats.swap_used_mb / 1024).toFixed(1) : "0.0") + "G / " + 
                                                      (currentStats ? (currentStats.swap_total_mb / 1024).toFixed(1) : "0.0") + "G"
                                            color: Theme.primary
                                            Layout.alignment: Qt.AlignVCenter
                                        }
                                        
                                        ChartView {
                                            Layout.fillWidth: true; Layout.fillHeight: true
                                            Layout.minimumHeight: 150
                                            backgroundColor: "transparent"; legend.visible: false; antialiasing: true
                                            ValueAxis { id: swapHistoryAxisX; min: 0; max: historyLimit; visible: false }
                                            ValueAxis { id: swapHistoryAxisY; min: 0; max: 100; labelsColor: Theme.textSecondary; titleText: "%" }
                                            LineSeries { id: swapHistorySeries; axisX: swapHistoryAxisX; axisY: swapHistoryAxisY; color: Theme.primary; width: 2 }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Middle Row: Disks & Network Info
                    RowLayout {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 220
                        spacing: Theme.spacing
                        
                        Card {
                            Layout.fillWidth: true; Layout.fillHeight: true
                            ScrollView {
                                anchors.fill: parent; anchors.margins: 20
                                contentWidth: availableWidth
                                clip: true
                                ColumnLayout {
                                    width: parent.width
                                    Text { text: serverManager.t("chart.disks"); color: Theme.textMain; font.bold: true; font.pixelSize: 16; font.family: Theme.fontFamily }
                                    Repeater {
                                        model: currentStats ? currentStats.disks : []
                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            RowLayout {
                                                Text { text: modelData.mount_point; color: Theme.textMain; font.pixelSize: 12; font.family: Theme.fontFamily }
                                                Item { Layout.fillWidth: true }
                                                Text { text: formatBytes(modelData.used_bytes) + " / " + formatBytes(modelData.total_bytes); color: Theme.textMain; font.pixelSize: 12; font.family: Theme.fontFamily }
                                            }
                                            ProgressBar {
                                                id: diskProgressBar
                                                Layout.fillWidth: true
                                                value: modelData.total_bytes > 0 ? (modelData.used_bytes / modelData.total_bytes) : 0
                                                background: Rectangle { color: Theme.bgHover; radius: 4; height: 10 }
                                                contentItem: Item {
                                                    Rectangle {
                                                        width: diskProgressBar.visualPosition * diskProgressBar.width; height: 10
                                                        radius: 4; color: index % 2 === 0 ? Theme.primary : "#ec4899"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Card {
                            Layout.fillWidth: true; Layout.fillHeight: true
                            ColumnLayout {
                                anchors.fill: parent; anchors.margins: 20
                                Text { text: serverManager.t("chart.nets"); color: Theme.textMain; font.bold: true; font.pixelSize: 16; font.family: Theme.fontFamily }
                                Text { text: serverManager.t("chart.connection_name").replace("%{name}", currentStats ? currentStats.net_interface_name : ""); color: Theme.textMain; font.pixelSize: 14; font.bold: true; font.family: Theme.fontFamily }
                                RowLayout {
                                    spacing: 30
                                    ColumnLayout {
                                        RowLayout {
                                            Rectangle { width: 12; height: 12; color: Theme.primary; radius: 2 }
                                            Text { text: serverManager.t("chart.dl"); color: Theme.textMain; font.pixelSize: 12; font.family: Theme.fontFamily }
                                            Text { text: (currentStats ? currentStats.net_rx_kbps.toFixed(1) : "0.0") + " KiB/s"; color: Theme.textMain; font.pixelSize: 12; font.bold: true; font.family: Theme.fontFamily }
                                        }
                                        RowLayout {
                                            Rectangle { width: 12; height: 12; color: "#ec4899"; radius: 2 }
                                            Text { text: serverManager.t("chart.ul"); color: Theme.textMain; font.pixelSize: 12; font.family: Theme.fontFamily }
                                            Text { text: (currentStats ? currentStats.net_tx_kbps.toFixed(1) : "0.0") + " KiB/s"; color: Theme.textMain; font.pixelSize: 12; font.bold: true; font.family: Theme.fontFamily }
                                        }
                                    }
                                    ColumnLayout {
                                        RowLayout {
                                            Rectangle { width: 12; height: 12; color: "#f59e0b"; radius: 2 }
                                            Text { text: "IPv4"; color: Theme.textMain; font.pixelSize: 12; font.family: Theme.fontFamily }
                                            Text { text: currentStats ? currentStats.ipv4 : ""; color: Theme.textMain; font.pixelSize: 12; font.bold: true; font.family: Theme.fontFamily }
                                        }
                                        RowLayout {
                                            Rectangle { width: 12; height: 12; color: Theme.success; radius: 2 }
                                            Text { text: "IPv6"; color: Theme.textMain; font.pixelSize: 12; font.family: Theme.fontFamily }
                                            Text { text: currentStats ? currentStats.ipv6 : ""; color: Theme.textMain; font.pixelSize: 12; font.bold: true; font.family: Theme.fontFamily }
                                        }
                                    }
                                }
                                Item { Layout.fillHeight: true }
                            }
                        }
                    }

                    // Two charts in one row (CPU and Net)
                    RowLayout {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 350
                        spacing: Theme.spacing
                        
                        Card {
                            Layout.fillWidth: true; Layout.fillHeight: true
                            ColumnLayout {
                                anchors.fill: parent; anchors.margins: 15
                                Text { text: serverManager.t("chart.proc"); color: Theme.textMain; font.bold: true; font.pixelSize: 16; font.family: Theme.fontFamily }
                                ChartView {
                                    id: cpuDetailedChart
                                    Layout.fillWidth: true; Layout.fillHeight: true
                                    backgroundColor: "transparent"; legend.labelColor: Theme.textMain; antialiasing: true
                                    legend.alignment: Qt.AlignBottom
                                    ValueAxis { id: cpuDetailedAxisX; min: 0; max: historyLimit; visible: false }
                                    ValueAxis { id: cpuDetailedAxisY; min: 0; max: 100; labelsColor: Theme.textSecondary; titleText: "%" }
                                }
                            }
                        }

                        Card {
                            Layout.fillWidth: true; Layout.fillHeight: true
                            ColumnLayout {
                                anchors.fill: parent; anchors.margins: 15
                                Text { text: serverManager.t("chart.net_single"); color: Theme.textMain; font.bold: true; font.pixelSize: 16; font.family: Theme.fontFamily }
                                ChartView {
                                    id: netDetailedChart
                                    Layout.fillWidth: true; Layout.fillHeight: true
                                    backgroundColor: "transparent"; legend.labelColor: Theme.textMain; antialiasing: true
                                    legend.alignment: Qt.AlignBottom
                                    ValueAxis { id: netDetailedAxisX; min: 0; max: historyLimit; visible: false }
                                    ValueAxis { id: netDetailedAxisY; min: 0; max: 1000; labelsColor: Theme.textSecondary; titleText: "KiB/s" }
                                    LineSeries { id: netDetailedRxSeries; name: serverManager.t("chart.dl"); axisX: netDetailedAxisX; axisY: netDetailedAxisY; color: Theme.primary; width: 2 }
                                    LineSeries { id: netDetailedTxSeries; name: serverManager.t("chart.ul"); axisX: netDetailedAxisX; axisY: netDetailedAxisY; color: "#ec4899"; width: 2 }
                                }
                            }
                        }
                    }

                    // Bottom Row: Assetto Corsa Players
                    RowLayout {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 300
                        
                        Card {
                            Layout.fillWidth: true; Layout.fillHeight: true
                            ColumnLayout {
                                anchors.fill: parent; anchors.margins: 20
                                Text { text: serverManager.t("chart.players"); color: Theme.textMain; font.bold: true; font.pixelSize: 16; font.family: Theme.fontFamily }
                                
                                ListView {
                                    id: playersList
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                    clip: true
                                    spacing: 8
                                    model: currentStats ? currentStats.ac_players : []
                                    delegate: Rectangle {
                                        width: playersList.width
                                        height: 70
                                        color: itemMouseArea.containsMouse ? Theme.bgHover : Theme.bgMain
                                        border.color: itemMouseArea.containsMouse ? Theme.primary : Theme.bgPanel
                                        border.width: 1
                                        radius: 10
                                        
                                        Behavior on color { ColorAnimation { duration: 150 } }
                                        Behavior on border.color { ColorAnimation { duration: 150 } }
                                        
                                        MouseArea {
                                            id: itemMouseArea
                                            anchors.fill: parent
                                            hoverEnabled: true
                                        }

                                        RowLayout {
                                            anchors.fill: parent
                                            anchors.margins: 12
                                            spacing: 20
                                            
                                            // Left Side: Driver Name & Team/Nation
                                            ColumnLayout {
                                                Layout.preferredWidth: 250
                                                spacing: 4
                                                
                                                RowLayout {
                                                    spacing: 8
                                                    Rectangle {
                                                        width: 8; height: 8; radius: 4
                                                        color: Theme.success
                                                    }
                                                    Text { text: modelData.name; color: Theme.textMain; font.bold: true; font.pixelSize: 15; font.family: Theme.fontFamily }
                                                    
                                                    Rectangle {
                                                        visible: modelData.nation !== ""
                                                        color: Theme.bgHover; radius: 4
                                                        Layout.preferredHeight: 18
                                                        width: nationText.width + 10
                                                        Text { id: nationText; text: modelData.nation; color: Theme.textMuted; font.pixelSize: 10; anchors.centerIn: parent; font.family: Theme.fontFamily }
                                                    }
                                                }
                                                
                                                Text { 
                                                    text: modelData.team !== "" ? modelData.team : "Independent"
                                                    color: Theme.textSecondary
                                                    font.pixelSize: 12
                                                    font.family: Theme.fontFamily
                                                }
                                            }

                                            // Middle: Car & Skin
                                            ColumnLayout {
                                                Layout.fillWidth: true
                                                spacing: 4
                                                
                                                Text { text: modelData.car; color: Theme.textMuted; font.bold: true; font.pixelSize: 14; font.family: Theme.fontFamily }
                                                Text { text: modelData.skin; color: Theme.textSecondary; font.pixelSize: 12; font.family: Theme.fontFamily }
                                            }

                                            // Right Side: Badges / GUID
                                            ColumnLayout {
                                                Layout.alignment: Qt.AlignRight
                                                spacing: 4
                                                
                                                Rectangle {
                                                    visible: modelData.is_entry_list
                                                    color: Theme.primary; radius: 4
                                                    Layout.preferredHeight: 20; Layout.alignment: Qt.AlignRight
                                                    width: entryText.width + 12
                                                    Text { id: entryText; text: "Entry List"; color: Theme.textMain; font.pixelSize: 10; font.bold: true; anchors.centerIn: parent; font.family: Theme.fontFamily }
                                                }
                                                
                                                Text { text: modelData.guid || ""; color: Theme.secondaryHover; font.pixelSize: 10; Layout.alignment: Qt.AlignRight; font.family: Theme.fontFamily }
                                            }
                                        }
                                    }
                                    Text {
                                        visible: playersList.count === 0 && currentStats && currentStats.ac_server_status === "ON"
                                        text: serverManager.t("chart.no_players")
                                        color: Theme.textSecondary
                                        font.family: Theme.fontFamily
                                        anchors.centerIn: parent
                                    }
                                    Text {
                                        visible: currentStats && currentStats.ac_server_status !== "ON"
                                        text: "Serwer wyłączony lub niezainstalowany."
                                        color: Theme.textSecondary
                                        font.family: Theme.fontFamily
                                        anchors.centerIn: parent
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Dialogs.SteamLoginDialog {
        id: steamLoginDialog
        serverManager: dashboardView.serverManager
        currentId: dashboardView.currentId
        onAccepted: dashboardView.openInstallLog()
    }

    Dialogs.SteamTwoFactorDialog {
        id: twoFactorDialog
        serverManager: dashboardView.serverManager
    }

    Dialogs.InstallLogDialog {
        id: installLogDialog
        serverManager: dashboardView.serverManager
    }

    function openInstallLog() {
        installLogDialog.open();
    }

    Connections {
        target: serverManager
        function onNeeds_steam_2faChanged() {
            if (serverManager.needs_steam_2fa) {
                twoFactorDialog.open()
            }
        }
    }
}
