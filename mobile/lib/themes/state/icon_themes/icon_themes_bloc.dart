// Kanso — icon_themes_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/installed_icon_theme.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/file_picker_repository.dart';
import '../../../core/models/repos/abs/themes_repository.dart';
import '../../models/data/theme_import_outcome.dart';

part 'icon_themes_bloc.freezed.dart';
part 'icon_themes_event.dart';
part 'icon_themes_state.dart';

/// The icon themes the phone has, and importing one from a file.
///
/// Choosing one, and drawing the app's icons from it, come later; this list
/// is what they will add a selection to.
@injectable
class IconThemesBloc extends Bloc<IconThemesEvent, IconThemesState> {
  IconThemesBloc(this._themes, this._files) : super(const IconThemesState()) {
    on<IconThemesStartedEvent>(_onStarted, transformer: restartable());
    // A second tap while a file is being imported does nothing.
    on<IconThemesImportRequestedEvent>(
      _onImportRequested,
      transformer: droppable(),
    );
  }

  final ThemesRepository _themes;
  final FilePickerRepository _files;

  Future<void> _onStarted(
    IconThemesStartedEvent event,
    Emitter<IconThemesState> emit,
  ) async {
    emit(const IconThemesState());
    emit(await _loaded(const IconThemesState()));
  }

  Future<void> _onImportRequested(
    IconThemesImportRequestedEvent event,
    Emitter<IconThemesState> emit,
  ) async {
    final picked = await _files.pickFile();
    final pickError = picked.fold<String?>(
      (error) => error.message,
      (_) => null,
    );
    if (pickError != null) {
      emit(
        state.copyWith(outcome: ThemeImportOutcome.failed(message: pickError)),
      );
      return;
    }
    final path = picked.fold<String?>((error) => null, (path) => path);
    if (path == null) {
      return;
    }
    emit(state.copyWith(importing: true, outcome: null));
    final imported = await _themes.importIconTheme(path);
    await imported.fold(
      (error) async => emit(
        state.copyWith(
          importing: false,
          outcome: ThemeImportOutcome.failed(message: error.message),
        ),
      ),
      (result) async {
        final reloaded = await _loaded(state.copyWith(importing: false));
        emit(
          reloaded.copyWith(
            outcome: ThemeImportOutcome.imported(
              names: result.names,
              summary: result.summary,
            ),
          ),
        );
      },
    );
  }

  Future<IconThemesState> _loaded(IconThemesState base) async {
    final listed = await _themes.iconThemes();
    return listed.fold(
      (error) => base.copyWith(status: LoadStatus.failed),
      (themes) => base.copyWith(status: LoadStatus.ready, themes: themes),
    );
  }
}
