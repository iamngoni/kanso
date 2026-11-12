// Kanso — agent_session.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'agent_task.dart';

part 'agent_session.freezed.dart';

/// The colour of a session's status dot.
enum AgentSessionDot {
  /// Blue and pulsing: the AI is working.
  working,

  /// Amber: a question or changes to review.
  needsYou,

  /// Grey.
  idle,

  /// Red.
  failed;

  /// The dot the core calls [word]; an unknown word is [idle].
  static AgentSessionDot of(String word) => switch (word) {
    'working' => working,
    'needs_you' => needsYou,
    'failed' => failed,
    _ => idle,
  };
}

/// One agent task as a session (issue #375): a row in the list, and the head
/// of the session screen.
@freezed
abstract class AgentSession with _$AgentSession {
  const factory AgentSession({
    /// The task's id for this launch: Review, Stop and Apply take it.
    required int id,

    /// The session's own id, which a restart keeps.
    required String key,

    /// The goal cut to one line.
    required String title,
    required AgentTaskPhase phase,
    @Default('Kanso AI') String agent,

    /// The question, the error or the number of changes, by [phase].
    @Default('') String detail,
    @Default(AgentSessionDot.idle) AgentSessionDot dot,

    /// New items since the session was last opened.
    @Default(false) bool unread,
    @Default(0) int lastActivityMs,
    @Default(0) int notesRead,
    @Default(0) int changes,
    @Default(false) bool canStop,
    @Default(false) bool canReview,

    /// Whether the composer can send now.
    @Default(false) bool canFollowUp,
  }) = _AgentSession;

  const AgentSession._();

  /// Whether the AI is working, or waiting to.
  bool get working =>
      phase == AgentTaskPhase.running || phase == AgentTaskPhase.waiting;

  /// Whether the session has ended and can be dismissed.
  bool get finished => !working && phase != AgentTaskPhase.asking;
}

/// What a tool call does, as the agent says.
enum AgentToolKind {
  read,
  edit,
  delete,
  move,
  search,
  execute,
  think,
  fetch,
  switchMode,
  other;

  static AgentToolKind of(String word) => switch (word) {
    'read' => read,
    'edit' => edit,
    'delete' => delete,
    'move' => move,
    'search' => search,
    'execute' => execute,
    'think' => think,
    'fetch' => fetch,
    'switch_mode' => switchMode,
    _ => other,
  };
}

/// How a tool call went.
enum AgentToolStatus {
  pending,
  inProgress,
  completed,
  failed;

  static AgentToolStatus of(String word) => switch (word) {
    'pending' => pending,
    'completed' => completed,
    'failed' => failed,
    _ => inProgress,
  };

  /// Whether the call is still going.
  bool get running => this == pending || this == inProgress;
}

/// What the person answered when the agent asked for a tool.
enum AgentPermissionAnswer {
  allowedOnce,
  allowedForSession,
  allowedAlways,
  rejected;

  static AgentPermissionAnswer of(String word) => switch (word) {
    'allowed_for_session' => allowedForSession,
    'allowed_always' => allowedAlways,
    'rejected' => rejected,
    _ => allowedOnce,
  };
}

/// Where a round's proposal stands.
enum AgentRoundState {
  toReview,
  applied,
  undone,
  discarded,
  lost;

  static AgentRoundState of(String word) => switch (word) {
    'applied' => applied,
    'undone' => undone,
    'discarded' => discarded,
    'lost' => lost,
    _ => toReview,
  };
}

/// One thing in a session's transcript.
@freezed
sealed class AgentTranscriptItem with _$AgentTranscriptItem {
  /// The person's goal or a follow-up.
  const factory AgentTranscriptItem.user(String text) = AgentUserItem;

  /// What the agent said, as Markdown.
  const factory AgentTranscriptItem.agent(String text) = AgentSaidItem;

  /// What the agent thought, collapsed until opened.
  const factory AgentTranscriptItem.thought({
    required String text,
    @Default(0) int seconds,
  }) = AgentThoughtItem;

  /// A tool the agent called.
  const factory AgentTranscriptItem.tool({
    required String title,
    required AgentToolKind kind,
    required AgentToolStatus status,
  }) = AgentToolItem;

  /// What the person answered when the agent asked for a tool.
  const factory AgentTranscriptItem.permission({
    required String title,
    required AgentPermissionAnswer answer,
  }) = AgentPermissionItem;

  /// A round's proposal: "3 new notes, 1 edit".
  const factory AgentTranscriptItem.proposed({
    required int round,
    required String counts,
    required AgentRoundState state,
  }) = AgentProposedItem;

  /// A muted line: "Stopped", "Hit the 10-minute limit".
  const factory AgentTranscriptItem.notice(String text) = AgentNoticeItem;
}

/// A session and everything said in it.
@freezed
abstract class AgentTranscript with _$AgentTranscript {
  const factory AgentTranscript({
    required AgentSession session,

    /// Moves with every change.
    @Default(0) int version,
    @Default(<AgentTranscriptItem>[]) List<AgentTranscriptItem> items,
  }) = _AgentTranscript;
}
