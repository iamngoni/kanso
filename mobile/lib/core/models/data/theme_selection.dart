// Kanso — theme_selection.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'theme_selection.freezed.dart';
part 'theme_selection.g.dart';

/// Which built-in themes the app is drawn in: the desktop's `ThemeSettings`,
/// for the interface.
///
/// Themes are named by their file's name without `.json` (`nord`). Following
/// the system shows [light] in light mode and [dark] in dark mode; otherwise
/// [ui] is shown whatever the mode.
@freezed
abstract class ThemeSelection with _$ThemeSelection {
  const factory ThemeSelection({
    /// Whether the system's light or dark mode picks the theme.
    @Default(true) bool followSystem,

    /// The theme shown while the system is not followed.
    @Default('kanso-dark') String ui,

    /// The theme shown in light mode while the system is followed.
    @Default('kanso-light') String light,

    /// The theme shown in dark mode while the system is followed.
    @Default('kanso-dark') String dark,
  }) = _ThemeSelection;

  const ThemeSelection._();

  /// The themes Kanso always has, which a slot falls back to when the theme
  /// in it is removed.
  static const String kansoLight = 'kanso-light';
  static const String kansoDark = 'kanso-dark';

  factory ThemeSelection.fromJson(Map<String, dynamic> json) =>
      _$ThemeSelectionFromJson(json);

  /// The theme to show while the system is in dark mode ([systemDark]) or
  /// light mode.
  String uiFor({required bool systemDark}) {
    if (!followSystem) {
      return ui;
    }
    return systemDark ? dark : light;
  }

  /// The selection after the theme [id] is picked, [dark] being whether it is
  /// a dark theme.
  ///
  /// As on the desktop, a theme picked while the system is followed takes the
  /// slot of its own appearance, and becomes the fixed choice too, for when
  /// following stops. A theme picked otherwise is the fixed choice only.
  ThemeSelection picked(String id, {required bool dark}) {
    if (!followSystem) {
      return copyWith(ui: id);
    }
    return dark ? copyWith(ui: id, dark: id) : copyWith(ui: id, light: id);
  }

  /// Whether the theme [id] is one of the ones in use: the light or dark one
  /// while the system is followed, the fixed one otherwise.
  bool isSelected(String id) {
    return followSystem ? id == light || id == dark : id == ui;
  }

  /// The themes the app needs drawn: both slots while following the system,
  /// the fixed one otherwise.
  List<String> get ids {
    if (!followSystem) {
      return <String>[ui];
    }
    return light == dark ? <String>[light] : <String>[light, dark];
  }

  /// The selection once the theme [id] is gone (removed from the gallery's
  /// installed themes): each slot that held it is Kanso's own theme for that
  /// slot, and the rest is as it was.
  ThemeSelection without(String id) {
    return copyWith(
      ui: ui == id ? kansoDark : ui,
      light: light == id ? kansoLight : light,
      dark: dark == id ? kansoDark : dark,
    );
  }
}
