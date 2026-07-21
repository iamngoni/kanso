import SwiftUI
import AppKit

extension Color {
    init(hex: UInt32) {
        let r = Double((hex >> 16) & 0xFF) / 255
        let g = Double((hex >> 8) & 0xFF) / 255
        let b = Double(hex & 0xFF) / 255
        self.init(.sRGB, red: r, green: g, blue: b, opacity: 1)
    }

    init(light: UInt32, dark: UInt32) {
        self.init(nsColor: NSColor(name: nil) { appearance in
            let match = appearance.bestMatch(from: [.aqua, .darkAqua])
            return NSColor(hex: match == .darkAqua ? dark : light)
        })
    }
}

private extension NSColor {
    convenience init(hex: UInt32) {
        let r = CGFloat((hex >> 16) & 0xFF) / 255
        let g = CGFloat((hex >> 8) & 0xFF) / 255
        let b = CGFloat(hex & 0xFF) / 255
        self.init(srgbRed: r, green: g, blue: b, alpha: 1)
    }
}

enum Theme {
    static let appBg = Color(light: 0xF6F2EA, dark: 0x161719)
    static let sidebar = Color(light: 0xECE7DD, dark: 0x242528)
    static let noteList = Color(light: 0xF3EEE5, dark: 0x202124)
    static let editor = Color(light: 0xFCFAF5, dark: 0x1B1C1F)
    static let elevated = Color(light: 0xFFFFFF, dark: 0x2B2D31)
    static let panelElevated = Color(light: 0xFFFFFF, dark: 0x26282C)
    static let field = Color(light: 0xEEE7DA, dark: 0x17181B)

    static let textPrimary = Color(light: 0x24231F, dark: 0xF2F0E8)
    static let textSecondary = Color(light: 0x5C584F, dark: 0xC6C1B7)
    static let textMuted = Color(light: 0x8B8578, dark: 0x89857B)
    static let divider = Color(light: 0xD8D0C4, dark: 0x383A3E)

    static let accent = Color(light: 0x4C6F9F, dark: 0x8BA6CF)
    static let success = Color(light: 0x4E7E4B, dark: 0x7FBA86)
    static let warning = Color(light: 0x9B6A28, dark: 0xD0A15E)
    static let destructive = Color(light: 0xA54F45, dark: 0xD1756A)
}
