// Kanso — publish_suite.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/publish.dart' as publish;
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// Publish calls through the bridge without a signed-in session, so these
/// checks never contact the share service.
void publishSuite(OpenLibrary open) {
  group('KansoPublisher', () {
    test(
      'expiry choices and current note and notebook plans are readable',
      () async {
        final library = await open();
        final publisher = await library.publisher();
        final noteId = await makeNote(library, 'Publish', 'A private note.');
        final notebook = await library.createNotebook(name: 'Site');

        const link = publish.PublishLink(
          url: 'https://example.invalid/note#key',
          expiry: publish.PublishExpiry.never,
          publishedAtMs: 0,
        );
        const expiryOption = publish.PublishExpiryOption(
          expiry: publish.PublishExpiry.never,
          label: 'Never',
        );
        const noteSnapshot = publish.PublishNoteInfo(
          id: 'note:published',
          title: 'Published note',
          body: 'Body',
          rev: '1-a',
          published: link,
          needsUpdate: false,
        );
        const notebookSnapshot = publish.PublishNotebookInfo(
          id: 'book:published',
          name: 'Published site',
          published: link,
          needsUpdate: false,
          contents: '1 note.',
          locked: '',
          noteCount: 1,
          fileCount: 0,
        );
        expect(expiryOption.expiry, publish.PublishExpiry.never);
        expect(noteSnapshot.published, link);
        expect(notebookSnapshot.noteCount, 1);

        final choices = await publish.KansoPublisher.expiryOptions();
        expect(choices.map((choice) => choice.expiry), <publish.PublishExpiry>[
          publish.PublishExpiry.never,
          publish.PublishExpiry.day,
          publish.PublishExpiry.week,
          publish.PublishExpiry.month,
        ]);
        expect(choices.map((choice) => choice.label), isNotEmpty);

        final note = await publisher.note(id: noteId);
        expect(note.title, 'Publish');
        expect(note.published, isNull);
        expect(note.refusal, isNull);

        final site = await publisher.notebook(id: notebook.id);
        expect(site.name, 'Site');
        expect(site.published, isNull);
        expect(site.contents, isNotEmpty);
      },
    );

    test(
      'publish and unpublish require sign-in before any share request',
      () async {
        final library = await open();
        final publisher = await library.publisher();
        final noteId = await makeNote(library, 'Private', 'Not uploaded.');
        final notebook = await library.createNotebook(name: 'Private site');
        const palette = PaletteDto(
          dark: false,
          background: 0,
          text: 0,
          muted: 0,
          accent: 0,
          border: 0,
          surface: 0,
          quote: 0,
          taskDone: 0,
          inlineCodeBackground: 0,
        );
        const options = PreviewOptions(
          size: 16,
          utcOffsetSecs: 0,
          code: <SyntaxStyleRow>[],
        );

        await expectLater(
          publisher.publishNote(
            id: noteId,
            expiry: publish.PublishExpiry.never,
            palette: palette,
            options: options,
          ),
          throwsA(anything),
        );
        await expectLater(
          publisher.unpublishNote(id: noteId),
          throwsA(anything),
        );
        await expectLater(
          publisher.publishNotebook(
            id: notebook.id,
            expiry: publish.PublishExpiry.never,
            palette: palette,
            options: options,
          ),
          throwsA(anything),
        );
        await expectLater(
          publisher.unpublishNotebook(id: notebook.id),
          throwsA(anything),
        );
      },
    );
  });
}
