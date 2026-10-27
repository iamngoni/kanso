// Kanso — shared_note_live_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/repos/abs/shared_notes_repository.dart';

part 'shared_note_live_bloc.freezed.dart';
part 'shared_note_live_event.dart';
part 'shared_note_live_state.dart';

/// The live room of the shared note that is open.
///
/// While the note is open and shared, its room is open: what other people
/// type is merged into the library within a moment (and so reaches the
/// editor through the library's changes), and what the person types goes out
/// the same way. Closing the bloc leaves the room, even when it is closed
/// while the room is still being opened. A note that is not shared has no
/// room, and this does nothing for it.
///
/// The room is closed when the app goes to the background; coming back to
/// the front is [SharedNoteLiveEvent.resumed]. Every event runs after the
/// one before it finished, so a resume never reopens the room of a note
/// that was left meanwhile.
@injectable
class SharedNoteLiveBloc
    extends Bloc<SharedNoteLiveEvent, SharedNoteLiveState> {
  SharedNoteLiveBloc(this._shared) : super(const SharedNoteLiveState()) {
    on<SharedNoteLiveEvent>(
      (event, emit) => switch (event) {
        SharedNoteLiveOpenedEvent(:final noteId) => _onOpened(noteId, emit),
        SharedNoteLiveResumedEvent() => _enter(state.noteId, emit),
      },
      transformer: sequential(),
    );
  }

  final SharedNotesRepository _shared;

  /// The note whose room this holds open, if any.
  String? _room;

  Future<void> _onOpened(
    String noteId,
    Emitter<SharedNoteLiveState> emit,
  ) async {
    // The room of the note that was open before this one is left.
    if (_room != null && _room != noteId) {
      await _leave();
    }
    emit(SharedNoteLiveState(noteId: noteId));
    await _enter(noteId, emit);
  }

  Future<void> _enter(String noteId, Emitter<SharedNoteLiveState> emit) async {
    if (noteId.isEmpty || isClosed) {
      return;
    }
    final read = await _shared.info(noteId);
    final shared = read.fold((_) => false, (info) => info != null);
    if (!shared) {
      return;
    }
    final opened = await _shared.openLive(noteId, this);
    if (opened.isLeft()) {
      emit(state.copyWith(live: false));
      return;
    }
    _room = noteId;
    if (isClosed) {
      // Closed while the room was opening: nobody is left to leave it.
      await _leave();
      return;
    }
    emit(state.copyWith(live: true));
  }

  Future<void> _leave() async {
    final room = _room;
    _room = null;
    if (room != null) {
      await _shared.closeLive(room, this);
    }
  }

  @override
  Future<void> close() async {
    await super.close();
    await _leave();
  }
}
