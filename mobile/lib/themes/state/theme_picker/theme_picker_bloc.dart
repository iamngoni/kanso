// Kanso — theme_picker_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/gallery_repository.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../models/data/theme_preview.dart';
import '../../utils/theme_preview_from_installed.dart';

part 'theme_picker_bloc.freezed.dart';
part 'theme_picker_event.dart';
part 'theme_picker_state.dart';

/// The themes the picker offers: the built-in ones in the desktop's order,
/// then the ones installed from the gallery, each with the preview drawn
/// from its tokens.
///
/// Which are in use, and picking one, are the `AppearanceBloc`'s.
@injectable
class ThemePickerBloc extends Bloc<ThemePickerEvent, ThemePickerState> {
  ThemePickerBloc(this._renderer, this._gallery)
    : super(const ThemePickerState()) {
    on<ThemePickerStartedEvent>(_onStarted, transformer: restartable());
    on<ThemePickerInstalledChangedEvent>(
      _onInstalledChanged,
      transformer: restartable(),
    );
  }

  final RendererRepository _renderer;
  final GalleryRepository _gallery;

  Future<void> _onStarted(
    ThemePickerStartedEvent event,
    Emitter<ThemePickerState> emit,
  ) async {
    emit(const ThemePickerState());
    final result = await _renderer.builtinThemes();
    final builtIn = result.fold<List<BuiltinTheme>?>((_) => null, (all) => all);
    if (builtIn == null) {
      emit(const ThemePickerState(status: LoadStatus.failed));
      return;
    }
    emit(
      ThemePickerState(
        status: LoadStatus.ready,
        themes: <ThemePreview>[
          ...builtIn.map(themePreviewFromBuiltin),
          ...await _installed(),
        ],
      ),
      (result) async {
        final reloaded = await _loaded(state.copyWith(importing: false));
        emit(
          reloaded.copyWith(
            outcome: ThemeImportOutcome.imported(names: result.names),
          ),
        );
      },
    );
  }

  /// [base] with the themes and notices read afresh; failed when the
  /// themes cannot be read (notices that cannot are none).
  Future<ThemePickerState> _loaded(ThemePickerState base) async {
    final listed = await _themes.list();
    final noticed = await _themes.notices();
    return listed.fold(
      (error) => base.copyWith(status: LoadStatus.failed),
      (themes) => base.copyWith(
        status: LoadStatus.ready,
        themes: <ThemePreview>[...themes.map(themePreviewFromInstalled)],
        notices: noticed.getOrElse(() => const <ThemeFileNotice>[]),
      ),
    );
  }

  Future<void> _onInstalledChanged(
    ThemePickerInstalledChangedEvent event,
    Emitter<ThemePickerState> emit,
  ) async {
    if (state.status != LoadStatus.ready) {
      return;
    }
    final installed = await _installed();
    emit(
      state.copyWith(
        themes: <ThemePreview>[
          ...state.themes.where((theme) => theme.builtIn),
          ...installed,
        ],
      ),
    );
  }

  /// The previews of the themes installed from the gallery. Themes that
  /// can't be read are left out: the picker still has the built-in ones.
  Future<List<ThemePreview>> _installed() async {
    final read = await _gallery.installedThemes();
    return read.fold(
      (_) => const <ThemePreview>[],
      (themes) => <ThemePreview>[
        for (final theme in themes)
          themePreviewFromBuiltin(theme, builtIn: false),
      ],
    );
  }
}
