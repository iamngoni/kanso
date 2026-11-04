// Kanso — feedback_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../data/feedback_draft.dart';
import '../../data/feedback_entry.dart';
import '../../data/feedback_result.dart';

/// Feedback reports: the outbox on the phone and what the server says about
/// them. Nothing here reads a note.
abstract interface class FeedbackRepository {
  /// The exact JSON a send would carry, or why there is none yet.
  /// [screen] is one of the fixed screens (`settings`, `editor`…).
  Future<FeedbackPayload> preview(FeedbackDraft draft, {String screen});

  /// Saves the report on the phone, then tries to send it.
  Future<FeedbackSendResult> send(FeedbackDraft draft, {String screen});

  /// The reports, newest first. With [refresh] the server is asked first;
  /// offline, what the phone last knew is returned. Null if the library
  /// could not be read.
  Future<List<FeedbackEntry>?> entries({bool refresh});

  /// Whether the launch refresh is due (at most every 6 hours).
  bool get refreshDue;

  /// Deletes a report here and on the server.
  Future<bool> delete(String id);

  /// Sends what is waiting (app resume, connectivity returned).
  Future<void> flush();

  /// The released reports were looked at.
  Future<void> markSeen();
}
