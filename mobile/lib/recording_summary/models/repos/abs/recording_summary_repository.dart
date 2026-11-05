// Kanso — recording_summary_repository.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/recording_summary_types.dart';
import '../../../../core/models/data/application_error.dart';

/// A summary and action items for a recording, made by the person's own AI
/// from the transcript alone, and written into the note only once the
/// person has looked them over (issue #38). The words are the Rust core's.
abstract interface class RecordingSummaryRepository {
  /// Whether Summarise Recording is offered for note [id], and what to say
  /// before anything is sent. Sends nothing.
  Future<Either<ApplicationError, SummaryOffer>> offer(String id);

  /// Asks the AI for a summary of note [id]'s recording. Nothing is written.
  /// A failure says why; [cancel] stops a request in flight.
  Future<Either<ApplicationError, SummaryDraftView>> summarize(String id);

  /// Stops the request [summarize] is waiting on, if any.
  void cancel();

  /// Writes [draft] above the transcript of note [id], with the action items
  /// at the positions in [ticked]. One change, with an Undo ticket.
  Future<Either<ApplicationError, SummaryApplied>> apply(
    String id,
    SummaryDraftView draft,
    List<int> ticked,
  );
}
