// Kanso — ask_outcome.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How a question in the conversation ended, or that it has not yet.
enum AskOutcome {
  /// Being answered.
  pending,

  /// The answer is complete.
  answered,

  /// No note matches the question.
  nothingMatches,

  /// The model stopped before it finished.
  cutOff,

  /// The person stopped it; what was written is kept.
  stopped,

  /// It could not be answered.
  failed,
}
