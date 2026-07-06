import SwiftUI

// Warm-graphite palette from the macOS Desktop Design Spec. Calm, restrained,
// off-white on graphite, muted blue-gray accent. Dark mode is primary.
extension Color {
    init(hex: UInt32) {
        let r = Double((hex >> 16) & 0xFF) / 255
        let g = Double((hex >> 8) & 0xFF) / 255
        let b = Double(hex & 0xFF) / 255
        self.init(.sRGB, red: r, green: g, blue: b, opacity: 1)
    }
}

enum Theme {
    static let appBg = Color(hex: 0x181816)
    static let sidebar = Color(hex: 0x1D1D1A)
    static let noteList = Color(hex: 0x232320)
    static let editor = Color(hex: 0x282824)
    static let elevated = Color(hex: 0x30302B)

    static let textPrimary = Color(hex: 0xF1EFE7)
    static let textSecondary = Color(hex: 0xB7B3A8)
    static let textMuted = Color(hex: 0x7E7A70)
    static let divider = Color(hex: 0x3A3933)

    static let accent = Color(hex: 0x7E96B3)
    static let success = Color(hex: 0x7BAA78)
    static let warning = Color(hex: 0xC49A5A)
    static let destructive = Color(hex: 0xC96E63)
}
