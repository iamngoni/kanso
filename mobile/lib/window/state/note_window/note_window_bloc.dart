// Kanso — note_window_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/models/repos/abs/session_repository.dart';
import '../../../core/utils/debounce.dart';
import '../../../core/utils/note_liveness.dart';
import '../../models/data/note_window_status.dart';

part 'note_window_bloc.freezed.dart';
part 'note_window_event.dart';
part 'note_window_state.dart';

/// How long the writes of a burst may be apart for the note to be looked at
/// again once, after the last.
const Duration noteWindowDebounce = Duration(milliseconds: 100);

/// What a note's own window knows of its note whatever the editor is doing:
/// its title and whether it is locked (the window's name), whether it was
/// moved to the trash (the banner), and whether it was deleted for good
/// (the window closes).
///
/// It looks at the note again whenever the library writes it, in whichever
/// window, and whenever the lock session opens or closes.
@injectable
class NoteWindowBloc extends Bloc<NoteWindowEvent, NoteWindowState> {
  NoteWindowBloc(this._library, this._session)
    : super(const NoteWindowState()) {
    on<NoteWindowStartedEvent>(_onStarted, transformer: restartable());
    on<NoteWindowLookedAtEvent>(
      (event, emit) => _read(emit),
      transformer: debounceRestartable<NoteWindowLookedAtEvent>(
        noteWindowDebounce,
      ),
    );
    on<NoteWindowRestoreRequestedEvent>(
      _onRestoreRequested,
      transformer: droppable(),
    );
  }

  final LibraryRepository _library;
  final SessionRepository _session;
  StreamSubscription<List<String>>? _changes;
  StreamSubscription<LockEvent>? _locks;
  int _reading = 0;

  Future<void> _onStarted(
    NoteWindowStartedEvent event,
    Emitter<NoteWindowState> emit,
  ) async {
    emit(NoteWindowState(noteId: event.noteId));
    await _changes?.cancel();
    await _locks?.cancel();
    _changes = _library.changes().listen((ids) {
      if (ids.contains(event.noteId)) {
        add(const NoteWindowEvent.lookedAt());
      }
    });
    _locks = _session.lockEvents().listen(
      (_) => add(const NoteWindowEvent.lookedAt()),
      onError: (Object _) {},
    );
    await _read(emit);
  }

  /// Reads the note. A reading that a later one overtook is dropped.
  Future<void> _read(Emitter<NoteWindowState> emit) async {
    final reading = ++_reading;
    final result = await _library.note(state.noteId);
    if (reading != _reading || emit.isDone) {
      return;
    }
    result.fold(
      (error) {
        if (error is NotFoundError) {
          emit(state.copyWith(status: NoteWindowStatus.gone));
        }
        // Any other failure leaves the window as it was: the note is shown
        // by the note screen, which says what is wrong.
      },
      (note) => emit(
        state.copyWith(
          status: NoteWindowStatus.ready,
          title: note.title,
          locked: note.lock is! LockState_Plain,
          trashed: note.notebookId == trashNotebookId,
        ),
      ),
    );
  }

  /// Takes the note out of the trash, into the first notebook there is.
  Future<void> _onRestoreRequested(
    NoteWindowRestoreRequestedEvent event,
    Emitter<NoteWindowState> emit,
  ) async {
    final books = await _library.notebooks();
    final target = books
        .map<String?>(
          (rows) => rows
              .where((row) => row.id != trashNotebookId)
              .map((row) => row.id)
              .firstOrNull,
        )
        .getOrElse(() => null);
    if (target == null) {
      return;
    }
    await _library.change(<String>[
      state.noteId,
    ], NoteChange.move(notebookId: target));
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    await _locks?.cancel();
    return super.close();
  }
}
