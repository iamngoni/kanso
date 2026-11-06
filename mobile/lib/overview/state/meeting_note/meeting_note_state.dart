// Kanso — meeting_note_state.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'meeting_note_state.freezed.dart';

@freezed
abstract class MeetingNoteState with _$MeetingNoteState {
  const factory MeetingNoteState({
    @Default(false) bool working,
    String? createdNoteId,
    String? error,
  }) = _MeetingNoteState;
}
