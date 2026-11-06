// Kanso — digest_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/overview_types.dart';
import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The Rust core's open loops and Overview (Today and This week).
///
/// Locked notes and notes in locked notebooks are in none of them.
abstract interface class DigestRepository {
  /// The open loops across the notes, newest notes first.
  Future<Either<ApplicationError, OpenLoopsBoard>> openLoops();

  /// Closes a loop; false when it is gone from its note already.
  Future<Either<ApplicationError, bool>> closeLoop({
    required OpenLoopItem item,
    required LoopAction action,
  });

  /// Closes the loop named [key] of [noteId] as done; false when it is gone
  /// from its note already.
  Future<Either<ApplicationError, bool>> doneLoop({
    required String noteId,
    required String key,
    required int line,
  });

  /// The last Today kept, when it is today's: the first frame, at once.
  OverviewToday? overviewSnapshot();

  /// Today, worked out now.
  Future<Either<ApplicationError, OverviewToday>> overviewToday();

  /// The week holding [from] (`YYYY-MM-DD`), or this week.
  Future<Either<ApplicationError, OverviewWeek>> overviewWeek({String? from});

  /// Keeps the latest selected system-calendar events on this device.
  Future<Either<ApplicationError, void>> putCalendarEvents(String eventsJson);

  /// Creates a Journal note linked to one system-calendar event.
  Future<Either<ApplicationError, String>> createMeetingNote({
    required String eventJson,
    required String context,
  });

  /// Links an existing note to one system-calendar event occurrence.
  Future<Either<ApplicationError, void>> linkMeetingNote({
    required String noteId,
    required String eventJson,
  });

  /// "Not now" on Pick up: hidden until the note is written in again.
  Future<Either<ApplicationError, void>> hidePlace(String noteId);

  /// "Not yet" on Waiting on others: asked again in three days.
  Future<Either<ApplicationError, void>> snoozeWaiting(String key);

  /// "Not useful" on the note from the past.
  Future<Either<ApplicationError, void>> dismissPast(String noteId);

  /// The choices of "Revisit in…", with the day each means today.
  List<RevisitOption> revisitOptions();

  /// Brings [noteId] back on Overview on [choice]'s day; the day, as
  /// `YYYY-MM-DD`. Setting it is not an edit.
  Future<Either<ApplicationError, String>> setRevisit({
    required String noteId,
    required RevisitChoice choice,
  });

  /// Takes [noteId] off "To revisit".
  Future<Either<ApplicationError, void>> clearRevisit(String noteId);

  /// Typing in a note, for Pick up: the caret and the characters typed since
  /// the last call.
  Future<Either<ApplicationError, void>> noteWriting({
    required String noteId,
    required int line,
    required int column,
    required int typedChars,
    required String body,
  });

  /// Saves the review of the week from [from] as a Journal note.
  Future<Either<ApplicationError, ReviewSaved>> saveReview({
    required String from,
    required bool replace,
  });

  /// The brief and review settings.
  Future<Either<ApplicationError, BriefPrefs>> prefs();

  /// Saves the settings; what was kept.
  Future<Either<ApplicationError, BriefPrefs>> setPrefs(BriefPrefs prefs);

  /// The notifications to hand the system for the next week.
  Future<Either<ApplicationError, List<PlannedNotice>>> plan({
    required bool canNotify,
  });

  /// Reads the recent notes with the model for open loops the rules missed.
  /// Slow (a network request): it runs off the UI isolate, and
  /// [stopAi] stops it. Nothing is saved until [keepLoops].
  Future<Either<ApplicationError, LoopScan>> findLoops();

  /// Keeps the proposals the person chose; how many were added.
  Future<Either<ApplicationError, int>> keepLoops(List<LoopProposal> picks);

  /// A short paragraph over Today, or over the week of [from] when [week].
  Future<Either<ApplicationError, AiSummary>> summariseOverview({
    required bool week,
    String? from,
  });

  /// Stops the AI request in flight, if any.
  void stopAi();
}
