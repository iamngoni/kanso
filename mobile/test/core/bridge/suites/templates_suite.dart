// Kanso — templates_suite.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// Templates through the bridge: the Templates notebook, the daily template
/// and a note made from a template. It runs on this computer
/// (`test/core/bridge`) and on a simulator or an emulator (`integration_test`).
void templatesSuite(OpenLibrary open) {
  group('KansoLibrary templates', () {
    // 09:30 on 2026-05-20.
    const now = LocalTime(
      year: 2026,
      month: 5,
      day: 20,
      hour: 9,
      minute: 30,
      second: 0,
    );

    /// Writes [body] into note [id] with [title], as the editor does: the
    /// title in its field, the text typed into the body.
    Future<void> writeTemplate(
      KansoLibrary library,
      String id,
      String title,
      String body,
    ) async {
      final editor = await library.edit(id: id);
      final snapshot = await editor.snapshot();
      editor
        ..setTitle(title: title)
        ..apply(
          change: TextChange(
            baseVersion: snapshot.version,
            start: 0,
            end: 0,
            text: body,
            selection: const Range16(start: 0, end: 0),
          ),
        );
      await editor.close();
    }

    test('the daily template fills today, and places the cursor', () async {
      final library = await open();
      expect(await library.dailyTemplate(), isNull);
      expect(await library.templates(), isEmpty);

      final made = await library.newTemplate();
      expect(made.titleFirst, isTrue);
      final empty = await library.note(id: made.id);
      expect(empty.title, isEmpty);
      // It is a template already, with the empty title it has.
      final listed = await library.templates();
      expect(listed.map((row) => row.id), <String>[made.id]);
      expect(listed.single.title, isEmpty);

      await writeTemplate(
        library,
        made.id,
        'Daily',
        'Daily {{date}}\n\n{{cursor}}',
      );
      final written = await library.note(id: made.id);
      expect(written.title, 'Daily');
      expect(written.body, 'Daily {{date}}\n\n{{cursor}}');
      final rows = await library.templates();
      expect(rows.map((row) => row.id), <String>[made.id]);
      expect(rows.single.title, 'Daily');

      await library.setDailyTemplate(templateId: made.id);
      expect(await library.dailyTemplate(), made.id);

      final today = await library.journalToday(now: now);
      final note = await library.note(id: today.id);
      final body = note.body!;
      expect(body, startsWith('Daily 2026-05-20\n\n'));
      // The cursor sits where `{{cursor}}` stood: after the blank line, in
      // UTF-16 units.
      expect(today.cursor16, body.indexOf('\n\n') + 2);
      // Found again, there is no cursor to place.
      expect((await library.journalToday(now: now)).cursor16, isNull);

      await library.setDailyTemplate();
      expect(await library.dailyTemplate(), isNull);
      final after = await library.journalOpenDay(
        day: const DayDate(year: 2026, month: 5, day: 21),
        now: now,
      );
      expect(after.cursor16, isNull);
    });

    test('a note from a template goes to the notebook it is made in', () async {
      final library = await open();
      final template = await library.newTemplate();
      await writeTemplate(
        library,
        template.id,
        'Meeting',
        '# Meeting {{date}}\n\n{{cursor}}',
      );
      final work = await library.createNotebook(name: 'Work');

      final made = await library.newFromTemplate(
        templateId: template.id,
        bookId: work.id,
        now: now,
      );
      final note = await library.note(id: made.id);
      expect(note.notebookId, work.id);
      expect(note.body, startsWith('# Meeting 2026-05-20'));
      expect(made.cursor16, isNotNull);

      // Never into the Templates notebook itself: the Inbox instead.
      final templatesBook = (await library.note(id: template.id)).notebookId;
      final elsewhere = await library.newFromTemplate(
        templateId: template.id,
        bookId: templatesBook,
        now: now,
      );
      expect(
        (await library.note(id: elsewhere.id)).notebookId,
        isNot(templatesBook),
      );

      await expectLater(
        library.newFromTemplate(templateId: 'note:missing', now: now),
        throwsA(isA<KansoError_NotFound>()),
      );
    });
  });
}
