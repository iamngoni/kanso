// Kanso — ask_phase.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the conversation is doing now.
enum AskPhase {
  /// Waiting for a question.
  idle,

  /// The notes are being searched.
  searching,

  /// The model has the notes and has not yet written.
  thinking,

  /// The answer is being written.
  answering,
}
