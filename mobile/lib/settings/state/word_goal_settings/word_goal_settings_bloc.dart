// Kanso — word_goal_settings_bloc.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/library_repository.dart';

part 'word_goal_settings_bloc.freezed.dart';
part 'word_goal_settings_event.dart';
part 'word_goal_settings_state.dart';

/// The default word goal for notes without a note-specific choice.
@injectable
class WordGoalSettingsBloc
    extends Bloc<WordGoalSettingsEvent, WordGoalSettingsState> {
  WordGoalSettingsBloc(this._library) : super(const WordGoalSettingsState()) {
    on<WordGoalSettingsEvent>(
      (event, emit) => switch (event) {
        WordGoalSettingsStartedEvent() => _start(emit),
        WordGoalDefaultChangedEvent(:final words) => _change(words, emit),
      },
      transformer: sequential(),
    );
  }

  final LibraryRepository _library;

  Future<void> _start(Emitter<WordGoalSettingsState> emit) async {
    final result = await _library.defaultWordGoal();
    emit(
      result.fold(
        (error) => state.copyWith(status: LoadStatus.failed, error: error),
        (words) =>
            WordGoalSettingsState(status: LoadStatus.ready, words: words),
      ),
    );
  }

  Future<void> _change(
    int? words,
    Emitter<WordGoalSettingsState> emit,
  ) async {
    final before = state.words;
    if (state.status != LoadStatus.ready || before == words) {
      return;
    }
    emit(state.copyWith(words: words, error: null));
    final saved = await _library.setDefaultWordGoal(words);
    emit(
      saved.fold(
        (error) => state.copyWith(words: before, error: error),
        (stored) => state.copyWith(words: stored, error: null),
      ),
    );
  }
}
