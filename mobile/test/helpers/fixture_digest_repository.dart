// Kanso — fixture_digest_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/overview_types.dart';
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
    openToOverview: true,
    quietDays: 5,
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

  /// Today, as a test sets it; an empty library's by default.
  OverviewToday today = const OverviewToday(
    day: '2026-11-02',
    greeting: 'Good morning',
    overline: 'MONDAY 2 NOVEMBER',
    lede: 'Nothing on today.',
    state: OverviewState.quiet,
    earlier: <OverviewAgendaRow>[],
    agenda: <OverviewAgendaRow>[],
    agendaMore: 0,
    taskTotal: 0,
    lists: <OverviewList>[],
    also: <OverviewLine>[],
    footer: '',
  );

  /// The snapshot, when a test wants one drawn first.
  OverviewToday? snapshot;

  /// The week, as a test sets it.
  OverviewWeek week = const OverviewWeek(
    from: '2026-11-02',
    rangeLabel: '2 \u2013 8 NOVEMBER',
    weekLabel: 'Week 45',
    isThisWeek: true,
    lede: 'A quiet week.',
    projects: <OverviewProject>[],
    finished: <OverviewFinished>[],
    finishedMore: 0,
    waiting: <OverviewListRow>[],
    stillOpen: <OverviewAgendaRow>[],
    stillOpenTotal: 0,
    ahead: <OverviewAheadDay>[],
    also: <OverviewLine>[],
    savedAs: '',
  );

  /// What was done on the page's rows.
  final List<String> done = <String>[];
  final List<String> hidden = <String>[];
  final List<(String, String)> meetingLinks = <(String, String)>[];
  final List<String> snoozed = <String>[];
  final List<String> dismissed = <String>[];
  final List<(String, RevisitChoice)> revisits = <(String, RevisitChoice)>[];
  final List<String> cleared = <String>[];
  final List<(String?, bool)> summarised = <(String?, bool)>[];

  @override
  Future<Either<ApplicationError, bool>> doneLoop({
    required String noteId,
    required String key,
    required int line,
  }) async {
    done.add('$noteId/$key');
    return const Right<ApplicationError, bool>(true);
  }

  @override
  OverviewToday? overviewSnapshot() => snapshot;

  @override
  Future<Either<ApplicationError, OverviewToday>> overviewToday() async =>
      Right<ApplicationError, OverviewToday>(today);

  @override
  Future<Either<ApplicationError, OverviewWeek>> overviewWeek({
    String? from,
  }) async => Right<ApplicationError, OverviewWeek>(
    from == null
        ? week
        : OverviewWeek(
            from: from,
            rangeLabel: week.rangeLabel,
            weekLabel: week.weekLabel,
            isThisWeek: false,
            lede: week.lede,
            projects: week.projects,
            finished: week.finished,
            finishedMore: week.finishedMore,
            waiting: week.waiting,
            stillOpen: week.stillOpen,
            stillOpenTotal: week.stillOpenTotal,
            ahead: week.ahead,
            also: week.also,
            savedAs: week.savedAs,
          ),
  );

  @override
  Future<Either<ApplicationError, void>> putCalendarEvents(
    String eventsJson,
  ) async => const Right<ApplicationError, void>(null);

  @override
  Future<Either<ApplicationError, String>> createMeetingNote({
    required String eventJson,
    required String context,
  }) async => const Right<ApplicationError, String>('note:meeting');

  @override
  Future<Either<ApplicationError, void>> linkMeetingNote({
    required String noteId,
    required String eventJson,
  }) async {
    meetingLinks.add((noteId, eventJson));
    return const Right<ApplicationError, void>(null);
  }

  @override
  Future<Either<ApplicationError, void>> hidePlace(String noteId) async {
    hidden.add(noteId);
    return const Right<ApplicationError, void>(null);
  }

  @override
  Future<Either<ApplicationError, void>> snoozeWaiting(String key) async {
    snoozed.add(key);
    return const Right<ApplicationError, void>(null);
  }

  @override
  Future<Either<ApplicationError, void>> dismissPast(String noteId) async {
    dismissed.add(noteId);
    return const Right<ApplicationError, void>(null);
  }

  @override
  List<RevisitOption> revisitOptions() => const <RevisitOption>[
    RevisitOption(
      choice: RevisitChoice.tomorrow,
      label: 'Tomorrow',
      dayLabel: 'Tue 3 Nov',
    ),
    RevisitOption(
      choice: RevisitChoice.nextWeek,
      label: 'Next week',
      dayLabel: 'Mon 9 Nov',
    ),
  ];

  @override
  Future<Either<ApplicationError, String>> setRevisit({
    required String noteId,
    required RevisitChoice choice,
  }) async {
    revisits.add((noteId, choice));
    return const Right<ApplicationError, String>('2026-11-03');
  }

  @override
  Future<Either<ApplicationError, void>> clearRevisit(String noteId) async {
    cleared.add(noteId);
    return const Right<ApplicationError, void>(null);
  }

  @override
  Future<Either<ApplicationError, void>> noteWriting({
    required String noteId,
    required int line,
    required int column,
    required int typedChars,
    required String body,
  }) async => const Right<ApplicationError, void>(null);

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
  Future<Either<ApplicationError, AiSummary>> summariseOverview({
    required bool week,
    String? from,
  }) async {
    summarised.add((from, week));
    return Right<ApplicationError, AiSummary>(summary);
  }

  @override
  void stopAi() => stops++;
}
