// Kanso — comments_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/shared_notes_repository.dart';

part 'comments_bloc.freezed.dart';
part 'comments_event.dart';
part 'comments_state.dart';

/// The comments on an open shared note, the desktop's Comments tab: those
/// on words still in the text in text order, then the ones whose words were
/// deleted. Anyone who may comment resolves and reopens them; this
/// account's own can be edited and deleted.
///
/// The list is read again whenever a sync of shared notes says the note's
/// comments or sharing changed, and after each change made here. A note
/// that isn't shared has none, and says so.
@injectable
class CommentsBloc extends Bloc<CommentsEvent, CommentsState> {
  CommentsBloc(this._shared) : super(const CommentsState()) {
    on<CommentsEvent>(
      (event, emit) => switch (event) {
        CommentsStartedEvent(:final noteId) => _onStarted(noteId, emit),
        CommentsRefreshedEvent() => _load(emit),
        CommentsResolvedEvent(:final commentId, :final resolved) => _change(
          emit,
          () => _shared.resolveComment(
            state.noteId,
            commentId,
            resolved: resolved,
          ),
        ),
        CommentsEditedEvent(:final commentId, :final body) => _change(
          emit,
          () => _shared.editComment(state.noteId, commentId, body),
        ),
        CommentsDeletedEvent(:final commentId) => _change(
          emit,
          () => _shared.deleteComment(state.noteId, commentId),
        ),
        CommentsSelectedEvent(:final commentId) => _show(
          emit,
          state.copyWith(selecting: commentId),
        ),
        CommentsSelectionHandledEvent() => _show(
          emit,
          state.copyWith(selecting: null),
        ),
      },
      transformer: sequential(),
    );
  }

  final SharedNotesRepository _shared;
  StreamSubscription<SharedEvent>? _events;

  Future<void> _onStarted(String noteId, Emitter<CommentsState> emit) async {
    emit(CommentsState(noteId: noteId));
    await _events?.cancel();
    _events = _shared.events().listen((event) {
      if (event.commentNotes.contains(noteId) ||
          event.noteIds.contains(noteId)) {
        add(const CommentsEvent.refreshed());
      }
    });
    await _load(emit);
  }

  Future<void> _load(Emitter<CommentsState> emit) async {
    final noteId = state.noteId;
    if (noteId.isEmpty) {
      return;
    }
    final info = await _shared.info(noteId);
    final shared = info.fold((_) => null, (info) => info);
    if (shared == null) {
      emit(
        state.copyWith(
          status: info.isLeft() ? LoadStatus.failed : LoadStatus.ready,
          role: null,
          comments: const <CommentRow>[],
          band: '',
          error: info.fold((error) => error, (_) => null),
        ),
      );
      return;
    }
    final comments = await _shared.comments(noteId);
    final band = await _shared.commentsBand(noteId);
    emit(
      comments.fold(
        (error) => state.copyWith(status: LoadStatus.failed, error: error),
        (rows) => state.copyWith(
          status: LoadStatus.ready,
          role: shared.myRole,
          comments: rows,
          band: band.getOrElse(() => ''),
          error: null,
        ),
      ),
    );
  }

  /// Every handler is a future, so each event waits for the one before.
  Future<void> _show(Emitter<CommentsState> emit, CommentsState next) async =>
      emit(next);

  Future<void> _change(
    Emitter<CommentsState> emit,
    Future<Either<ApplicationError, Unit>> Function() change,
  ) async {
    final done = await change();
    await _load(emit);
    done.fold((error) => emit(state.copyWith(error: error)), (_) {});
  }

  @override
  Future<void> close() async {
    await _events?.cancel();
    return super.close();
  }
}
