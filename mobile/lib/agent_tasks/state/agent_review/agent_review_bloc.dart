// Kanso — agent_review_bloc.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/agent_review.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/agent_tasks_repository.dart';

part 'agent_review_bloc.freezed.dart';
part 'agent_review_event.dart';
part 'agent_review_state.dart';

/// Reviewing what an agent proposed: tick the cards to apply, apply them in
/// one step, take them back in one. One sequential handler, so a second tap
/// on Apply can't run before the first has finished.
@injectable
class AgentReviewBloc extends Bloc<AgentReviewEvent, AgentReviewState> {
  AgentReviewBloc(this._tasks) : super(const AgentReviewState()) {
    on<AgentReviewEvent>(_handle, transformer: sequential());
  }

  final AgentTasksRepository _tasks;

  Future<void> _handle(
    AgentReviewEvent event,
    Emitter<AgentReviewState> emit,
  ) async {
    switch (event) {
      case AgentReviewOpenedEvent():
        await _onOpened(event, emit);
      case AgentReviewCardToggledEvent():
        emit(
          state.copyWith(
            ticked: _toggled(state.ticked, event.index, event.on),
          ),
        );
      case AgentReviewKeepToggledEvent():
        emit(
          state.copyWith(
            keep: _toggled(state.keep, event.index, event.on),
          ),
        );
      case AgentReviewAppliedEvent():
        await _onApplied(emit);
      case AgentReviewUndoneEvent():
        await _onUndone(emit);
    }
  }

  static Set<int> _toggled(Set<int> set, int index, bool on) =>
      on ? <int>{...set, index} : (<int>{...set}..remove(index));

  Future<void> _onOpened(
    AgentReviewOpenedEvent event,
    Emitter<AgentReviewState> emit,
  ) async {
    emit(AgentReviewState(task: event.task));
    final result = await _tasks.review(event.task);
    result.fold(
      (error) => emit(
        state.copyWith(status: LoadStatus.failed, trouble: error.message),
      ),
      (review) => emit(
        state.copyWith(
          status: LoadStatus.ready,
          review: review,
          ticked: <int>{
            for (final card in review.cards)
              if (card.ticked) card.index,
          },
        ),
      ),
    );
  }

  Future<void> _onApplied(Emitter<AgentReviewState> emit) async {
    if (state.applying || state.applied != null) {
      return;
    }
    emit(state.copyWith(applying: true, trouble: null));
    final result = await _tasks.apply(
      id: state.task,
      ticked: state.ticked.toList()..sort(),
      keepOriginals: state.keep.toList()..sort(),
    );
    result.fold(
      (error) => emit(state.copyWith(applying: false, trouble: error.message)),
      (sentence) => emit(state.copyWith(applying: false, applied: sentence)),
    );
  }

  Future<void> _onUndone(Emitter<AgentReviewState> emit) async {
    if (state.applied == null) {
      return;
    }
    final result = await _tasks.undo(state.task);
    result.fold(
      (error) => emit(state.copyWith(trouble: error.message)),
      (sentence) => emit(state.copyWith(applied: null, undone: sentence)),
    );
  }
}
