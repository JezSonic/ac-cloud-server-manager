import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: sidebar
    width: 250
    color: Theme.bgPanel
    
    property string currentSection: "dashboard"
    property QtObject serverManager
    signal sectionSelected(string section)
    signal logoutRequested()

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            height: 80
            color: "transparent"
            
            Text {
                anchors.centerIn: parent
                text: serverManager.t("sidebar.menu") || "Menu"
                font.family: Theme.fontFamily
                font.pixelSize: 20
                font.bold: true
                color: Theme.textMain
            }
        }

        // Dashboard
        SidebarButton {
            text: serverManager.t("sidebar.dashboard") || "Dashboard"
            iconSource: ""
            isActive: sidebar.currentSection === "dashboard"
            onClicked: sidebar.sectionSelected("dashboard")
        }

        // Vehicles
        SidebarButton {
            text: serverManager.t("sidebar.vehicles") || "Vehicles"
            iconSource: ""
            isActive: sidebar.currentSection === "vehicles"
            onClicked: sidebar.sectionSelected("vehicles")
        }

        // Tracks
        SidebarButton {
            text: serverManager.t("sidebar.tracks") || "Tracks"
            iconSource: ""
            isActive: sidebar.currentSection === "tracks"
            onClicked: sidebar.sectionSelected("tracks")
        }

        // Config
        SidebarButton {
            text: serverManager.t("sidebar.config") || "Configuration"
            iconSource: ""
            isActive: sidebar.currentSection === "config"
            onClicked: sidebar.sectionSelected("config")
        }

        Item {
            Layout.fillHeight: true // Spacer
        }

        // Bottom Section
        ColumnLayout {
            Layout.fillWidth: true
            Layout.margins: 15
            spacing: 10

            SecondaryButton {
                Layout.fillWidth: true
                text: "Logout"
                onClicked: sidebar.logoutRequested()
            }

            Text {
                Layout.alignment: Qt.AlignHCenter
                text: "v0.1.0"
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: 12
            }
        }
    }
}
