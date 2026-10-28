// Kanso — note_window_page_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/note/models/data/note_mode.dart';
import 'package:kanso/shell/models/data/kanso_intent.dart';
import 'package:kanso/window/models/data/note_window_request.dart';
import 'package:kanso/window/models/data/window_intent.dart';
import 'package:kanso/window/views/widgets/note_trash_banner.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/note_window_fixture.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

Future<void> _pump(WidgetTester tester, NoteWindowFixture fixture) async {
  final appearance = SeededAppearanceBloc(const AppearanceState());
  addTearDown(appearance.close);
  await atMockNow(
    () => pumpRouted(
      tester,
      BlocProvider<AppearanceBloc>.value(
        value: appearance,
        child: fixture.page,
      ),
    ),
  );
  // The title is told to the system once the note has been read.
  await tester.pump(const Duration(milliseconds: 200));
}

void main() {
  group('NoteWindowPage', () {
    testWidgets('shows only the note: its title, facts and text', (
      tester,
    ) async {
      final fixture = await NoteWindowFixture.open(MockIds.syncDesign);
      await _pump(tester, fixture);
      expect(find.text('Sync design notes'), findsOneWidget);
      expect(find.text('Projects / Kanso'), findsOneWidget);
      expect(find.textContaining('# Goals', findRichText: true), findsOne);
      // No library, list, tabs or bottom bar: a slim bar and the note.
      expect(find.bySemanticsLabel('Search'), findsNothing);
      expect(find.bySemanticsLabel('Settings'), findsNothing);
      expect(find.bySemanticsLabel('Back'), findsNothing);
      expect(find.bySemanticsLabel('Library'), findsNothing);
    });

    testWidgets('has Edit and Preview, and the note starts as read', (
      tester,
    ) async {
      final fixture = await NoteWindowFixture.open(MockIds.syncDesign);
      await _pump(tester, fixture);
      expect(find.bySemanticsLabel('Preview'), findsWidgets);
      expect(find.bySemanticsLabel('Edit'), findsWidgets);
      expect(fixture.note.state.mode, NoteMode.preview);
    });

    testWidgets('names the window after the note', (tester) async {
      final fixture = await NoteWindowFixture.open(MockIds.syncDesign);
      await _pump(tester, fixture);
      expect(fixture.windows.titles, contains('Sync design notes'));
    });

    testWidgets('a note with no title is called Untitled', (tester) async {
      final library = FixtureLibraryRepository();
      library.notes[MockIds.syncDesign] = mockNote(title: '  ');
      final fixture = await NoteWindowFixture.open(
        MockIds.syncDesign,
        library: library,
      );
      await _pump(tester, fixture);
      expect(fixture.windows.titles.last, 'Untitled');
    });

    testWidgets('a locked note shows its lock and no text, and its title is '
        'kept out of the window name', (tester) async {
      final fixture = await NoteWindowFixture.open(MockIds.privateJournal);
      await _pump(tester, fixture);
      expect(find.text('This note is locked'), findsOneWidget);
      expect(
        find.textContaining(mockSecret, findRichText: true),
        findsNothing,
      );
      expect(fixture.windows.titles, isNotEmpty);
      expect(fixture.windows.titles, everyElement('Locked note'));
      expect(
        fixture.windows.titles.any((name) => name.contains('Private')),
        isFalse,
      );
    });

    testWidgets('a note trashed elsewhere shows the trash banner', (
      tester,
    ) async {
      final fixture = await NoteWindowFixture.open(MockIds.syncDesign);
      await _pump(tester, fixture);
      expect(find.byType(NoteTrashBanner), findsNothing);

      fixture.library.notes[MockIds.syncDesign] = mockNote(
        notebookId: 'trash',
      );
      fixture.library.changeStream.add(<String>[MockIds.syncDesign]);
      await tester.pump(const Duration(milliseconds: 300));
      await tester.pump(const Duration(milliseconds: 300));
      expect(find.byType(NoteTrashBanner), findsOneWidget);
      expect(find.text('This note is in the trash.'), findsOneWidget);

      await tester.tap(find.text('Restore'));
      await tester.pump(const Duration(milliseconds: 50));
      expect(fixture.library.changesMade, hasLength(1));
    });

    testWidgets('a note deleted for good closes its window', (tester) async {
      final fixture = await NoteWindowFixture.open(MockIds.syncDesign);
      await _pump(tester, fixture);
      expect(fixture.windows.closed, 0);

      fixture.library.notes.remove(MockIds.syncDesign);
      fixture.library.changeStream.add(<String>[MockIds.syncDesign]);
      await tester.pump(const Duration(milliseconds: 300));
      await tester.pump(const Duration(milliseconds: 300));
      expect(fixture.windows.closed, 1);
    });

    testWidgets('the menu hands a new note and a search to the main window, '
        'and closes the window', (tester) async {
      final fixture = await NoteWindowFixture.open(MockIds.syncDesign);
      await _pump(tester, fixture);

      Future<void> choose(String label) async {
        await tester.tap(find.bySemanticsLabel('More'));
        await tester.pumpAndSettle();
        await tester.tap(find.text(label));
        await tester.pumpAndSettle();
      }

      await choose('New Note');
      await choose('Search All Notes');
      expect(fixture.windows.handed, const <NoteWindowRequest>[
        NoteWindowRequest.newNote(),
        NoteWindowRequest.search(),
      ]);
      await choose('Close Window');
      expect(fixture.windows.closed, 1);
    });

    testWidgets('the close intent (⌘W) closes the window; the new note and '
        'search intents go to the main window', (tester) async {
      final fixture = await NoteWindowFixture.open(MockIds.syncDesign);
      await _pump(tester, fixture);
      final context = tester.element(find.byType(Scaffold).last);

      Actions.invoke(context, const CloseNoteWindowIntent());
      Actions.invoke(context, const NewNoteIntent());
      Actions.invoke(context, const SearchAllIntent());
      await tester.pump();
      expect(fixture.windows.closed, 1);
      expect(fixture.windows.handed, const <NoteWindowRequest>[
        NoteWindowRequest.newNote(),
        NoteWindowRequest.search(),
      ]);
    });

    testWidgets('Open in New Window opens the note again: it is brought '
        'forward by the platform', (tester) async {
      final fixture = await NoteWindowFixture.open(MockIds.syncDesign);
      await _pump(tester, fixture);
      final context = tester.element(find.byType(Scaffold).last);
      Actions.invoke(context, const OpenInNewWindowIntent());
      Actions.invoke(context, const OpenInNewWindowIntent('other'));
      await tester.pump();
      expect(fixture.windows.opened, <String>[MockIds.syncDesign, 'other']);
    });
  });
}
