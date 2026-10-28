// Kanso — invitations_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'invitations_bloc.dart';

@freezed
sealed class InvitationsEvent with _$InvitationsEvent {
  /// The app started: read the invitations, and read them again each time a
  /// sync of shared notes finds news.
  const factory InvitationsEvent.started() = InvitationsStartedEvent;

  /// The invitations were asked for again: the screen was opened, a sync
  /// found news, or the person pulled to refresh.
  const factory InvitationsEvent.refreshed() = InvitationsRefreshedEvent;

  /// The invitation [shareId] was accepted.
  const factory InvitationsEvent.accepted(String shareId) =
      InvitationsAcceptedEvent;

  /// The invitation [shareId] was declined.
  const factory InvitationsEvent.declined(String shareId) =
      InvitationsDeclinedEvent;
}
