pragma Singleton
import QtQuick

QtObject {
    // Backgrounds
    property color bgMain: "#0f111a"
    property color bgPanel: "#1e293b"
    property color bgHover: "#334155"
    
    // Borders
    property color borderPanel: "#334155"
    property color borderFocus: "#3b82f6"
    
    // Texts
    property color textMain: "#ffffff"
    property color textSecondary: "#94a3b8"
    property color textMuted: "#cbd5e1"
    
    // Actions
    property color primary: "#3b82f6"
    property color primaryHover: "#2563eb"
    property color success: "#22c55e"
    property color successHover: "#166534"
    property color danger: "#ef4444"
    property color dangerHover: "#b91c1c"
    property color secondary: "#64748b"
    property color secondaryHover: "#475569"
    
    // Common settings
    property int radius: 8
    property int spacing: 20
    property int margins: 30
    property string fontFamily: "Inter, sans-serif"
}
