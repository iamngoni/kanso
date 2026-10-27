// Kanso — appearance_bloc.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:flutter/material.dart' show ThemeMode;
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../bridge/api/types.dart' show DiagramRenderer;
import '../../configs/kanso_palette.dart';
import '../../models/data/appearance_status.dart';
import '../../models/data/kanso_font_family.dart';
import '../../models/data/kanso_fonts.dart';
import '../../models/data/kanso_text_size.dart';
import '../../models/data/preferences.dart';
import '../../models/data/theme_selection.dart';
import '../../models/repos/abs/preferences_repository.dart';
import '../../models/repos/abs/themes_repository.dart';
import '../../utils/palette_from_tokens.dart';

part 'appearance_bloc.freezed.dart';
part 'appearance_event.dart';
part 'appearance_state.dart';

/// Holds how the app looks: the themes it is drawn in (the interface's, the
/// Markdown source's and the preview's, fitted together by the core), the
/// fonts of notes, the size of their text and what draws diagrams.
///
/// Every choice is shown at once and then saved in the preferences, so it is
/// still there at the next start. A choice that cannot be saved stays for
/// this run. Events are handled one after another, in the order they came.
@injectable
class AppearanceBloc extends Bloc<AppearanceEvent, AppearanceState> {
  AppearanceBloc(this._preferences, this._themes)
    : super(const AppearanceState()) {
    // One handler for every event, so they all run one after another in the
    // order they came (a transformer orders only the events of one handler).
    on<AppearanceEvent>(
      (event, emit) => switch (event) {
        AppearanceStartedEvent() => _onStarted(emit),
        AppearanceThemePickedEvent(:final id, :final dark) => _onThemePicked(
          id,
          dark,
          emit,
        ),
        AppearanceSyntaxPickedEvent(:final id) => _select(
          state.selection.copyWith(syntax: id),
          emit,
        ),
        AppearancePreviewPickedEvent(:final id) => _select(
          state.selection.copyWith(preview: id),
          emit,
        ),
        AppearanceFollowSystemChangedEvent(:final follow) =>
          _onFollowSystemChanged(follow, emit),
        AppearanceEditorFontChangedEvent(:final font) => _changeFonts(
          emit,
          state.fonts.copyWith(editorFont: font),
        ),
        AppearancePreviewFontChangedEvent(:final font) => _changeFonts(
          emit,
          state.fonts.copyWith(previewFont: font),
        ),
        // Code is always monospaced: any other family is ignored.
        AppearanceCodeFontChangedEvent(:final font) =>
          font.monospaced
              ? _changeFonts(emit, state.fonts.copyWith(codeFont: font))
              : Future<void>.value(),
        AppearanceTextSizeChangedEvent(:final size) => _onTextSizeChanged(
          size,
          emit,
        ),
        AppearancePreviewTextSizeChangedEvent(:final size) =>
          _onPreviewTextSizeChanged(size, emit),
        AppearanceDiagramRendererChangedEvent(:final renderer) =>
          _onDiagramRendererChanged(renderer, emit),
      },
      transformer: sequential(),
    );
  }

  final PreferencesRepository _preferences;
  final ThemesRepository _themes;

  /// The palettes read so far, by the combination they were fitted for (see
  /// [_key]); Kanso's own are known without asking.
  final Map<String, KansoPalette> _palettes = <String, KansoPalette>{
    _key('kanso-light', 'kanso-light', 'kanso-light'): KansoPalette.kansoLight,
    _key('kanso-dark', 'kanso-dark', 'kanso-dark'): KansoPalette.kansoDark,
  };

  static String _key(String ui, String syntax, String preview) =>
      '$ui|$syntax|$preview';

  Future<void> _onStarted(Emitter<AppearanceState> emit) async {
    final loaded = await _preferences.load();
    final preferences = loaded.getOrElse(Preferences.new);
    final shown = await _paletteSlots(preferences.themeSelection);
    emit(
      AppearanceState(
        selection: preferences.themeSelection,
        light: shown.light,
        dark: shown.dark,
        fonts: preferences.fonts,
        textSize: preferences.textSize,
        previewTextSize: preferences.previewTextSize,
        diagramRenderer: preferences.diagramRenderer,
        status: loaded.isRight() && shown.complete
            ? AppearanceStatus.ready
            : AppearanceStatus.failed,
      ),
    );
    // The core draws diagrams as it is told, so it is told at every start.
    await _themes.setDiagramRenderer(preferences.diagramRenderer);
  }

  Future<void> _onThemePicked(
    String id,
    bool dark,
    Emitter<AppearanceState> emit,
  ) async {
    final selection = state.selection.picked(id, dark: dark);
    await _select(selection, emit);
  }

  Future<void> _onFollowSystemChanged(
    bool follow,
    Emitter<AppearanceState> emit,
  ) async {
    final selection = state.selection.copyWith(followSystem: follow);
    await _select(selection, emit);
  }

  Future<void> _onTextSizeChanged(
    KansoTextSize size,
    Emitter<AppearanceState> emit,
  ) async {
    emit(state.copyWith(textSize: size));
    await _preferences.update((current) => current.copyWith(textSize: size));
  }

  Future<void> _onPreviewTextSizeChanged(
    KansoTextSize size,
    Emitter<AppearanceState> emit,
  ) async {
    emit(state.copyWith(previewTextSize: size));
    await _preferences.update(
      (current) => current.copyWith(previewTextSize: size),
    );
  }

  Future<void> _onDiagramRendererChanged(
    DiagramRenderer renderer,
    Emitter<AppearanceState> emit,
  ) async {
    emit(state.copyWith(diagramRenderer: renderer));
    await _themes.setDiagramRenderer(renderer);
    await _preferences.update(
      (current) => current.copyWith(diagramRenderer: renderer),
    );
  }

  Future<void> _changeFonts(
    Emitter<AppearanceState> emit,
    KansoFonts fonts,
  ) async {
    emit(state.copyWith(fonts: fonts));
    await _preferences.update((current) => current.copyWith(fonts: fonts));
  }

  /// Shows [selection] and saves it, unless a theme in it cannot be read: a
  /// theme that is not there changes nothing.
  Future<void> _select(
    ThemeSelection selection,
    Emitter<AppearanceState> emit,
  ) async {
    final shown = await _paletteSlots(selection);
    if (!shown.complete) {
      return;
    }
    emit(
      state.copyWith(
        selection: selection,
        light: shown.light,
        dark: shown.dark,
      ),
    );
    await _preferences.update(
      (current) => current.copyWith(themeSelection: selection),
    );
  }

  /// The palettes of the app's light and dark slots for [selection], each
  /// drawn from the tokens the core fits for the slot's interface, syntax
  /// and preview themes together, and whether every combination was read (a
  /// slot that was not is shown as Kanso's own).
  Future<({KansoPalette light, KansoPalette dark, bool complete})>
  _paletteSlots(ThemeSelection selection) async {
    var complete = true;
    Future<KansoPalette> palette({
      required bool dark,
      required KansoPalette fallback,
    }) async {
      final found = await _palette(
        selection.uiFor(systemDark: dark),
        selection.syntaxFor(systemDark: dark),
        selection.previewFor(systemDark: dark),
      );
      if (found == null) {
        complete = false;
      }
      return found ?? fallback;
    }

    if (selection.followSystem) {
      final light = await palette(
        dark: false,
        fallback: KansoPalette.kansoLight,
      );
      final dark = await palette(dark: true, fallback: KansoPalette.kansoDark);
      return (light: light, dark: dark, complete: complete);
    }
    final fixed = await palette(dark: true, fallback: KansoPalette.kansoDark);
    return (light: fixed, dark: fixed, complete: complete);
  }

  Future<KansoPalette?> _palette(
    String ui,
    String syntax,
    String preview,
  ) async {
    final key = _key(ui, syntax, preview);
    final known = _palettes[key];
    if (known != null) {
      return known;
    }
    final tokens = await _themes.fit(ui, syntax, preview);
    return tokens.fold((_) => null, (found) {
      final made = paletteFromTokens(ui, found);
      _palettes[key] = made;
      return made;
    });
  }
}
