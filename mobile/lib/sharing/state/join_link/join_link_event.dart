// Kanso — join_link_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'join_link_bloc.dart';

@freezed
sealed class JoinLinkEvent with _$JoinLinkEvent {
  /// Join was pressed with [link] typed or pasted.
  const factory JoinLinkEvent.submitted(String link) = JoinLinkSubmittedEvent;

  /// The person wants to join another note: back to an empty field.
  const factory JoinLinkEvent.restarted() = JoinLinkRestartedEvent;
}
