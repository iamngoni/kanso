// Kanso — join_link_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/repos/abs/shared_notes_repository.dart';
import '../../models/data/share_problem.dart';
import '../../utils/share_problem_of.dart';

part 'join_link_bloc.freezed.dart';
part 'join_link_event.dart';
part 'join_link_state.dart';

/// Asking to join a note with an invite link.
///
/// The note's owner lets the person in; until then there is nothing to open,
/// so the answer is only who was asked. The note shows in the library once
/// they approve and a sync brings it in.
@injectable
class JoinLinkBloc extends Bloc<JoinLinkEvent, JoinLinkState> {
  JoinLinkBloc(this._shared) : super(const JoinLinkState()) {
    on<JoinLinkSubmittedEvent>(_onSubmitted, transformer: droppable());
    on<JoinLinkRestartedEvent>((event, emit) => emit(const JoinLinkState()));
  }

  final SharedNotesRepository _shared;

  Future<void> _onSubmitted(
    JoinLinkSubmittedEvent event,
    Emitter<JoinLinkState> emit,
  ) async {
    final link = event.link.trim();
    if (link.isEmpty) {
      emit(state.copyWith(problem: const ShareProblem.linkMissing()));
      return;
    }
    emit(state.copyWith(joining: true, problem: null));
    final joined = await _shared.join(link);
    emit(
      joined.fold(
        (error) =>
            state.copyWith(joining: false, problem: shareProblemOf(error)),
        (asked) => state.copyWith(joining: false, asked: asked),
      ),
    );
  }
}
