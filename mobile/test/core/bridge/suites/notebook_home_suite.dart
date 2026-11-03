// Kanso — notebook_home_suite.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/notebook_home_types.dart';

import 'open_library.dart';

/// Notebook homepages through the bridge: every call of `KansoNotebookHome`
/// and `KansoLibrary.notebookHome`.
void notebookHomeSuite(OpenLibrary open) {
  group('KansoNotebookHome', () {
    test('the settings are kept, and the words come from the core', () async {
      final home = (await open()).notebookHome();
      final first = await home.settings();
      final saved = await home.setSettings(
        input: HomeSettings(
          showHomepages: !first.showHomepages,
          summariesWithAi: first.summariesWithAi,
          refresh: first.refresh,
        ),
      );
      expect(saved.showHomepages, !first.showHomepages);
      expect((await home.settings()).showHomepages, saved.showHomepages);

      final words = await home.words();
      for (final line in <String>[
        words.askButton,
        words.openHome,
        words.addHomeNote,
        words.firstVisitButton,
        words.noOpenTasks,
        words.locked,
        words.homeMoved,
      ]) {
        expect(line, isNotEmpty);
      }
      expect(words.sectionKeys, isNotEmpty);
    });

    test('a notebook has a page, a card, a map and a Home note', () async {
      final library = await open();
      final home = library.notebookHome();
      await home.setSettings(
        input: HomeSettings(
          showHomepages: true,
          summariesWithAi: false,
          refresh: (await home.settings()).refresh,
        ),
      );
      final note = await makeNote(library, 'Plans', 'Ship the thing.\n');
      final book = (await library.note(id: note)).notebookId;

      final panel = await home.open(bookId: book);
      expect(panel, isNotNull);
      expect(panel!.locked, isFalse);
      expect(panel.header.name, isNotEmpty);
      expect(panel.sections, isNotEmpty);
      expect(await home.open(bookId: 'no-such-notebook'), isNull);

      final card = await home.card(bookId: book);
      expect(card.show_, isTrue);
      expect(card.name, panel.header.name);

      final map = await home.map(bookId: book);
      expect(map.width, 320);
      expect(map.height, 200);

      expect((await home.candidates(bookId: book)).map((c) => c.id), [note]);
      await home.chooseHome(bookId: book, noteId: note);
      expect((await home.open(bookId: book))!.header.homeNoteId, note);
      expect(await home.clearHome(bookId: book), isTrue);
      expect(await home.newHome(bookId: book), isNotEmpty);
      // A note that did not move out of its Home notebook keeps its mark.
      expect(await home.afterMove(noteId: note), isFalse);
    });

    test(
      'the layout is customised, a section hidden, then all reset',
      () async {
        final library = await open();
        final home = library.notebookHome();
        final note = await makeNote(library, 'Plans', 'Ship the thing.\n');
        final book = (await library.note(id: note)).notebookId;

        final settings = await home.sections(bookId: book);
        expect(settings, isNotEmpty);
        final keys = settings.map((s) => s.key).toList();
        final turned = await home.customise(
          bookId: book,
          order: keys.reversed.toList(),
          hidden: <String>[keys.first],
        );
        expect(turned, isNotNull);
        expect(turned!.hidden.map((h) => h.key), contains(keys.first));
        final shown = await home.showHidden(bookId: book);
        expect(shown!.hidden, isEmpty);
        final hidden = await home.hide_(
          bookId: book,
          key: keys.last,
          hidden: true,
        );
        expect(hidden!.hidden.map((h) => h.key), contains(keys.last));
        expect(await home.resetLayout(bookId: book), isNotNull);
        expect(
          await home.sectionMarkdown(bookId: book, key: keys.first),
          isA<String>(),
        );
        expect(await home.showAll(bookId: book, key: 'open_tasks'), isNotNull);
        // A section is appended to the Home note, which is made if need be.
        expect(
          await home.copyIntoHome(bookId: book, key: 'recent'),
          isA<String>(),
        );
      },
    );
  });
}
