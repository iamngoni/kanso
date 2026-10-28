// Kanso — word_goal_intent_bloc.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

part 'word_goal_intent_event.dart';
part 'word_goal_intent_state.dart';
part 'word_goal_intent_bloc.freezed.dart';

/// Carries a requested word-goal picker from app-level surfaces, such as
/// the command palette, to the mounted note editor.
@injectable
class WordGoalIntentBloc
    extends Bloc<WordGoalIntentEvent, WordGoalIntentState> {
  WordGoalIntentBloc() : super(const WordGoalIntentState()) {
    on<WordGoalIntentRequestedEvent>((event, emit) {
      if (event.noteId.isEmpty) {
        return;
      }
      emit(
        WordGoalIntentState(
          request: state.request + 1,
          noteId: event.noteId,
        ),
      );
    });
    on<WordGoalIntentConsumedEvent>((event, emit) {
      if (event.request == state.request) {
        emit(WordGoalIntentState(request: state.request));
      }
    });
  }
}
