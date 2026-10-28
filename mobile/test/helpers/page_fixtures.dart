// Kanso — page_fixtures.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/models/repos/abs/reminder_notifier.dart';

import 'fixture_board_repository.dart';
import 'fixture_journal_repository.dart';
import 'fixture_reminder_notifier.dart';

/// The fixtures behind the blocs the pages make for themselves (see
/// `registerPageBlocs`): journal and templates, boards, and the system's
/// reminders. A test changes them before it pumps, or reads what they were
/// asked.
class PageFixtures {
  PageFixtures({
    FixtureJournalRepository? journal,
    FixtureBoardRepository? boards,
    FixtureReminderNotifier? notifier,
  }) : journal = journal ?? FixtureJournalRepository(),
       boards = boards ?? FixtureBoardRepository(),
       notifier = notifier ?? FixtureReminderNotifier();

  final FixtureJournalRepository journal;
  final FixtureBoardRepository boards;
  final FixtureReminderNotifier notifier;

  ReminderNotifier get reminderNotifier => notifier;
}
