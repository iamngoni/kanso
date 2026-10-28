// Kanso — kanso_icon_set.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'kanso_icon_set.freezed.dart';

/// The icon theme the app is drawn in: its [id], its [name] and the SVGs it
/// draws, by the Lucide name of the icon they stand for.
///
/// An icon the theme leaves out is not in [svgs]; the app draws its own
/// Lucide icon there, icon by icon. Kanso's own icons are the theme that
/// leaves them all out ([KansoIconSet.lucide]).
@freezed
abstract class KansoIconSet with _$KansoIconSet {
  const factory KansoIconSet({
    /// The theme's folder name: `lucide`, `phosphor`.
    @Default(KansoIconSet.lucideId) String id,

    /// The name shown in Settings.
    @Default(KansoIconSet.lucideName) String name,

    /// The theme's SVG for each icon it draws, by Lucide name.
    @Default(<String, String>{}) Map<String, String> svgs,
  }) = _KansoIconSet;

  const KansoIconSet._();

  /// Kanso's own icons: Lucide, drawn from the bundled files.
  static const KansoIconSet lucide = KansoIconSet();

  /// The ID of the theme drawn when none is chosen.
  static const String lucideId = 'lucide';

  /// The name of that theme.
  static const String lucideName = 'Lucide';

  /// The theme's SVG for the icon [lucideName], or `null` where the bundled
  /// Lucide icon is drawn.
  String? svgFor(String lucideName) => svgs[lucideName];
}
