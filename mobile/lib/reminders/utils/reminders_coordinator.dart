// Kanso — reminders_coordinator.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:flutter/foundation.dart';
import 'package:injectable/injectable.dart';

import '../../core/bridge/api/types.dart';
import '../../core/models/data/reminder_permission.dart';
import '../../core/models/data/reminder_response.dart';
import '../../core/models/repos/abs/library_repository.dart';
import '../../core/models/repos/abs/preferences_repository.dart';
import '../../core/models/repos/abs/reminder_notifier.dart';
import '../../core/models/repos/abs/reminders_repository.dart';
import '../../core/utils/debounce.dart';

/// How long the library must be quiet before the reminders are planned
/// again.
const Duration remindersDebounce = Duration(milliseconds: 500);

/// Keeps the system's reminders what the Rust core says they should be.
///
/// It plans (`reminders_plan`) at launch, when the app comes back to the
/// foreground, on every change to the library and when the switch in
/// Settings flips, shows what is due now, and has the system show the rest
/// at their times. With reminders turned off in Settings it removes them
/// all and plans nothing. It also applies the answers to notifications
/// (Mark Done, Dismiss, a tap) once each: those given while the app was
/// closed at launch and on return, the others as they come. A tap that
/// opens a task is a [TaskPlace] on [opened].
///
/// Only the title and body the core returns reach the system; no note text
/// goes through here.
@lazySingleton
class RemindersCoordinator {
  RemindersCoordinator(
    this._preferences,
    this._reminders,
    this._notifier,
    this._library,
  );

  final PreferencesRepository _preferences;
  final RemindersRepository _reminders;
  final ReminderNotifier _notifier;
  final LibraryRepository _library;

  // Single-subscription: a tap that launched the app is kept until the
  // app's host listens.
  final StreamController<TaskPlace> _opened = StreamController<TaskPlace>();
  StreamSubscription<List<String>>? _changes;
  StreamSubscription<ReminderResponse>? _responses;
  Future<void> _queue = Future<void>.value();
  bool _started = false;
  Set<String> _scheduledKeys = <String>{};

  /// The tasks the person asked to open from a notification.
  Stream<TaskPlace> get opened => _opened.stream;

  /// Starts listening, then answers what was waiting and plans.
  Future<void> start() async {
    if (_started) {
      await resumed();
      return;
    }
    _started = true;
    _changes = debounceStream(
      _library.changes(),
      remindersDebounce,
    ).listen((_) => unawaited(refresh()));
    _responses = _notifier.responses.listen(
      (response) => unawaited(
        _serialised(() async {
          try {
            await _apply(response);
          } on Object catch (error, stack) {
            _report(error, stack, 'applying a live reminder response');
          }
        }),
      ),
    );
    await resumed();
  }

  /// The app came to the foreground (or started): apply the answers given
  /// meanwhile, then plan.
  Future<void> resumed() async {
    await _serialised(() async {
      try {
        final waiting = await _notifier.takeResponses();
        for (final response in waiting) {
          try {
            await _apply(response);
          } on Object catch (error, stack) {
            _report(error, stack, 'applying a saved reminder response');
          }
        }
      } on Object catch (error, stack) {
        _report(error, stack, 'reading saved reminder responses');
      }
      await _plan();
    });
  }

  /// Plans again.
  Future<void> refresh() => _serialised(_plan);

  Future<void> dispose() async {
    await _changes?.cancel();
    await _responses?.cancel();
    // A tap that launched the app is kept until the app's host listens, so
    // the stream may never have had one. A single-subscription controller's
    // done future only completes once its stream is listened to and drained,
    // so closing is not awaited: it would never return.
    unawaited(_opened.close());
  }

  Future<void> _serialised(Future<void> Function() action) {
    final result = _queue.then((_) => action());
    _queue = result.then<void>((_) {}, onError: (Object _) {});
    return result;
  }

  Future<void> _plan() async {
    final saved = await _preferences.load();
    final enabled = saved.fold((_) => true, (it) => it.remindersEnabled);
    final now = clock.now();
    final nowUtcMs = now.toUtc().millisecondsSinceEpoch;
    if (!enabled) {
      if (!await _cancelAll()) {
        return;
      }
      await _forgetFuture(nowUtcMs);
      _scheduledKeys.clear();
      return;
    }
    ReminderPermission permission;
    try {
      permission = await _notifier.permission();
    } on Object catch (error, stack) {
      _report(error, stack, 'checking notification permission');
      permission = ReminderPermission.denied;
    }
    if (permission != ReminderPermission.granted) {
      final cancelled = await _cancelAll();
      final refused = await _reminders.plan(
        nowUtcMs: nowUtcMs,
        offsetMs: now.timeZoneOffset.inMilliseconds,
        canNotify: false,
      );
      refused.fold(
        (error) =>
            _report(error, StackTrace.current, 'planning without permission'),
        (_) {},
      );
      if (cancelled) {
        await _forgetFuture(nowUtcMs);
        _scheduledKeys.clear();
      }
      return;
    }
    final planned = await _reminders.plan(
      nowUtcMs: nowUtcMs,
      offsetMs: now.timeZoneOffset.inMilliseconds,
      canNotify: true,
    );
    await planned.fold(
      (error) async {
        _report(error, StackTrace.current, 'planning task reminders');
      },
      (plan) async {
        try {
          if (plan.showNow.isNotEmpty) {
            await _notifier.show(plan.showNow);
          }
        } on Object catch (error, stack) {
          _report(error, stack, 'showing due task reminders');
          await _forget(<String>{
            ...plan.showNow.map((reminder) => reminder.key),
            ...plan.schedule
                .map((reminder) => reminder.key)
                .where((key) => !_scheduledKeys.contains(key)),
          }, nowUtcMs);
          return;
        }
        try {
          await _notifier.schedule(plan.schedule);
          _scheduledKeys = plan.schedule
              .map((reminder) => reminder.key)
              .toSet();
        } on Object catch (error, stack) {
          _report(error, stack, 'scheduling future task reminders');
          if (await _cancelAll()) {
            await _forgetFuture(nowUtcMs);
            _scheduledKeys.clear();
          }
        }
      },
    );
  }

  Future<void> _apply(ReminderResponse response) async {
    final answered = await _reminders.answer(
      key: response.key,
      action: response.action,
      nowUtcMs: clock.now().toUtc().millisecondsSinceEpoch,
    );
    answered.fold(
      (error) =>
          _report(error, StackTrace.current, 'answering a task reminder'),
      (_) {},
    );
    if (response.action == ReminderAnswer.open && !_opened.isClosed) {
      final place = answered.fold((_) => null, (it) => it);
      // A task that is not there any more still has its note to go to.
      _opened.add(
        place ?? TaskPlace(noteId: response.noteId, line: response.line),
      );
    }
    // Answering wrote to the library: the change stream plans again.
  }

  Future<bool> _cancelAll() async {
    try {
      await _notifier.cancelAll();
      return true;
    } on Object catch (error, stack) {
      _report(error, stack, 'cancelling task reminders');
      return false;
    }
  }

  Future<void> _forget(Set<String> keys, int nowUtcMs) async {
    if (keys.isEmpty) {
      return;
    }
    final forgotten = await _reminders.forget(
      keys: keys.toList(growable: false),
      nowUtcMs: nowUtcMs,
    );
    forgotten.fold(
      (error) => _report(error, StackTrace.current, 'retrying task reminders'),
      (_) {},
    );
  }

  Future<void> _forgetFuture(int nowUtcMs) async {
    final forgotten = await _reminders.forgetFuture(nowUtcMs: nowUtcMs);
    forgotten.fold(
      (error) =>
          _report(error, StackTrace.current, 'clearing future reminders'),
      (_) {},
    );
  }

  void _report(Object error, StackTrace stack, String operation) {
    FlutterError.reportError(
      FlutterErrorDetails(
        exception: error,
        stack: stack,
        library: 'task reminders',
        context: ErrorDescription(operation),
      ),
    );
  }
}
