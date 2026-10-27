// Kanso — appearance_bloc.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

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
import '../../models/data/kanso_icon_set.dart';
import '../../models/data/kanso_text_size.dart';
import '../../models/data/preferences.dart';
import '../../models/data/theme_selection.dart';
import '../../models/repos/abs/gallery_repository.dart';
import '../../models/repos/abs/preferences_repository.dart';
import '../../models/repos/abs/themes_repository.dart';
import '../../utils/palette_from_tokens.dart';

part 'appearance_bloc.freezed.dart';
part 'appearance_event.dart';
part 'appearance_state.dart';

/// Holds how the app looks: the themes it is drawn in, the icon theme, the
/// fonts of notes and the size of their text.
///
/// Every choice is shown at once and then saved in the preferences, so it is
/// still there at the next start. A choice that cannot be saved stays for
/// this run. Events are handled one after another, in the order they came.
///
/// A colour theme installed from the gallery that changes, wherever it was
/// changed, is shown again: in its new colours, or, once it is gone,
/// replaced by Kanso's own in every slot that held it, shown or not. A
/// theme that cannot be read at start is replaced the same way.
@injectable
class AppearanceBloc extends Bloc<AppearanceEvent, AppearanceState> {
  AppearanceBloc(this._preferences, this._renderer, this._gallery)
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
        AppearanceIconThemePickedEvent(:final id) => _onIconThemePicked(
          id,
          emit,
        ),
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
        AppearanceThemeChangedEvent(:final id) => _onThemeChanged(id, emit),
      },
      transformer: sequential(),
    );
    _themeChanges = _gallery.themeChanges().listen(
      (id) => add(AppearanceEvent.themeChanged(id)),
    );
  }

  final PreferencesRepository _preferences;
  final RendererRepository _renderer;
  final GalleryRepository _gallery;
  late final StreamSubscription<String> _themeChanges;
  final Map<String, KansoPalette> _palettes = <String, KansoPalette>{
    _key('kanso-light', 'kanso-light', 'kanso-light'): KansoPalette.kansoLight,
    _key('kanso-dark', 'kanso-dark', 'kanso-dark'): KansoPalette.kansoDark,
  };
  final Map<String, KansoIconSet> _iconSets = <String, KansoIconSet>{
    KansoIconSet.lucideId: KansoIconSet.lucide,
  };

  static String _key(String ui, String syntax, String preview) =>
      '$ui|$syntax|$preview';

  Future<void> _onStarted(Emitter<AppearanceState> emit) async {
    final loaded = await _preferences.load();
    final preferences = loaded.getOrElse(Preferences.new);
    final saved = preferences.themeSelection;
    // A theme that is gone (removed while the app was not running) is
    // Kanso's own in its slots from now on.
    var selection = saved;
    for (final id in <String>{saved.ui, saved.light, saved.dark}) {
      if (await _palette(id) == null) {
        selection = selection.without(id);
      }
    }
    final shown = await _paletteSlots(selection);
    final icons = await _iconSet(preferences.iconTheme);
    emit(
      AppearanceState(
        selection: selection,
        light: shown.light,
        dark: shown.dark,
        iconSet: icons ?? KansoIconSet.lucide,
        fonts: preferences.fonts,
        textSize: preferences.textSize,
        status: loaded.isRight() && shown.complete && icons != null
            ? AppearanceStatus.ready
            : AppearanceStatus.failed,
      ),
    );
    if (selection != saved) {
      await _preferences.update(
        (current) => current.copyWith(themeSelection: selection),
      );
    }
  }

  Future<void> _onThemePicked(
    String id,
    bool dark,
    Emitter<AppearanceState> emit,
  ) async {
    final selection = state.selection.picked(id, dark: dark);
    await _select(selection, emit);
  }

  Future<void> _onThemeChanged(String id, Emitter<AppearanceState> emit) async {
    _palettes.remove(id);
    final selection = state.selection;
    final without = selection.without(id);
    if (without == selection) {
      return;
    }
    if (await _palette(id) == null) {
      // Removed: Kanso's own theme in every slot that held it, shown or
      // not, so it is not back when the system's appearance changes.
      await _select(without, emit);
      return;
    }
    // Updated: the same theme in new colours.
    final shown = await _paletteSlots(selection);
    emit(state.copyWith(light: shown.light, dark: shown.dark));
  }

  /// Shows the icon theme [id] and saves it, unless it cannot be read: an
  /// icon theme that is not there changes nothing.
  Future<void> _onIconThemePicked(
    String id,
    Emitter<AppearanceState> emit,
  ) async {
    final icons = await _iconSet(id);
    if (icons == null) {
      return;
    }
    emit(state.copyWith(iconSet: icons));
    await _preferences.update((current) => current.copyWith(iconTheme: id));
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
    // A built-in theme, or one installed from the gallery.
    var tokens = await _renderer.builtinThemeTokens(id);
    if (tokens.isLeft()) {
      tokens = await _gallery.installedThemeTokens(id);
    }
    return tokens.fold((_) => null, (found) {
      final made = paletteFromTokens(ui, found);
      _palettes[key] = made;
      return made;
    });
  }

  /// The icon theme [id] with the SVGs it draws; read once and kept. `null`
  /// when it cannot be read or is not one.
  Future<KansoIconSet?> _iconSet(String id) async {
    final known = _iconSets[id];
    if (known != null) {
      return known;
    }
    final themes = await _renderer.iconThemes();
    final icons = await _renderer.iconThemeIcons(id);
    final name = themes
        .map(
          (rows) => rows
              .where((row) => row.id == id)
              .map((row) => row.name)
              .firstOrNull,
        )
        .getOrElse(() => null);
    return icons.fold((_) => null, (found) {
      if (name == null) {
        return null;
      }
      final made = KansoIconSet(
        id: id,
        name: name,
        svgs: <String, String>{
          for (final icon in found)
            if (icon.svg != null) icon.name: icon.svg!,
        },
      );
      _iconSets[id] = made;
      return made;
    });
  }

  @override
  Future<void> close() async {
    await _themeChanges.cancel();
    return super.close();
  }
}
