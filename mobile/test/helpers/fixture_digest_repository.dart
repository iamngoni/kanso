// Kanso — fixture_digest_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/digest_repository.dart';

/// The core's side of the open loops, the brief and the review for tests:
/// no loops, an empty brief, and the notifications in [planned].
class FixtureDigestRepository implements DigestRepository {
  FixtureDigestRepository({this.planned = const <PlannedNotice>[]});

  /// What the plan for the next week holds.
  List<PlannedNotice> planned;

  /// The settings, as set.
  BriefPrefs saved = const BriefPrefs(
    loops: true,
    brief: true,
    briefHour: 8,
    briefMinute: 0,
    briefWeekdaysOnly: false,
    review: true,
    reviewDay: 6,
    reviewHour: 18,
    reviewMinute: 0,
    notifyBrief: true,
    notifyReview: true,
    markDoneInNote: false,
  );

  /// The open loops.
  List<OpenLoopItem> loops = <OpenLoopItem>[];

  /// What "Find more open loops" finds, and what was kept.
  LoopScan scan = const LoopScan(proposals: <LoopProposal>[]);
  final List<List<LoopProposal>> kept = <List<LoopProposal>>[];

  /// What the summaries say.
  AiSummary summary = const AiSummary(text: 'A calm day.');

  /// How many AI requests were stopped.
  int stops = 0;

  /// The loops closed, as they were closed.
  final List<(String, LoopAction)> closed = <(String, LoopAction)>[];

  @override
  Future<Either<ApplicationError, OpenLoopsBoard>> openLoops() async =>
      Right<ApplicationError, OpenLoopsBoard>(
        OpenLoopsBoard(items: loops, total: loops.length, enabled: true),
      );

  @override
  Future<Either<ApplicationError, bool>> closeLoop({
    required OpenLoopItem item,
    required LoopAction action,
  }) async {
    closed.add((item.key, action));
    return const Right<ApplicationError, bool>(true);
  }

  /// The brief, when a test wants one that isn't empty.
  BriefView? briefView;

  @override
  Future<Either<ApplicationError, BriefView>> brief() async {
    final set = briefView;
    if (set != null) {
      return Right<ApplicationError, BriefView>(set);
    }
    return const Right<ApplicationError, BriefView>(
      BriefView(
        dateLabel: 'Monday 2 November',
        due: <TaskItem>[],
        loops: <OpenLoopItem>[],
        loopsTotal: 0,
        laterToday: <BriefReminder>[],
        relevant: <BriefNote>[],
        wroteYesterday: 0,
        finishedYesterday: 0,
        isEmpty: true,
      ),
    );
  }

  @override
  Future<Either<ApplicationError, ReviewView>> review({String? from}) async =>
      Right<ApplicationError, ReviewView>(
        ReviewView(
          from: from ?? '2026-11-02',
          weekLabel: '2026-W45',
          rangeLabel: '2 Nov to 8 Nov',
          countsText: '0 new, 0 changed',
          written: const <ReviewWritten>[],
          finished: const <ReviewFinished>[],
          stillOpen: const <OpenLoopItem>[],
          overdue: const <TaskItem>[],
          themes: const <ReviewTheme>[],
          next: const <TaskItem>[],
          isThisWeek: from == null,
        ),
      );

  @override
  Future<Either<ApplicationError, ReviewSaved>> saveReview({
    required String from,
    required bool replace,
  }) async => const Right<ApplicationError, ReviewSaved>(
    ReviewSaved(noteId: 'note:weekly-review-2026-w45', exists: false),
  );

  @override
  Future<Either<ApplicationError, BriefPrefs>> prefs() async =>
      Right<ApplicationError, BriefPrefs>(saved);

  @override
  Future<Either<ApplicationError, BriefPrefs>> setPrefs(
    BriefPrefs prefs,
  ) async {
    saved = prefs;
    return Right<ApplicationError, BriefPrefs>(prefs);
  }

  @override
  Future<Either<ApplicationError, List<PlannedNotice>>> plan({
    required bool canNotify,
  }) async => Right<ApplicationError, List<PlannedNotice>>(
    canNotify ? planned : const <PlannedNotice>[],
  );

  @override
  Future<Either<ApplicationError, LoopScan>> findLoops() async =>
      Right<ApplicationError, LoopScan>(scan);

  @override
  Future<Either<ApplicationError, int>> keepLoops(
    List<LoopProposal> picks,
  ) async {
    kept.add(picks);
    return Right<ApplicationError, int>(picks.length);
  }

  @override
  Future<Either<ApplicationError, AiSummary>> summariseBrief() async =>
      Right<ApplicationError, AiSummary>(summary);

  @override
  Future<Either<ApplicationError, AiSummary>> summariseReview({
    String? from,
  }) async => Right<ApplicationError, AiSummary>(summary);

  @override
  void stopAi() => stops++;
}
