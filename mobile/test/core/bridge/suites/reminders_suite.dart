// Kanso — reminders_suite.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// Reminders through the bridge: what to show now, what to have the system
/// show later, what an answer does, and what forgetting one writes. It runs
/// on this computer (`test/core/bridge`) and on a simulator or an emulator
/// (`integration_test`).
void remindersSuite(OpenLibrary open) {
  group('KansoLibrary reminders', () {
    /// The phone's zone: two hours ahead of UTC.
    const int offsetMs = 2 * 60 * 60 * 1000;

    /// 09:00:30 local on 2026-05-20, as UTC milliseconds.
    final int nowUtcMs = DateTime.utc(
      2026,
      5,
      20,
      7,
      0,
      30,
    ).millisecondsSinceEpoch;

    const String body =
        '- [ ] Call Sam @2026-05-20T09:00\n'
        '- [ ] Later @2026-05-20T18:00\n'
        '- [ ] Next day @2026-05-21T08:00\n';

    test('a plan shows what is due and schedules what is to come', () async {
      final library = await open();
      await library.create(
        note: const NewNote(title: 'Chores', body: body),
      );

      final plan = await library.remindersPlan(
        nowUtcMs: nowUtcMs,
        offsetMs: offsetMs,
        canNotify: true,
      );
      expect(
        plan.showNow.map((note) => note.title),
        <String>['Call Sam'],
      );
      expect(plan.showNow.single.body, 'Today 09:00 · Chores');
      expect(plan.showNow.single.line, 0);
      expect(plan.showNow.single.fireAt.hour, 9);
      expect(
        plan.schedule.map((note) => note.title),
        <String>['Later', 'Next day'],
      );
      expect(plan.schedule.first.body, 'Today 18:00 · Chores');

      // With notifications off nothing is planned, and nothing recorded.
      final off = await library.remindersPlan(
        nowUtcMs: nowUtcMs,
        offsetMs: offsetMs,
        canNotify: false,
      );
      expect(off.showNow, isEmpty);
      expect(off.schedule, isEmpty);
      // What was shown is recorded, so a later plan doesn't show it again;
      // what is still to come is planned again.
      final again = await library.remindersPlan(
        nowUtcMs: nowUtcMs,
        offsetMs: offsetMs,
        canNotify: true,
      );
      expect(again.showNow, isEmpty);
      expect(
        again.schedule.map((note) => note.title),
        <String>['Later', 'Next day'],
      );
    });

    test(
      'forgetting retries one, and forgetting the future clears the rest',
      () async {
        final library = await open();
        await library.create(
          note: const NewNote(title: 'Chores', body: body),
        );

        final first = await library.remindersPlan(
          nowUtcMs: nowUtcMs,
          offsetMs: offsetMs,
          canNotify: true,
        );
        expect(first.showNow, hasLength(1));
        expect(first.schedule, hasLength(2));

        // The platform failed to show it: forgetting it lets the next plan try.
        await library.remindersForget(
          keys: <String>[first.showNow.single.key],
          nowUtcMs: nowUtcMs,
        );
        final retried = await library.remindersPlan(
          nowUtcMs: nowUtcMs,
          offsetMs: offsetMs,
          canNotify: true,
        );
        expect(retried.showNow.map((note) => note.title), <String>['Call Sam']);
        expect(retried.schedule, hasLength(2));

        // Notifications were turned off: the future's records are cleared, so
        // the platform is asked for nothing while they are off...
        await library.remindersForgetFuture(nowUtcMs: nowUtcMs);
        final off = await library.remindersPlan(
          nowUtcMs: nowUtcMs,
          offsetMs: offsetMs,
          canNotify: false,
        );
        expect(off.showNow, isEmpty);
        expect(off.schedule, isEmpty);
        // ...and what is still to come can be scheduled again when they are
        // back on. The one already shown stays shown.
        final retriedAll = await library.remindersPlan(
          nowUtcMs: nowUtcMs,
          offsetMs: offsetMs,
          canNotify: true,
        );
        expect(retriedAll.showNow, isEmpty);
        expect(
          retriedAll.schedule.map((note) => note.title),
          <String>['Later', 'Next day'],
        );
      },
    );

    test('an answer says where to go, or ticks the task off', () async {
      final library = await open();
      final made = await library.create(
        note: const NewNote(title: 'Chores', body: body),
      );
      final plan = await library.remindersPlan(
        nowUtcMs: nowUtcMs,
        offsetMs: offsetMs,
        canNotify: true,
      );
      final key = plan.showNow.single.key;

      final place = await library.reminderAnswer(
        key: key,
        answer: ReminderAnswer.open,
        nowUtcMs: nowUtcMs,
      );
      expect(place, isNotNull);
      expect(place!.noteId, made.id);
      expect(place.line, 0);

      // Dismissing says nothing about where to go.
      expect(
        await library.reminderAnswer(
          key: key,
          answer: ReminderAnswer.dismiss,
          nowUtcMs: nowUtcMs + 1000,
        ),
        isNull,
      );

      // Mark Done ticks the task off in its note.
      await library.reminderAnswer(
        key: key,
        answer: ReminderAnswer.markDone,
        nowUtcMs: nowUtcMs + 2000,
      );
      final note = await library.note(id: made.id);
      expect(note.body, contains('- [x] Call Sam'));
      // A reminder that is gone is nothing to do.
      expect(
        await library.reminderAnswer(
          key: 'note:nowhere#0@2026-05-20T09:00',
          answer: ReminderAnswer.open,
          nowUtcMs: nowUtcMs + 3000,
        ),
        isNull,
      );
    });
  });
}
