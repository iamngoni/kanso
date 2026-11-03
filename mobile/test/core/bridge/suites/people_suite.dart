// Kanso — people_suite.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/people_types.dart';

import 'open_library.dart';

/// People & Projects through the bridge: every call of `KansoPeople` and
/// `KansoLibrary.people`.
void peopleSuite(OpenLibrary open) {
  group('KansoPeople', () {
    test('the settings start as specified and are kept', () async {
      final people = (await open()).people();
      final first = await people.settings();
      expect(first.suggest, isTrue);
      expect(first.writeWithAi, isTrue);
      expect(first.detectWithAi, isFalse);
      expect(first.refresh, 'auto');
      final saved = await people.setSettings(
        input: PeopleSettings(
          suggest: false,
          keepIn: first.keepIn,
          writeWithAi: first.writeWithAi,
          refresh: 'press',
          detectWithAi: false,
        ),
      );
      expect(saved.refresh, 'press');
      expect((await people.settings()).suggest, isFalse);
    });

    test('a page is made, listed, shown and merged', () async {
      final library = await open();
      final people = library.people();
      final sam = await people.create(
        kind: PeopleKind.person,
        name: 'Sam Moyo',
        aliases: <String>['Sam'],
      );
      await makeNote(library, 'Weekly', 'I promised Sam the contract.');
      final pages = await people.pages();
      expect(pages.map((p) => p.title), contains('Sam Moyo'));
      expect((await people.status()).pages, greaterThanOrEqualTo(1));

      final panel = await people.page(id: sam);
      expect(panel, isNotNull);
      expect(panel!.header.title, 'Sam Moyo');
      expect(panel.sections.map((s) => s.key), contains('timeline'));
      final plain = await makeNote(library, 'Plain', 'Nothing here.');
      expect(await people.page(id: plain), isNull);

      final other = await people.create(
        kind: PeopleKind.person,
        name: 'T. Moyo',
        aliases: <String>[],
      );
      expect(await people.addAlias(pageId: sam, alias: 'Tendai'), isTrue);
      final merged = await people.merge(into: sam, from: other);
      expect(merged.into, sam);
    });

    test('suggestions can be dismissed and asked for again', () async {
      final people = (await open()).people();
      expect(await people.suggestions(), isA<List<PeopleSuggestion>>());
      await people.suggestAgain(name: 'nobody');
      expect(await people.dismissed(), isA<List<String>>());
      // AI labelling is off until the person turns it on.
      await expectLater(people.detectWithAi(), throwsA(anything));
    });

    test('sections can be hidden and shown again', () async {
      final people = (await open()).people();
      final id = await people.create(
        kind: PeopleKind.project,
        name: 'Lake Placid',
        aliases: <String>[],
      );
      final hidden = await people.hide_(id: id, key: 'timeline', hidden: true);
      expect(hidden!.hidden.map((h) => h.key), contains('timeline'));
      final shown = await people.showHidden(id: id);
      expect(shown!.hidden, isEmpty);
      expect(
        await people.sectionMarkdown(id: id, key: 'timeline'),
        isA<String>(),
      );
    });
    test(
      'detection runs, a suggestion is turned away, a section is kept',
      () async {
        final library = await open();
        final people = library.people();
        final book = await makeNote(
          library,
          'Lunch',
          'Lunch with Tendai Chirwa went well\n@rudo follows up\n',
        );
        expect(book, isNotEmpty);
        expect(await people.detect(force: true), greaterThanOrEqualTo(0));
        final found = await people.suggestions();
        for (final s in found) {
          await people.dismiss(key: s.key, never: true);
        }
        expect(await people.suggestions(), isEmpty);

        final id = await people.create(
          kind: PeopleKind.person,
          name: 'Sam Moyo',
          aliases: <String>[],
        );
        await people.confirmAlias(key: 'nobody', pageId: id);
        final kept = await people.pin(id: id, key: 'promises', on_: true);
        expect(kept, isNotNull);
        expect(await people.copyIntoNote(id: id, key: 'timeline'), isA<bool>());
        final marked = await people.notRight(id: id, key: 'promises', index: 0);
        expect(marked, isNotNull);
        final all = await people.showAll(id: id, key: 'timeline');
        expect(all, isNotNull);
      },
    );

    test('writing a section is refused while AI is off', () async {
      final people = (await open()).people();
      final id = await people.create(
        kind: PeopleKind.project,
        name: 'Lake Placid',
        aliases: <String>[],
      );
      await expectLater(
        people.refreshSection(id: id, key: 'discussed'),
        throwsA(anything),
      );
      await expectLater(people.refreshAll(id: id), throwsA(anything));
    });
  });
}
