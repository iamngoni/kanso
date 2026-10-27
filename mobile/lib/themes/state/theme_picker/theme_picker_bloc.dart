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

/// The themes the picker offers: every installed theme (built in, the
/// person's own and the gallery's) in the desktop's order, each with the
/// preview drawn from its tokens; the theme files that were left out; and
/// importing a theme from a file.
///
/// Which are in use, and picking one, are the `AppearanceBloc`'s.
@injectable
class ThemePickerBloc extends Bloc<ThemePickerEvent, ThemePickerState> {
  ThemePickerBloc(this._themes, this._files) : super(const ThemePickerState()) {
    on<ThemePickerStartedEvent>(_onStarted, transformer: restartable());
    // A second tap while a file is being imported does nothing.
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

  Future<void> _onImportRequested(
    ThemePickerImportRequestedEvent event,
    Emitter<ThemePickerState> emit,
  ) async {
    final picked = await _files.pickFile();
    final path = picked.fold<String?>((error) => null, (path) => path);
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
    if (path == null) {
      return;
    }
    emit(state.copyWith(importing: true, outcome: null));
    final imported = await _themes.importTheme(path);
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
}
