// Kanso — appearance_state.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'appearance_bloc.dart';

@freezed
abstract class AppearanceState with _$AppearanceState {
  const factory AppearanceState({
    /// The themes chosen.
    @Default(ThemeSelection()) ThemeSelection selection,

    /// The palette of the app's light theme: the light slot while the system
    /// is followed, the fixed theme otherwise.
    @Default(KansoPalette.kansoLight) KansoPalette light,

    /// The palette of the app's dark theme: the dark slot while the system
    /// is followed, the fixed theme otherwise (so the same as [light]).
    @Default(KansoPalette.kansoDark) KansoPalette dark,

    /// The icon theme every icon is drawn in.
    @Default(KansoIconSet.lucide) KansoIconSet iconSet,

    @Default(KansoFonts()) KansoFonts fonts,
    @Default(KansoTextSize.standard) KansoTextSize textSize,
    @Default(AppearanceStatus.loading) AppearanceStatus status,
  }) = _AppearanceState;

  const AppearanceState._();

  /// Which of the app's two themes shows: the system's pick while it is
  /// followed, else the one the fixed theme is.
  ThemeMode get themeMode {
    if (selection.followSystem) {
      return ThemeMode.system;
    }
    return light.dark ? ThemeMode.dark : ThemeMode.light;
  }
}
