// Kanso — brief_suite.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/overview_types.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// Open loops, the morning brief and the weekly review through the bridge:
/// finding, closing, saving and the settings. It runs on this computer
/// (`test/core/bridge`) and on a simulator or an emulator
/// (`integration_test`).
void briefSuite(OpenLibrary open) {
  group('KansoLibrary open loops, brief and review', () {
    test('a loop is found, listed in the brief and closed', () async {
      final library = await open();
      final id = (await library.create(
        note: const NewNote(
          title: 'Plans',
          body:
              "I'll send the deck to Sam.\nWaiting on Rudo for the invoice.\n",
        ),
      )).id;

      final board = await library.openLoops();
      expect(board.enabled, isTrue);
      expect(board.total, 2);
      final mine = board.items.singleWhere(
        (item) => item.kind == OpenLoopKind.mine,
      );
      expect(mine.noteId, id);
      expect(mine.text, contains('send the deck'));

      final brief = await library.brief();
      expect(brief.loopsTotal, 2);
      expect(brief.isEmpty, isFalse);

      final changed = await library.closeLoop(
        noteId: mine.noteId,
        key: mine.key,
        line: mine.line,
        action: LoopAction.done,
      );
      expect(changed, isTrue);
      expect((await library.openLoops()).total, 1);
    });

    test('the review is saved once and replaced only when asked', () async {
      final library = await open();
      await library.create(
        note: const NewNote(title: 'Plans', body: "I'll call Sam.\n"),
      );
      final review = await library.review();
      expect(review.isThisWeek, isTrue);
      expect(review.savedNoteId, isNull);

      final first = await library.saveReview(from: review.from, replace: false);
      expect(first.exists, isFalse);
      final again = await library.saveReview(from: review.from, replace: false);
      expect(again.exists, isTrue);
      expect(again.noteId, first.noteId);
      final replaced = await library.saveReview(
        from: review.from,
        replace: true,
      );
      expect(replaced.exists, isFalse);
      expect(
        (await library.review(from: review.from)).savedNoteId,
        first.noteId,
      );
    });

    test('Overview is read, its places hidden and revisits set', () async {
      final library = await open();
      final id = (await library.create(
        note: const NewNote(title: 'Plans', body: 'Write the plan.\n'),
      )).id;

      final today = await library.overviewToday();
      expect(today.greeting, isNotEmpty);
      expect(library.overviewSnapshot(), isNotNull);
      final week = await library.overviewWeek();
      expect(week.lede, isNotNull);

      await library.noteWriting(
        noteId: id,
        line: 0,
        column: 5,
        typedChars: 60,
        body: 'Write the plan.\n',
      );
      await library.overviewHidePlace(noteId: id);
      await library.overviewDismissPast(noteId: id);
      await library.overviewSnoozeWaiting(key: 'nothing');

      expect(library.revisitOptions(), isNotEmpty);
      expect(await library.revisitOf(noteId: id), isNull);
      final day = await library.setRevisit(
        noteId: id,
        choice: RevisitChoice.tomorrow,
      );
      expect(await library.revisitOf(noteId: id), day);
      await library.clearRevisit(noteId: id);
      expect(await library.revisitOf(noteId: id), isNull);
    });

    test('settings are clamped and plan fixed-text notices', () async {
      final library = await open();
      final prefs = await library.briefPrefs();
      expect(prefs.loops, isTrue);

      final kept = await library.setBriefPrefs(
        prefs: BriefPrefs(
          loops: prefs.loops,
          brief: true,
          briefHour: 99,
          briefMinute: 99,
          briefWeekdaysOnly: false,
          review: true,
          reviewDay: 9,
          reviewHour: 18,
          reviewMinute: 0,
          notifyBrief: true,
          notifyReview: true,
          markDoneInNote: false,
          openToOverview: true,
          quietDays: 99,
        ),
      );
      expect(kept.briefHour, 23);
      expect(kept.briefMinute, 59);
      expect(kept.reviewDay, 6);

      expect(await library.briefPlan(canNotify: false), isEmpty);
      final plan = await library.briefPlan(canNotify: true);
      expect(plan, isNotEmpty);
      for (final notice in plan) {
        expect(notice.id, startsWith(notice.isReview ? 'review-' : 'brief-'));
        expect(notice.body, isNot(contains('Plans')));
      }
    });
  });
}
