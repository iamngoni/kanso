// Kanso — meeting_note_event.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/overview_types.dart';

part 'meeting_note_event.freezed.dart';

@freezed
sealed class MeetingNoteEvent with _$MeetingNoteEvent {
  const factory MeetingNoteEvent.create({required OverviewAgendaRow row}) =
      MeetingNoteCreateRequested;
}
