import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"
import "../views"

Item {
    id: managementPanel

    property QtObject serverManager
    property string currentId: ""

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Sidebar {
            id: sidebar
            Layout.fillHeight: true
            Layout.preferredWidth: 250
            serverManager: managementPanel.serverManager
            
            onSectionSelected: (section) => {
                sidebar.currentSection = section;
                stackLayout.currentIndex = ["dashboard", "vehicles", "tracks", "config"].indexOf(section);
            }
            
            onLogoutRequested: {
                serverManager.disconnect_from_server()
                managementPanel.currentId = ""
                // Also reset stackLayout current index to dashboard
                sidebar.currentSection = "dashboard"
                stackLayout.currentIndex = 0
            }
        }

        StackLayout {
            id: stackLayout
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: 0

            DashboardView {
                serverManager: managementPanel.serverManager
                currentId: managementPanel.currentId
            }

            VehiclesView {
                serverManager: managementPanel.serverManager
                currentId: managementPanel.currentId
            }

            TracksView {
                serverManager: managementPanel.serverManager
                currentId: managementPanel.currentId
            }

            ConfigView {
                serverManager: managementPanel.serverManager
                currentId: managementPanel.currentId
            }
        }
    }
}
