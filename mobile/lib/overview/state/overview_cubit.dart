// Kanso — overview_cubit.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../core/bridge/api/overview_types.dart';
import '../../core/models/repos/abs/ai_repository.dart';
import '../../core/models/repos/abs/digest_repository.dart';
import '../../core/models/repos/abs/library_repository.dart';
import '../../core/utils/debounce.dart';
import '../system_calendar.dart';

/// Which of Overview's two views shows.
enum OverviewPerspective { today, week }

/// How long a tick can be undone.
const Duration overviewUndoFor = Duration(seconds: 5);

/// How long the library must be quiet before the page is worked out again.
const Duration overviewRefreshDebounce = Duration(milliseconds: 400);

/// A tick that can still be taken back.
@immutable
class OverviewUndo {
  const OverviewUndo({
    required this.noteId,
    required this.line,
    required this.raw,
    required this.title,
  });

  final String noteId;
  final int line;
  final String raw;
  final String title;

  @override
  bool operator ==(Object other) =>
      other is OverviewUndo &&
      other.noteId == noteId &&
      other.line == line &&
      other.raw == raw &&
      other.title == title;

  @override
  int get hashCode => Object.hash(noteId, line, raw, title);
}

/// What the Overview page shows.
@immutable
class OverviewViewState {
  const OverviewViewState({
    this.perspective = OverviewPerspective.today,
    this.today,
    this.week,
    this.weekFrom,
    this.failed = false,
    this.aiReady = false,
    this.aiWorking = false,
    this.aiMessage,
    this.aiHidden = false,
    this.aiMore = false,
    this.expanded = const <String>{},
    this.undo,
    this.notice,
  });

  final OverviewPerspective perspective;
  final OverviewToday? today;
  final OverviewWeek? week;

  /// The Monday of the week shown, or null for this week.
  final String? weekFrom;

  /// Reading failed and there is nothing to show.
  final bool failed;
  final bool aiReady;
  final bool aiWorking;

  /// Why the last request failed.
  final String? aiMessage;
  final bool aiHidden;
  final bool aiMore;

  /// Sections opened past their cap.
  final Set<String> expanded;
  final OverviewUndo? undo;

  /// A line to tell the person once.
  final String? notice;

  OverviewViewState copyWith({
    OverviewPerspective? perspective,
    Object? today = _keep,
    Object? week = _keep,
    Object? weekFrom = _keep,
    bool? failed,
    bool? aiReady,
    bool? aiWorking,
    Object? aiMessage = _keep,
    bool? aiHidden,
    bool? aiMore,
    Set<String>? expanded,
    Object? undo = _keep,
    Object? notice = _keep,
  }) => OverviewViewState(
    perspective: perspective ?? this.perspective,
    today: identical(today, _keep) ? this.today : today as OverviewToday?,
    week: identical(week, _keep) ? this.week : week as OverviewWeek?,
    weekFrom: identical(weekFrom, _keep) ? this.weekFrom : weekFrom as String?,
    failed: failed ?? this.failed,
    aiReady: aiReady ?? this.aiReady,
    aiWorking: aiWorking ?? this.aiWorking,
    aiMessage: identical(aiMessage, _keep)
        ? this.aiMessage
        : aiMessage as String?,
    aiHidden: aiHidden ?? this.aiHidden,
    aiMore: aiMore ?? this.aiMore,
    expanded: expanded ?? this.expanded,
    undo: identical(undo, _keep) ? this.undo : undo as OverviewUndo?,
    notice: identical(notice, _keep) ? this.notice : notice as String?,
  );

  @override
  bool operator ==(Object other) =>
      other is OverviewViewState &&
      other.perspective == perspective &&
      other.today == today &&
      other.week == week &&
      other.weekFrom == weekFrom &&
      other.failed == failed &&
      other.aiReady == aiReady &&
      other.aiWorking == aiWorking &&
      other.aiMessage == aiMessage &&
      other.aiHidden == aiHidden &&
      other.aiMore == aiMore &&
      setEquals(other.expanded, expanded) &&
      other.undo == undo &&
      other.notice == notice;

  @override
  int get hashCode => Object.hash(
    perspective,
    today,
    week,
    weekFrom,
    failed,
    aiReady,
    aiWorking,
    aiMessage,
    aiHidden,
    aiMore,
    Object.hashAllUnordered(expanded),
    undo,
    notice,
  );
}

const Object _keep = Object();

/// Overview: reads Today and the week from the Rust core, shows the last
/// Today kept at once, reads again when the library changes, and turns taps
/// on rows into writes. Every read goes through one generation counter and
/// every write through [_queue], so two taps on the same data never race.
class OverviewCubit extends Cubit<OverviewViewState> {
  OverviewCubit(this._digest, this._library, this._ai)
    : super(const OverviewViewState());

  final DigestRepository _digest;
  final LibraryRepository _library;
  final AiRepository _ai;

  StreamSubscription<List<String>>? _changes;
  StreamSubscription<Object?>? _aiWatch;
  Timer? _undoTimer;
  int _generation = 0;

  /// The write in flight; the next waits for it.
  Future<void> _queue = Future<void>.value();

  /// Shows the snapshot, reads the page, and keeps it up to date.
  Future<void> load({OverviewPerspective? perspective}) async {
    final first = _digest.overviewSnapshot();
    emit(
      state.copyWith(
        perspective: perspective ?? state.perspective,
        today: state.today ?? first,
      ),
    );
    _changes ??= debounceStream(
      _library.changes(),
      overviewRefreshDebounce,
    ).listen((_) => unawaited(refresh()));
    _aiWatch ??= _ai.settingsChanges().listen(
      (_) => unawaited(_refreshAiReady()),
    );
    unawaited(_refreshAiReady());
    await refresh();
  }

  /// Works the page out again.
  Future<void> refresh() async {
    final mine = ++_generation;
    await _refreshSystemCalendarEvents();
    final today = await _digest.overviewToday();
    final wantWeek = state.perspective == OverviewPerspective.week;
    final week = wantWeek
        ? await _digest.overviewWeek(from: state.weekFrom)
        : null;
    if (isClosed || mine != _generation) {
      return;
    }
    var next = state;
    today.fold(
      (_) => next = next.copyWith(failed: next.today == null),
      (it) => next = next.copyWith(today: it, failed: false),
    );
    week?.fold((_) {}, (it) => next = next.copyWith(week: it));
    emit(next);
  }

  Future<void> _refreshSystemCalendarEvents() async {
    final gateway = SystemCalendarGateway();
    try {
      await gateway.calendars();
      final today = DateTime.now();
      final start = DateTime(today.year, today.month, today.day);
      final events = await gateway.events(
        start: start.subtract(const Duration(days: 7)).millisecondsSinceEpoch,
        end: start.add(const Duration(days: 21)).millisecondsSinceEpoch,
      );
      await _digest.putCalendarEvents(
        jsonEncode(events.map((event) => event.toEngineMap()).toList()),
      );
    } on PlatformException {
      // Revoked access must not leave events from the prior permission grant
      // visible in the local Overview cache.
      await _digest.putCalendarEvents('[]');
    } on MissingPluginException {
      // The system calendar exists only on iOS and Android.
      await _digest.putCalendarEvents('[]');
    }
  }

  Future<void> _refreshAiReady() async {
    final settings = (await _ai.settings()).fold((_) => null, (it) => it);
    var ready = false;
    if (settings != null && settings.enabled) {
      final providers = (await _ai.providers()).fold((_) => null, (it) => it);
      final provider = providers
          ?.where((it) => it.key == settings.provider)
          .firstOrNull;
      ready = provider != null && (settings.hasKey || !provider.needsKey);
    }
    if (!isClosed) {
      emit(state.copyWith(aiReady: ready));
    }
  }

  // MARK: Views

  Future<void> showPerspective(OverviewPerspective perspective) async {
    if (perspective == state.perspective) {
      return;
    }
    emit(
      state.copyWith(
        perspective: perspective,
        aiHidden: false,
        aiMore: false,
        aiMessage: null,
      ),
    );
    await refresh();
  }

  /// Moves the week shown by `weeks`.
  Future<void> stepWeek(int weeks) async {
    final from = state.week?.from;
    if (from == null) {
      return;
    }
    final parts = from.split('-').map(int.parse).toList();
    final moved = DateTime.utc(parts[0], parts[1], parts[2] + 7 * weeks);
    emit(
      state.copyWith(
        weekFrom: _iso(moved),
        week: null,
        aiHidden: false,
        aiMore: false,
      ),
    );
    await refresh();
  }

  Future<void> thisWeek() async {
    emit(state.copyWith(weekFrom: null, week: null));
    await refresh();
  }

  void expand(String key) =>
      emit(state.copyWith(expanded: <String>{...state.expanded, key}));

  void toggleExpanded(String key) {
    final next = <String>{...state.expanded};
    if (!next.remove(key)) {
      next.add(key);
    }
    emit(state.copyWith(expanded: next));
  }

  void clearNotice() => emit(state.copyWith(notice: null));

  // MARK: Rows

  /// Ticks an agenda row: a task is checked in its note and can be undone
  /// for a few seconds; a promise is closed.
  Future<void> tick(OverviewAgendaRow row) => _write(() async {
    if (row.noteId.isEmpty) {
      return;
    }
    if (row.isLoop) {
      await _digest.doneLoop(noteId: row.noteId, key: row.key, line: row.line);
    } else {
      final result = await _library.setTask(
        noteId: row.noteId,
        line: row.line,
        text: row.raw,
        checked: true,
      );
      result.fold((_) {}, (_) {
        _setUndo(
          OverviewUndo(
            noteId: row.noteId,
            line: row.line,
            raw: row.raw,
            title: row.title,
          ),
        );
      });
    }
    await refresh();
  });

  void _setUndo(OverviewUndo undo) {
    _undoTimer?.cancel();
    emit(state.copyWith(undo: undo));
    _undoTimer = Timer(overviewUndoFor, () {
      if (!isClosed) {
        emit(state.copyWith(undo: null));
      }
    });
  }

  Future<void> undoTick() => _write(() async {
    final undo = state.undo;
    if (undo == null) {
      return;
    }
    _undoTimer?.cancel();
    emit(state.copyWith(undo: null));
    await _library.setTask(
      noteId: undo.noteId,
      line: undo.line,
      text: undo.raw,
      checked: false,
    );
    await refresh();
  });

  /// "Done" on a waiting row.
  Future<void> doneWaiting(OverviewListRow row) => _write(() async {
    if (row.isLoop) {
      await _digest.doneLoop(noteId: row.noteId, key: row.key, line: row.line);
    } else {
      await _library.setTask(
        noteId: row.noteId,
        line: row.line,
        text: row.raw,
        checked: true,
      );
    }
    await refresh();
  });

  Future<void> snoozeWaiting(OverviewListRow row) => _write(() async {
    await _digest.snoozeWaiting(row.key);
    await refresh();
  });

  Future<void> hidePlace(OverviewListRow row) => _write(() async {
    await _digest.hidePlace(row.noteId);
    await refresh();
  });

  Future<void> dismissPast(OverviewPast past) => _write(() async {
    await _digest.dismissPast(past.noteId);
    await refresh();
  });

  /// Brings [noteId] back on [choice]'s day; tells which day it is.
  Future<void> revisit(String noteId, OverviewRevisit choice) => _write(
    () async {
      final result = await _digest.setRevisit(
        noteId: noteId,
        choice: choice.choice,
      );
      result.fold((_) {}, (_) => emit(state.copyWith(notice: choice.dayLabel)));
      await refresh();
    },
  );

  // MARK: AI

  /// Writes one short paragraph over what shows.
  Future<void> summarise() async {
    if (!state.aiReady || state.aiWorking) {
      return;
    }
    final week = state.perspective == OverviewPerspective.week;
    emit(
      state.copyWith(
        aiWorking: true,
        aiMessage: null,
        aiHidden: false,
        aiMore: false,
      ),
    );
    final result = await _digest.summariseOverview(
      from: week ? state.week?.from : null,
      week: week,
    );
    if (isClosed) {
      return;
    }
    await result.fold(
      (error) async =>
          emit(state.copyWith(aiWorking: false, aiMessage: error.message)),
      (_) async {
        emit(state.copyWith(aiWorking: false));
        await refresh();
      },
    );
  }

  void stopSummary() => _digest.stopAi();

  void hideAi() => emit(state.copyWith(aiHidden: true));

  void moreAi() => emit(state.copyWith(aiMore: true));

  // MARK: Plumbing

  Future<void> _write(Future<void> Function() write) {
    final next = _queue.then((_) => write());
    _queue = next.catchError((Object _) {});
    return next;
  }

  @override
  Future<void> close() async {
    _undoTimer?.cancel();
    await _changes?.cancel();
    await _aiWatch?.cancel();
    _digest.stopAi();
    return super.close();
  }
}

/// A *Revisit in…* choice with the words for its day.
@immutable
class OverviewRevisit {
  const OverviewRevisit(this.choice, this.dayLabel);

  final RevisitChoice choice;
  final String dayLabel;
}

String _iso(DateTime day) =>
    '${day.year.toString().padLeft(4, '0')}-'
    '${day.month.toString().padLeft(2, '0')}-'
    '${day.day.toString().padLeft(2, '0')}';
