// Kanso — shared_note_status_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/shared_notes_repository.dart';

part 'shared_note_status_bloc.freezed.dart';
part 'shared_note_status_event.dart';
part 'shared_note_status_state.dart';

/// What the page of an open shared note says of its sharing, as the
/// desktop's editor footer and problems banner do: the role, whether the
/// room is live, a private copy once sharing ended, how near the note is to
/// its size limit, and what couldn't be done for it.
///
/// It is read again whenever the room or a sync of shared notes says the
/// note changed. A note that isn't shared has no status.
@injectable
class SharedNoteStatusBloc
    extends Bloc<SharedNoteStatusEvent, SharedNoteStatusState> {
  SharedNoteStatusBloc(this._shared) : super(const SharedNoteStatusState()) {
    on<SharedNoteStatusEvent>(
      (event, emit) => switch (event) {
        SharedNoteStatusStartedEvent(:final noteId) => _onStarted(noteId, emit),
        SharedNoteStatusRefreshedEvent() => _load(emit),
        SharedNoteStatusProblemsDismissedEvent() => _onDismissed(emit),
      },
      transformer: sequential(),
    );
  }

  final SharedNotesRepository _shared;
  StreamSubscription<SharedEvent>? _events;

  Future<void> _onStarted(
    String noteId,
    Emitter<SharedNoteStatusState> emit,
  ) async {
    emit(SharedNoteStatusState(noteId: noteId));
    await _events?.cancel();
    _events = _shared.events().listen((event) {
      if (event.liveNotes.contains(noteId) || event.noteIds.contains(noteId)) {
        add(const SharedNoteStatusEvent.refreshed());
      }
    });
    await _load(emit);
  }

  Future<void> _load(Emitter<SharedNoteStatusState> emit) async {
    final noteId = state.noteId;
    if (noteId.isEmpty) {
      return;
    }
    final read = await _shared.liveStatus(noteId);
    final beat = state.beat + 1;
    read.fold(
      (error) => emit(state.copyWith(error: error, beat: beat)),
      (status) => emit(state.copyWith(status: status, error: null, beat: beat)),
    );
  }

  Future<void> _onDismissed(Emitter<SharedNoteStatusState> emit) async {
    final done = await _shared.dismissProblems(state.noteId);
    done.fold((error) => emit(state.copyWith(error: error)), (_) {});
    await _load(emit);
  }

  @override
  Future<void> close() async {
    await _events?.cancel();
    return super.close();
  }
}
