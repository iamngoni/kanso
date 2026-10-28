// Kanso — note_window_goldens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/note_window_fixture.dart';

/// The windows a note's own window is pictured in: an iPad in both ways
/// round, and an Android tablet. The window is the whole screen of a
/// tablet here: a note window is the app's only content (a window beside
/// the app is narrower, and is laid out the same).
const List<KansoViewport> _viewports = <KansoViewport>[
  KansoViewport.iPadPortrait,
  KansoViewport.iPadLandscape,
  KansoViewport.androidTablet,
];

void main() {
  for (final brightness in Brightness.values) {
    final name = brightness.name;

    for (final viewport in _viewports) {
      testWidgets('a note in its own window on ${viewport.slug}, $name', (
        tester,
      ) async {
        final fixture = await NoteWindowFixture.open(
          MockIds.syncDesign,
          brightness: brightness,
        );
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            fixture.page,
            name: 'note_window_${viewport.slug}_$name',
            viewport: viewport,
            brightness: brightness,
          ),
        );
      });
    }

    testWidgets('a note in the trash, in its own window, $name', (
      tester,
    ) async {
      final library = FixtureLibraryRepository();
      library.notes[MockIds.syncDesign] = mockNote(
        notebookId: 'trash',
        body: mockNoteBody,
        tags: <String>[MockIds.tagIdea, MockIds.tagDraft],
        status: 'active',
      );
      final fixture = await NoteWindowFixture.open(
        MockIds.syncDesign,
        brightness: brightness,
        library: library,
      );
      await atMockNow(
        () => expectGoldenOnBothPlatforms(
          tester,
          fixture.page,
          name: 'note_window_trash_ipad_portrait_$name',
          viewport: KansoViewport.iPadPortrait,
          brightness: brightness,
        ),
      );
    });

    testWidgets('a locked note in its own window, $name', (tester) async {
      final fixture = await NoteWindowFixture.open(
        MockIds.privateJournal,
        brightness: brightness,
      );
      await expectGoldenOnBothPlatforms(
        tester,
        fixture.page,
        name: 'note_window_locked_ipad_portrait_$name',
        viewport: KansoViewport.iPadPortrait,
        brightness: brightness,
      );
      // Nothing of the note but its lock is on screen or in the window's name.
      expect(
        fixture.windows.titles.any((title) => title.contains('Private')),
        isFalse,
      );
    });
  }
}
