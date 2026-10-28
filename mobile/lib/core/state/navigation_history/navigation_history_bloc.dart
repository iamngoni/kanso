// Kanso — navigation_history_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/data/navigation_history.dart';
import '../../models/repos/abs/library_repository.dart';

part 'navigation_history_bloc.freezed.dart';
part 'navigation_history_event.dart';
part 'navigation_history_state.dart';

/// The id of the trash notebook: a note in it is gone from the person's
/// point of view.
const String trashNotebookId = 'trash';

/// Back and forward between the notes opened, as in a browser (the desktop's
/// `NavHistory`).
///
/// A screen records each note it opens as a visit and, when `pendingNoteId`
/// appears in the state, opens that note and says `consumed`. A note that is
/// gone (deleted, or in the trash) is skipped, and asked of the library.
@injectable
class NavigationHistoryBloc
    extends Bloc<NavigationHistoryEvent, NavigationHistoryState> {
  NavigationHistoryBloc(this._library) : super(const NavigationHistoryState()) {
    // One handler for every event, so each sees the history the one before
    // left.
    on<NavigationHistoryEvent>(
      (event, emit) => switch (event) {
        NavigationHistoryVisitedEvent(:final noteId) => _onVisited(
          noteId,
          emit,
        ),
        NavigationHistoryBackRequestedEvent() => _onStep(back: true, emit),
        NavigationHistoryForwardRequestedEvent() => _onStep(back: false, emit),
        NavigationHistoryConsumedEvent() => Future<void>.sync(
          () => emit(state.copyWith(pendingNoteId: null)),
        ),
      },
      transformer: sequential(),
    );
  }

  final LibraryRepository _library;

  Future<void> _onVisited(
    String noteId,
    Emitter<NavigationHistoryState> emit,
  ) async {
    final history = state.history.visit(noteId);
    if (history == state.history) {
      return;
    }
    emit(await _withReach(state.copyWith(history: history)));
  }

  Future<void> _onStep(
    Emitter<NavigationHistoryState> emit, {
    required bool back,
  }) async {
    final history = state.history;
    final live = await _liveness(
      back ? history.candidatesBack : history.candidatesForward,
      history.current,
    );
    bool isLive(String id) => live[id] ?? false;
    final stepped = back ? history.goBack(isLive) : history.goForward(isLive);
    emit(
      await _withReach(
        state.copyWith(
          history: stepped.history,
          pendingNoteId: stepped.landed ?? state.pendingNoteId,
        ),
      ),
    );
  }

  /// [next] with whether going back and forward would land anywhere.
  Future<NavigationHistoryState> _withReach(NavigationHistoryState next) async {
    final history = next.history;
    final backward = await _liveness(history.candidatesBack, history.current);
    final forward = await _liveness(history.candidatesForward, history.current);
    return next.copyWith(
      canGoBack: history.canGoBack((id) => backward[id] ?? false),
      canGoForward: history.canGoForward((id) => forward[id] ?? false),
    );
  }

  /// Whether each of [candidates] is still there, asked of the library
  /// nearest first, up to and including the first that is live and is not
  /// [current]: the ones beyond cannot matter.
  Future<Map<String, bool>> _liveness(
    Iterable<String> candidates,
    String? current,
  ) async {
    final answers = <String, bool>{};
    for (final id in candidates) {
      if (id == current || answers.containsKey(id)) {
        continue;
      }
      final alive = await _isLive(id);
      answers[id] = alive;
      if (alive) {
        break;
      }
    }
    return answers;
  }

  Future<bool> _isLive(String id) async {
    final note = await _library.note(id);
    return note.fold(
      (_) => false,
      (found) => found.notebookId != trashNotebookId,
    );
  }
}
