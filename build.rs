use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("com.acmanager").qml_files([
        "qml/main.qml",
        "qml/components/Theme.qml",
        "qml/components/PrimaryButton.qml",
        "qml/components/SuccessButton.qml",
        "qml/components/DangerButton.qml",
        "qml/components/SecondaryButton.qml",
        "qml/components/Card.qml",
        "qml/components/FormField.qml",
        "qml/components/DropdownField.qml",
        "qml/components/ProfileList.qml",
        "qml/components/ProfileEditor.qml",
        "qml/components/Sidebar.qml",
        "qml/components/SidebarButton.qml",
        "qml/components/TerminalOutput.qml",
        "qml/dialogs/DependencyDialog.qml",
        "qml/dialogs/SteamLoginDialog.qml",
        "qml/dialogs/SteamTwoFactorDialog.qml",
        "qml/dialogs/InstallLogDialog.qml",
        "qml/dialogs/ProgressDialog.qml",
        "qml/dialogs/ConfirmDeleteDialog.qml",
        "qml/views/ManagementPanel.qml",
        "qml/views/DashboardView.qml",
        "qml/components/CircularGauge.qml",
        "qml/views/VehiclesView.qml",
        "qml/views/TracksView.qml",
        "qml/views/ConfigView.qml",
    ]))
    .file("src/gui/mod.rs")
    .qrc("qml/components/resources.qrc")
    .build();
}
