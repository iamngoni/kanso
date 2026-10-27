// Kanso — installed_theme_origin.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where an installed theme or icon theme came from.
enum InstalledThemeOrigin {
  /// It comes with Kanso.
  builtin,

  /// The person made it, or imported it into their themes folder.
  own,

  /// It was installed from the signed gallery.
  gallery,
}
