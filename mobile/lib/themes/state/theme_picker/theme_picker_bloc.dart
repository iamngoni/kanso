// Kanso — theme_picker_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/load_status.dart';
import '../../../core/models/data/theme_file_notice.dart';
import '../../../core/models/repos/abs/file_picker_repository.dart';
import '../../../core/models/repos/abs/themes_repository.dart';
import '../../models/data/theme_import_outcome.dart';
import '../../models/data/theme_preview.dart';
import '../../utils/theme_preview_from_installed.dart';

part 'theme_picker_bloc.freezed.dart';
part 'theme_picker_event.dart';
part 'theme_picker_state.dart';

/// The built-in, personal and gallery themes the picker offers, plus the
/// files that could not be read and a row to import another theme.
@injectable
class ThemePickerBloc extends Bloc<ThemePickerEvent, ThemePickerState> {
  ThemePickerBloc(this._themes, this._files) : super(const ThemePickerState()) {
    on<ThemePickerStartedEvent>(_onStarted, transformer: restartable());
    on<ThemePickerInstalledChangedEvent>(
      _onInstalledChanged,
      transformer: restartable(),
    );
    on<ThemePickerImportRequestedEvent>(
      _onImportRequested,
      transformer: droppable(),
    );
  }

  final ThemesRepository _themes;
  final FilePickerRepository _files;

  Future<void> _onStarted(
    ThemePickerStartedEvent event,
    Emitter<ThemePickerState> emit,
  ) async {
    emit(const ThemePickerState());
    emit(await _loaded(const ThemePickerState()));
  }

  Future<void> _onInstalledChanged(
    ThemePickerInstalledChangedEvent event,
    Emitter<ThemePickerState> emit,
  ) async {
    if (state.status != LoadStatus.ready) {
      return;
    }
    emit(await _loaded(state));
  }

  Future<void> _onImportRequested(
    ThemePickerImportRequestedEvent event,
    Emitter<ThemePickerState> emit,
  ) async {
    final picked = await _files.pickFile();
    final path = picked.fold<String?>(
      (error) {
        emit(
          state.copyWith(
            outcome: ThemeImportOutcome.failed(message: error.message),
          ),
        );
        return null;
      },
      (path) => path,
    );
    if (path == null) {
      return;
    }

    emit(state.copyWith(importing: true, outcome: null));
    final imported = await _themes.importTheme(path);
    await imported.fold<Future<void>>(
      (error) async => emit(
        state.copyWith(
          importing: false,
          outcome: ThemeImportOutcome.failed(message: error.message),
        ),
      ),
      (result) async {
        final loaded = await _loaded(state.copyWith(importing: false));
        emit(
          loaded.copyWith(
            outcome: ThemeImportOutcome.imported(names: result.names),
          ),
        );
      },
    );
  }

  Future<ThemePickerState> _loaded(ThemePickerState base) async {
    final listed = await _themes.list();
    final noticed = await _themes.notices();
    return listed.fold(
      (error) => base.copyWith(status: LoadStatus.failed),
      (themes) => base.copyWith(
        status: LoadStatus.ready,
        themes: themes.map(themePreviewFromInstalled).toList(growable: false),
        notices: noticed.getOrElse(() => const []),
      ),
    );
  }
}
