// Kanso — linking_suite.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/linking_types.dart';

import 'open_library.dart';

/// Links and getting around, through the bridge: every call of `KansoLinking`
/// and `KansoLibrary.linking`.
void linkingSuite(OpenLibrary open) {
  group('KansoLinking', () {
    test(
      'the outline names every heading, and the one a line is under',
      () async {
        final library = await open();
        final linking = library.linking();
        const text = 'Intro\n\n# One\n\n## Two\n';
        final headings = linking.outline(text: text);

        expect(headings.map((h) => h.text), <String>['One', 'Two']);
        expect(headings.map((h) => h.level), <int>[1, 2]);
        expect(headings.map((h) => h.line), <int>[2, 4]);

        // Above the first heading there is none; a heading covers the lines
        // under it until the next one.
        expect(linking.currentHeading(headings: headings, line: 0), isNull);
        expect(linking.currentHeading(headings: headings, line: 2), 0);
        expect(linking.currentHeading(headings: headings, line: 3), 0);
        expect(linking.currentHeading(headings: headings, line: 4), 1);
        expect(linking.currentHeading(headings: headings, line: 99), 1);

        // Text with no headings has no outline, and nothing is under it.
        expect(linking.outline(text: 'Plain words only.\n'), isEmpty);
      },
    );

    test('backlinks, unlinked mentions and Link on a mention', () async {
      final library = await open();
      final linking = library.linking();
      final target = await makeNote(
        library,
        'Target',
        'The note being pointed at.',
      );
      final linker = await makeNote(
        library,
        'Linker',
        'See [[Target]] for more.\n',
      );
      final mentioner = await makeNote(
        library,
        'Mentioner',
        'I was reading Target yesterday.\n',
      );

      final links = await linking.backlinks(id: target);
      expect(links.map((r) => r.id), contains(linker));
      expect(links.map((r) => r.title), contains('Linker'));

      final unlinked = await linking.unlinkedMentions(id: target);
      expect(unlinked.map((r) => r.id), contains(mentioner));
      expect(unlinked.map((r) => r.id), isNot(contains(linker)));

      // Link on the mention writes the link once, in one save.
      final saved = await linking.linkMention(
        targetId: target,
        mentionId: mentioner,
        line: 0,
      );
      expect(saved?.id, mentioner);

      final afterLinking = await linking.backlinks(id: target);
      expect(afterLinking.map((r) => r.id), contains(mentioner));
      final unlinkedAfter = await linking.unlinkedMentions(id: target);
      expect(unlinkedAfter.map((r) => r.id), isNot(contains(mentioner)));

      // The line holds no mention any more, so nothing is written.
      expect(
        await linking.linkMention(
          targetId: target,
          mentionId: mentioner,
          line: 0,
        ),
        isNull,
      );
    });

    test(
      'Open Note finds notes by title, and the palette opens places',
      () async {
        final library = await open();
        final linking = library.linking();
        final note = await makeNote(
          library,
          'Kilimanjaro',
          'The highest in Africa.',
        );
        await makeNote(library, 'Kalahari', 'A desert.');

        final openSections = await linking.quickOpen(query: 'Kili', open: note);
        expect(openSections, isNotEmpty);
        final opened = openSections.expand((s) => s.entries).toList();
        expect(opened.map((e) => e.label), contains('Kilimanjaro'));
        expect(opened.map((e) => e.label), isNot(contains('Kalahari')));

        // With nothing typed, the palette still offers the library's places.
        final sections = await linking.palette(
          query: '',
          context: const PaletteContext(
            signedIn: false,
            sessionHeld: false,
            aiOn: false,
            settingsOpen: false,
            imageLine: false,
            audioLine: false,
            sharedNote: false,
            handlers: <String>[],
            overdueTasks: 0,
          ),
        );
        expect(sections, isNotEmpty);
        final entries = sections.expand((s) => s.entries).toList();
        expect(entries, isNotEmpty);
        expect(
          entries.where((e) => e.target is PaletteTarget_Place),
          isNotEmpty,
        );
      },
    );
  });
}
