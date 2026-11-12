// Kanso — agent_session_state.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'agent_session_bloc.dart';

@freezed
abstract class AgentSessionState with _$AgentSessionState {
  const factory AgentSessionState({
    @Default(LoadStatus.loading) LoadStatus status,
    AgentTranscript? transcript,

    /// A message is on its way.
    @Default(false) bool sending,

    /// The session is no longer listed.
    @Default(false) bool gone,

    /// Why the last send or undo didn't happen, as the core words it.
    String? trouble,
  }) = _AgentSessionState;
}
