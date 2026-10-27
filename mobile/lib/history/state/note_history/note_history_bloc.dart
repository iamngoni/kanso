// Kanso — note_history_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/models/repos/abs/system_share_repository.dart';
import '../../models/data/history_pane.dart';
import '../../models/data/history_problem.dart';
import '../../models/data/history_status.dart';
import '../../models/repos/abs/history_repository.dart';

part 'note_history_bloc.freezed.dart';
part 'note_history_event.dart';
part 'note_history_state.dart';

/// A note's history: its saved versions, what changed since the one
/// selected, and bringing one back.
///
/// A locked note's versions are sealed: while the note is closed the
/// repository refuses and this shows [HistoryStatus.locked] with no version
/// in it. Restoring is checked against the revision the note had when the
/// history was read, so a note that moved on is not overwritten blind: the
/// history is read again and the person chooses again. Restoring keeps the
/// text the note had as a version of its own.
@injectable
class NoteHistoryBloc extends Bloc<NoteHistoryEvent, NoteHistoryState> {
  NoteHistoryBloc(this._history, this._library, this._system)
    : super(const NoteHistoryState()) {
    on<NoteHistoryStartedEvent>(_onStarted, transformer: restartable());
    on<NoteHistoryRetriedEvent>(_onRetried, transformer: restartable());
    on<NoteHistoryVersionSelectedEvent>(
      _onVersionSelected,
      transformer: restartable(),
    );
    on<NoteHistoryPaneChangedEvent>(_onPaneChanged);
    on<NoteHistorySectionCopiedEvent>(
      _onSectionCopied,
      transformer: sequential(),
    );
    on<NoteHistoryRestoreRequestedEvent>(
      _onRestoreRequested,
      transformer: droppable(),
    );
  }

  final HistoryRepository _history;
  final LibraryRepository _library;
  final SystemShareRepository _system;

  Future<void> _onStarted(
    NoteHistoryStartedEvent event,
    Emitter<NoteHistoryState> emit,
  ) async {
    emit(NoteHistoryState(noteId: event.noteId));
    await _read(emit);
  }

  Future<void> _onRetried(
    NoteHistoryRetriedEvent event,
    Emitter<NoteHistoryState> emit,
  ) async {
    emit(NoteHistoryState(noteId: state.noteId));
    await _read(emit);
  }

  /// Reads the note's revision and then its history, so a save in between
  /// leaves the revision older than what is shown (a restore is then
  /// refused, not wrong), and selects the newest version.
  Future<void> _read(Emitter<NoteHistoryState> emit) async {
    final id = state.noteId;
    final note = await _library.note(id);
    final base = note.fold<String?>((_) => null, (found) => found.rev);
    final read = await _history.history(id);
    switch (read) {
      case Left(value: final error):
        emit(
          state.copyWith(
            status: switch (error) {
              LockedError() => HistoryStatus.locked,
              NotFoundError() => HistoryStatus.notFound,
              _ => HistoryStatus.failed,
            },
          ),
        );
      case Right(value: final history):
        final first = history.versions.firstOrNull;
        emit(
          state.copyWith(
            status: HistoryStatus.ready,
            history: history,
            currentRev: base,
            selectedRev: first?.rev,
            diffStatus: first == null ? LoadStatus.ready : LoadStatus.loading,
            hunks: const <DiffHunk>[],
          ),
        );
        if (first != null) {
          await _diff(first.rev, emit);
        }
    }
  }

  Future<void> _onVersionSelected(
    NoteHistoryVersionSelectedEvent event,
    Emitter<NoteHistoryState> emit,
  ) async {
    emit(
      state.copyWith(
        selectedRev: event.rev,
        diffStatus: LoadStatus.loading,
        hunks: const <DiffHunk>[],
        copiedSection: null,
        problem: null,
      ),
    );
    await _diff(event.rev, emit);
  }

  Future<void> _diff(String rev, Emitter<NoteHistoryState> emit) async {
    final found = await _history.diff(state.noteId, rev);
    emit(
      found.fold(
        (error) => switch (error) {
          LockedError() => state.copyWith(status: HistoryStatus.locked),
          NotFoundError() => state.copyWith(
            diffStatus: LoadStatus.failed,
            problem: HistoryProblem.versionGone,
          ),
          _ => state.copyWith(
            diffStatus: LoadStatus.failed,
            problem: HistoryProblem.diffFailed,
          ),
        },
        (hunks) => state.copyWith(hunks: hunks, diffStatus: LoadStatus.ready),
      ),
    );
  }

  void _onPaneChanged(
    NoteHistoryPaneChangedEvent event,
    Emitter<NoteHistoryState> emit,
  ) {
    emit(state.copyWith(pane: event.pane));
  }

  Future<void> _onSectionCopied(
    NoteHistorySectionCopiedEvent event,
    Emitter<NoteHistoryState> emit,
  ) async {
    final hunk = state.hunks.elementAtOrNull(event.index);
    if (hunk == null) {
      return;
    }
    final copied = await _system.copy(hunk.oldText);
    emit(state.copyWith(copiedSection: copied.isRight() ? event.index : null));
  }

  Future<void> _onRestoreRequested(
    NoteHistoryRestoreRequestedEvent event,
    Emitter<NoteHistoryState> emit,
  ) async {
    final rev = state.selectedRev;
    final base = state.currentRev;
    if (state.restoring || rev == null || base == null) {
      return;
    }
    emit(state.copyWith(restoring: true, problem: null));
    final restored = await _history.restore(
      id: state.noteId,
      rev: rev,
      baseRev: base,
    );
    switch (restored) {
      case Right():
        emit(state.copyWith(restoring: false, restored: true));
      case Left(value: LockedError()):
        emit(state.copyWith(restoring: false, status: HistoryStatus.locked));
      case Left(value: ConflictError()):
        emit(state.copyWith(restoring: false));
        await _read(emit);
        emit(state.copyWith(problem: HistoryProblem.changedMeanwhile));
      case Left(value: NotFoundError()):
        emit(
          state.copyWith(
            restoring: false,
            problem: HistoryProblem.versionGone,
          ),
        );
      case Left():
        emit(
          state.copyWith(
            restoring: false,
            problem: HistoryProblem.restoreFailed,
          ),
        );
    }
  }
}
