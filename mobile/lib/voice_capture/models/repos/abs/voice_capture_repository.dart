// Kanso — voice_capture_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/voice_capture_types.dart';
import '../../../../core/models/data/application_error.dart';

/// Voice notes that land in the Inbox, the recordings left by the lock
/// screen, the Action Button and the Android tile, and tidying a voice note
/// with the person's own AI (issue #193). The words are the Rust core's.
abstract interface class VoiceCaptureRepository {
  /// Saves a voice note as a note in the Inbox. No note id in the answer
  /// means nothing was heard and nothing was saved. A locked Inbox is a
  /// `LockedError`.
  Future<Either<ApplicationError, VoiceCaptureSaved>> save(
    VoiceCaptureInput input,
  );

  /// The recordings waiting to be transcribed, oldest first.
  Future<Either<ApplicationError, List<VoiceDrop>>> drops();

  /// Saves recording [id] with the words the phone's recognizer made, and
  /// removes the recording.
  Future<Either<ApplicationError, VoiceCaptureSaved>> saveDrop(
    String id,
    String transcript, {
    required bool onDevice,
  });

  /// Saves recording [id] as a note with its audio and a line saying it was
  /// not turned into text.
  Future<Either<ApplicationError, VoiceCaptureSaved>> saveDropAudioOnly(
    String id,
  );

  /// Removes recording [id] without saving it.
  Future<Either<ApplicationError, Unit>> discardDrop(String id);

  /// Whether note [id] is a voice note, whether it was tidied, and whether
  /// Tidy may be offered. Null for a note that is not a voice note.
  Future<Either<ApplicationError, VoiceNoteInfo?>> noteInfo(String id);

  /// Tidies voice note [id]: a title, short paragraphs and the decisions and
  /// tasks the recording backs with a quote. One write at the end; a failure
  /// changes nothing and says why.
  Future<Either<ApplicationError, TidyOutcome>> tidy(String id);
}
