// Kanso — file_transcript.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why a recording could not be turned into text.
enum FileTranscriptProblem {
  /// This system can't recognize a file.
  unsupported,

  /// Only a server could, and that wasn't allowed.
  notOnDevice,

  /// Speech recognition isn't allowed.
  denied,

  /// The file couldn't be opened.
  unreadable,

  /// Anything else.
  failed,
}

/// What recognizing a recording gave.
class FileTranscript {
  const FileTranscript({
    required this.text,
    required this.onDevice,
    this.problem,
  });

  /// The words; empty if [problem] is set or nothing was heard.
  final String text;

  /// Whether the words were made on the phone.
  final bool onDevice;

  /// Null when the recording was recognized, even if it held no words.
  final FileTranscriptProblem? problem;
}
