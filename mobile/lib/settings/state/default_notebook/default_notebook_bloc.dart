// Kanso — default_notebook_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/repos/abs/preferences_repository.dart';

part 'default_notebook_bloc.freezed.dart';
part 'default_notebook_event.dart';
part 'default_notebook_state.dart';

/// The notebook new notes go in, which the person chooses in Settings.
///
/// The choice is shown at once and then saved in the preferences, so it is
/// still there at the next start; one that cannot be saved stays for this
/// run. Whether the notebook still exists, and is not locked, is for the
/// screens and the library to see: the choice is only the id.
@injectable
class DefaultNotebookBloc
    extends Bloc<DefaultNotebookEvent, DefaultNotebookState> {
  DefaultNotebookBloc(this._preferences) : super(const DefaultNotebookState()) {
    on<DefaultNotebookStartedEvent>(_onStarted, transformer: restartable());
    on<DefaultNotebookChosenEvent>(_onChosen, transformer: sequential());
  }

  final PreferencesRepository _preferences;

  Future<void> _onStarted(
    DefaultNotebookStartedEvent event,
    Emitter<DefaultNotebookState> emit,
  ) async {
    final loaded = await _preferences.load();
    emit(
      DefaultNotebookState(
        notebookId: loaded.fold(
          (_) => null,
          (found) => found.defaultNotebookId,
        ),
      ),
    );
  }

  Future<void> _onChosen(
    DefaultNotebookChosenEvent event,
    Emitter<DefaultNotebookState> emit,
  ) async {
    emit(DefaultNotebookState(notebookId: event.notebookId));
    await _preferences.update(
      (current) => current.copyWith(defaultNotebookId: event.notebookId),
    );
  }
}
