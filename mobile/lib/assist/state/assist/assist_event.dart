// Kanso — assist_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'assist_bloc.dart';

@freezed
sealed class AssistEvent with _$AssistEvent {
  /// The person chose [action] for [selection] of the note [noteId], with
  /// [before] the text just before it for context.
  const factory AssistEvent.requested({
    required String noteId,
    required AssistAction action,
    required String before,
    required String selection,
  }) = AssistRequestedEvent;

  /// The person asked for a summary of the note [noteId].
  const factory AssistEvent.summaryRequested(String noteId) =
      AssistSummaryRequestedEvent;

  /// The core told how the suggestion is going.
  const factory AssistEvent.updateReceived(AssistUpdate update) =
      AssistUpdateReceivedEvent;

  /// The core ended its updates.
  const factory AssistEvent.streamEnded() = AssistStreamEndedEvent;

  /// The person pressed Stop.
  const factory AssistEvent.stopped() = AssistStoppedEvent;

  /// The suggestion could not be applied because the text changed.
  const factory AssistEvent.changeNoticed() = AssistChangeNoticedEvent;

  /// The person threw the suggestion away, or went back to the actions.
  const factory AssistEvent.discarded() = AssistDiscardedEvent;
}
