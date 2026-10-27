// Kanso — theme_selection.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'theme_selection.freezed.dart';
part 'theme_selection.g.dart';

/// Which themes the app is drawn in: the desktop's `ThemeSettings`.
///
/// Themes are named by their file's name without `.json` (`nord`). Following
/// the system shows [light] in light mode and [dark] in dark mode; otherwise
/// [ui] is shown whatever the mode. The Markdown source and the rendered
/// preview may each have a theme of their own ([syntax], [preview]); without
/// one they are drawn in the interface's theme.
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

    /// The theme that colours the Markdown source and the code in it; null
    /// for the interface's own.
    String? syntax,

    /// The theme that colours the rendered note; null for the interface's
    /// own.
    String? preview,
  }) = _ThemeSelection;

  const ThemeSelection._();

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

  /// The theme that colours the Markdown source while the system is in dark
  /// mode ([systemDark]) or light mode: [syntax], or the interface's theme
  /// when there is none.
  String syntaxFor({required bool systemDark}) =>
      syntax ?? uiFor(systemDark: systemDark);

  /// The theme that colours the rendered note while the system is in dark
  /// mode ([systemDark]) or light mode: [preview], or the interface's theme
  /// when there is none.
  String previewFor({required bool systemDark}) =>
      preview ?? uiFor(systemDark: systemDark);

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

  /// The themes the app needs read: the ones it is drawn in and the syntax
  /// and preview themes, each once.
  Set<String> get allIds => <String>{...ids, ?syntax, ?preview};
}
