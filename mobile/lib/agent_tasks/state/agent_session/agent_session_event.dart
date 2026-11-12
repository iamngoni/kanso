// Kanso — agent_session_event.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'agent_session_bloc.dart';

@freezed
sealed class AgentSessionEvent with _$AgentSessionEvent {
  /// The screen opened session [key].
  const factory AgentSessionEvent.started(String key) =
      AgentSessionStartedEvent;

  /// The timer asks what has been said.
  const factory AgentSessionEvent.polled() = AgentSessionPolledEvent;

  /// The person sent [text] from the composer.
  const factory AgentSessionEvent.followUpSent(String text) =
      AgentSessionFollowUpSentEvent;

  /// Stop this turn.
  const factory AgentSessionEvent.stopped() = AgentSessionStoppedEvent;

  /// End the session.
  const factory AgentSessionEvent.ended() = AgentSessionEndedEvent;

  /// Take the finished session off the list.
  const factory AgentSessionEvent.dismissed() = AgentSessionDismissedEvent;

  /// Take back the last apply.
  const factory AgentSessionEvent.undone() = AgentSessionUndoneEvent;

  /// The person saw the trouble line.
  const factory AgentSessionEvent.troubleCleared() =
      AgentSessionTroubleClearedEvent;
}
