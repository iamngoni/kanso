// Kanso — WidgetPalette.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import SwiftUI

/// The colors a widget draws with.
///
/// The app writes them to `widget.json` beside the library whenever the theme
/// or the appearance changes (see `mobile/docs/capture.md`). Until it has, or
/// for any color the file leaves out, the widget uses Kanso Light and Kanso
/// Dark.
struct WidgetPalette {
  var background: Color
  var surface: Color
  var text: Color
  var muted: Color
  var accent: Color

  /// Kanso Light: the theme's app, elevated, text, muted text and accent colors.
  static let light = WidgetPalette(
    background: Color(hex: "#F4F1EA")!,
    surface: Color(hex: "#FFFDF8")!,
    text: Color(hex: "#26241F")!,
    muted: Color(hex: "#857C6E")!,
    accent: Color(hex: "#5E7895")!)

  /// Kanso Dark, the same colors.
  static let dark = WidgetPalette(
    background: Color(hex: "#181816")!,
    surface: Color(hex: "#282824")!,
    text: Color(hex: "#F1EFE7")!,
    muted: Color(hex: "#7E7A70")!,
    accent: Color(hex: "#7E96B3")!)

  /// The palette for an appearance, read from `widget.json` when it is there.
  static func current(for scheme: ColorScheme) -> WidgetPalette {
    let fallback = scheme == .dark ? dark : light
    guard
      let location = KansoLibraryLocation.shared(),
      let data = try? Data(contentsOf: location.widgetColors),
      let root = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
      let colors = root[scheme == .dark ? "dark" : "light"] as? [String: String]
    else { return fallback }

    func color(_ key: String, _ fallback: Color) -> Color {
      colors[key].flatMap { Color(hex: $0) } ?? fallback
    }
    return WidgetPalette(
      background: color("background", fallback.background),
      surface: color("surface", fallback.surface),
      text: color("text", fallback.text),
      muted: color("muted", fallback.muted),
      accent: color("accent", fallback.accent))
  }
}

extension Color {
  /// A color from `#RRGGBB` or `#RRGGBBAA`; `nil` for anything else.
  init?(hex: String) {
    var digits = hex.trimmingCharacters(in: .whitespaces)
    if digits.hasPrefix("#") { digits.removeFirst() }
    guard digits.count == 6 || digits.count == 8, let value = UInt64(digits, radix: 16)
    else { return nil }
    let hasAlpha = digits.count == 8
    let rgb = hasAlpha ? value >> 8 : value
    self.init(
      .sRGB,
      red: Double((rgb >> 16) & 0xFF) / 255,
      green: Double((rgb >> 8) & 0xFF) / 255,
      blue: Double(rgb & 0xFF) / 255,
      opacity: hasAlpha ? Double(value & 0xFF) / 255 : 1)
  }
}
