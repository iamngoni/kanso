// Kanso — notebook_sharing_bloc.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/repos/abs/shared_notebooks_repository.dart';
import '../../../core/models/repos/abs/shared_notes_repository.dart';

part 'notebook_sharing_bloc.freezed.dart';

/// What the note list of a notebook shows of its sharing: who last edited
/// each note ("Edited by Dan · 2 min ago") and whether the notebook is up to
/// date ("Offline · 4 changes waiting"). Nothing for a notebook that isn't
/// shared. Read again whenever a sync or the notebook's room says the
/// notebook changed.
@injectable
class NotebookSharingBloc
    extends Bloc<NotebookSharingEvent, NotebookSharingState> {
  NotebookSharingBloc(this._books, this._shared)
    : super(const NotebookSharingState()) {
    on<NotebookSharingStartedEvent>(_onStarted, transformer: restartable());
    on<NotebookSharingRefreshedEvent>(
      (event, emit) => _read(emit),
      transformer: restartable(),
    );
  }

  final SharedNotebooksRepository _books;
  final SharedNotesRepository _shared;

  Future<void> _onStarted(
    NotebookSharingStartedEvent event,
    Emitter<NotebookSharingState> emit,
  ) async {
    emit(NotebookSharingState(notebookId: event.notebookId));
    await _read(emit);
    await emit.onEach<SharedEvent>(
      _shared.events(),
      onData: (news) {
        if (news.notebooks.contains(state.notebookId) ||
            news.bookNotices.isNotEmpty) {
          add(const NotebookSharingEvent.refreshed());
        }
      },
    );
  }

  Future<void> _read(Emitter<NotebookSharingState> emit) async {
    final id = state.notebookId;
    if (id.isEmpty) {
      return;
    }
    final status = await _books.status(id);
    final shared = status.fold((_) => null, (status) => status);
    if (shared == null) {
      emit(NotebookSharingState(notebookId: id));
      return;
    }
    final edits = await _books.edits(id);
    emit(
      state.copyWith(
        status: shared,
        edits: edits.fold(
          (_) => state.edits,
          (rows) => <String, NoteEditRow>{
            for (final row in rows) row.noteId: row,
          },
        ),
      ),
    );
  }
}

@freezed
sealed class NotebookSharingEvent with _$NotebookSharingEvent {
  /// The list shows the notebook [notebookId].
  const factory NotebookSharingEvent.started(String notebookId) =
      NotebookSharingStartedEvent;

  const factory NotebookSharingEvent.refreshed() =
      NotebookSharingRefreshedEvent;
}

@freezed
abstract class NotebookSharingState with _$NotebookSharingState {
  const factory NotebookSharingState({
    @Default('') String notebookId,

    /// Null while the notebook isn't shared (or isn't read yet).
    BookSyncStatus? status,

    /// Who last edited each note, by note ID.
    @Default(<String, NoteEditRow>{}) Map<String, NoteEditRow> edits,
  }) = _NotebookSharingState;
}
