// Kanso — kanso_effects.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Opacities that tint, dim or soften a colour.
abstract final class KansoEffects {
  /// A scrim over content behind a modal surface, on a light theme: a warm
  /// dark at this opacity.
  static const double scrimLight = 0.34;

  /// A scrim over content behind a modal surface, on a dark theme.
  static const double scrimDark = 0.6;

  /// A tinted wash behind a tag or a search match.
  static const double wash = 0.18;

  /// A very quiet wash of the text colour, behind a badge or an icon.
  static const double quietWash = 0.08;

  /// A control that cannot be used.
  static const double disabled = 0.45;

  /// A control while it is pressed.
  static const double pressed = 0.6;

  /// How far a button shrinks while it is pressed.
  static const double pressScale = 0.97;

  /// A search match in running text, on a soft wash.
  static const double highlight = 0.26;
}
