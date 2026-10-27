// Kanso — kanso_sizes.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Fixed dimensions of controls, rows, bars and panes.
abstract final class KansoSizes {
  /// 44: the least a touch target measures, in each direction.
  static const double touchTarget = 44;

  /// 52: a top or bottom bar, below the safe area.
  static const double barHeight = 52;

  /// 44: a row of the library.
  static const double sidebarRow = 44;

  /// 48: a row of settings.
  static const double settingsRow = 48;

  /// 48: a button.
  static const double button = 48;

  /// 44: a search field.
  static const double searchField = 44;

  /// 32: a filter chip.
  static const double chip = 32;

  /// 56: the button for a new note that floats over a phone's list.
  static const double floatingButton = 56;

  /// 22: a tag chip.
  static const double tagChip = 22;

  /// 16: a small icon beside text.
  static const double iconSmall = 16;

  /// 20: an icon in a row or a bar.
  static const double icon = 20;

  /// 14: a status glyph in a meta line.
  static const double glyph = 14;

  /// 8: a coloured dot for a tag.
  static const double dot = 8;

  /// 51 by 31: a switch.
  static const double switchWidth = 51;
  static const double switchHeight = 31;

  /// 20: a checkbox.
  static const double checkbox = 20;

  /// 28: the app's mark in the library header.
  static const double mark = 28;

  /// 280: the library pane of a tablet.
  static const double libraryPane = 280;

  /// 340: the note list pane of a tablet.
  static const double listPane = 340;

  /// 600: a sheet's width on a tablet.
  static const double sheetWidth = 600;

  /// 340: a message card's width.
  static const double messageCardWidth = 340;

  /// 0.85: the share of the width a drawer takes on a phone.
  static const double drawerFraction = 0.85;

  /// 256: a theme card's width in a horizontal row.
  static const double themeCardWidth = 256;
}
