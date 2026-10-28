// Kanso — installed_icon_theme.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'installed_theme_origin.dart';

part 'installed_icon_theme.freezed.dart';

/// An icon theme the phone has: one the person imported or one from the
/// gallery.
@freezed
abstract class InstalledIconTheme with _$InstalledIconTheme {
  const factory InstalledIconTheme({
    /// The icon theme's id.
    required String id,

    /// The name people see.
    required String name,

    /// Who made the icon theme; empty when it does not say.
    required String author,

    /// The icon theme's licence; empty when it does not say.
    required String license,

    /// Where the icon theme came from.
    required InstalledThemeOrigin origin,
  }) = _InstalledIconTheme;
}
