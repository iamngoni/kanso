// Kanso — voice_note_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where saving a voice note is.
enum VoiceNoteStatus {
  /// Nothing saved yet.
  idle,

  /// The note is being written.
  saving,

  /// The note is saved; see its id.
  saved,

  /// The note could not be saved.
  failed,
}
