import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
import com.acmanager 1.0

import "components"
import "dialogs"
import "views"

Window {
    id: mainWindow
    width: 1000
    height: 700
    visibility: Window.Maximized
    title: serverManager.t("app.title")
    color: Theme.bgMain
    
    property string connectedServerId: ""

    // Fonts
    FontLoader {
        id: interFont
        source: "https://fonts.gstatic.com/s/inter/v12/UcCO3FwrK3iLTeHuS_fvQtMwCp50KnMw2boKoduKmMEVuLyfMZhrib2Bg-4.ttf" 
    }

    readonly property bool isSmallScreen: width < 900

    ServerManager {
        id: serverManager
    }

    Component.onCompleted: {
        serverManager.init();
    }

    // Dynamic Background Gradient
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.bgMain }
            GradientStop { position: 1.0; color: "#1a1d2e" }
        }
    }

    // Decorative Blur / Glow Element
    Rectangle {
        width: 300
        height: 300
        radius: 150
        color: Theme.primary
        opacity: 0.15
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: -50
    }

    StackLayout {
        id: mainStackLayout
        anchors.fill: parent
        currentIndex: serverManager.is_connected ? 1 : 0

        // Index 0: Connection & Profile View
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Theme.margins
                spacing: 20

                // Header
                Text {
                    text: serverManager.t("app.header")
                    font.family: Theme.fontFamily
                    font.pixelSize: 32
                    font.bold: true
                    color: Theme.textMain
                    Layout.alignment: Qt.AlignLeft
                }
                
                Text {
                    text: serverManager.status
                    font.family: Theme.fontFamily
                    font.pixelSize: 14
                    color: Theme.textSecondary
                    Layout.alignment: Qt.AlignLeft
                    Layout.bottomMargin: 20
                }

                // Profiles
                RowLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: Theme.spacing
                    visible: !mainWindow.isSmallScreen

                    ProfileList {
                        id: profileList
                        Layout.fillHeight: true
                        Layout.fillWidth: true
                        Layout.maximumWidth: 400
                        serverManager: serverManager
                        
                        onEditRequested: (profile) => {
                            editorPane.loadProfile(profile);
                        }
                        
                        onConnectRequested: (id) => {
                            mainWindow.connectedServerId = id;
                        }
                    }

                    ProfileEditor {
                        id: editorPane
                        Layout.fillHeight: true
                        Layout.fillWidth: true
                        serverManager: serverManager
                    }
                }

                StackLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    currentIndex: profileListSmall.active ? 0 : 1
                    visible: mainWindow.isSmallScreen

                    ProfileList {
                        id: profileListSmall
                        property bool active: true
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        serverManager: serverManager
                        onEditRequested: (profile) => {
                            if (profile === null) {
                                editorPaneSmall.clearAndShow();
                            } else {
                                editorPaneSmall.loadProfile(profile);
                            }
                            active = false;
                        }
                        onConnectRequested: (id) => {
                            mainWindow.connectedServerId = id;
                        }
                    }

                    ColumnLayout {
                        spacing: 20
                        RowLayout {
                            Layout.fillWidth: true
                            PrimaryButton {
                                text: "< Back to List"
                                onClicked: profileListSmall.active = true
                            }
                            Item { Layout.fillWidth: true }
                            PrimaryButton {
                                text: serverManager.t("button.add_server")
                                onClicked: {
                                    editorPaneSmall.clearAndShow();
                                }
                            }
                        }
                        ProfileEditor {
                            id: editorPaneSmall
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            serverManager: serverManager
                        }
                    }
                }
            }
        }

        // Index 1: Full-Screen Management Panel View
        ManagementPanel {
            id: managementPanel
            Layout.fillWidth: true
            Layout.fillHeight: true
            serverManager: serverManager
            currentId: mainWindow.connectedServerId
        }
    }

    DependencyDialog {
        id: dependencyDialog
        serverManager: serverManager
        currentId: mainWindow.connectedServerId
    }

    Connections {
        target: serverManager
        function onMissing_dependencies_jsonChanged() {
            if (serverManager.missing_dependencies_json !== "[]" && serverManager.missing_dependencies_json !== "") {
                dependencyDialog.show()
            }
        }
    }
}
