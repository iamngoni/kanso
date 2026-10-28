// Kanso — theme_card_kind.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What a grid of theme cards chooses, as on the desktop's Themes page.
enum ThemeCardKind {
  /// The theme of the interface: the cards show Markdown in the theme's
  /// colours.
  ui,

  /// The theme of the Markdown source and the code in it: the cards show
  /// more tokens on the theme's editor.
  syntax,

  /// The theme of the rendered note: the cards show a heading, text, a
  /// link, a quote and code as the preview draws them.
  preview,
}
