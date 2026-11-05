// Kanso — recording_speakers_repository.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/recording_speakers_types.dart';
import '../../../../core/models/data/application_error.dart';

/// The names before the paragraphs of a recording's transcript ("Speaker 1",
/// "You", "Priya"), listed and changed (issue #38). The rules are the Rust
/// core's, the desktop's as well.
abstract interface class RecordingSpeakersRepository {
  /// The names in note [id]'s first recording's transcript, in the order
  /// they first appear. Empty when the note has none to rename.
  Future<Either<ApplicationError, List<SpeakerNameView>>> names(String id);

  /// Writes [changes] to those names, in one change with an Undo ticket.
  Future<Either<ApplicationError, SpeakersRenamed>> rename(
    String id,
    List<SpeakerChange> changes,
  );
}
