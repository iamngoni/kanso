// Kanso — meeting_note_bloc.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/models/repos/abs/digest_repository.dart';
import 'meeting_note_event.dart';
import 'meeting_note_state.dart';

class MeetingNoteBloc extends Bloc<MeetingNoteEvent, MeetingNoteState> {
  MeetingNoteBloc(this._digest) : super(const MeetingNoteState()) {
    on<MeetingNoteCreateRequested>(_create);
  }

  final DigestRepository _digest;

  Future<void> _create(
    MeetingNoteCreateRequested event,
    Emitter<MeetingNoteState> emit,
  ) async {
    if (state.working) return;
    final row = event.row;
    if (row.eventJson.isEmpty) {
      emit(state.copyWith(working: false, error: 'invalid-event'));
      return;
    }
    emit(const MeetingNoteState(working: true));
    final result = await _digest.createMeetingNote(
      eventJson: '[${row.eventJson}]',
      context: row.context,
    );
    result.fold(
      (error) => emit(state.copyWith(working: false, error: error.message)),
      (id) => emit(state.copyWith(working: false, createdNoteId: id)),
    );
  }
}
