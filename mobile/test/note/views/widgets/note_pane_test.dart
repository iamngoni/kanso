// Kanso — note_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/navigation_history/navigation_history_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';
import 'package:kanso/core/views/widgets/kanso_skeleton.dart';
import 'package:kanso/core/views/widgets/kanso_status_glyph.dart';
import 'package:kanso/note/views/widgets/locked_note_pane.dart';
import 'package:kanso/note/views/widgets/note_not_found_pane.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';

/// Every span of the text on screen, with the style it was given.
List<TextSpan> _spans(WidgetTester tester) {
  final spans = <TextSpan>[];
  for (final text in tester.widgetList<RichText>(find.byType(RichText))) {
    final root = text.text;
    if (root is TextSpan) {
      spans.add(root);
    }
    root.visitChildren((span) {
      if (span is TextSpan) {
        spans.add(span);
      }
      return true;
    });
  }
  return spans;
}

void main() {
  group('NotePane', () {
    testWidgets('shows the title, the facts and the text', (tester) async {
      final blocs = await BrowseBlocs.start();
      final note = await blocs.noteOf(MockIds.syncDesign);
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NotePane(inPane: false), note: note),
        ),
      );
      expect(find.text('Sync design notes'), findsOneWidget);
      expect(find.text('Projects / Kanso'), findsOneWidget);
      expect(find.text('Active'), findsOneWidget);
      expect(find.byType(KansoStatusGlyph), findsOneWidget);
      expect(find.text('#idea'), findsOneWidget);
      expect(find.text('#draft'), findsOneWidget);
      expect(find.text('Created Oct 12, 2026'), findsOneWidget);
      expect(find.text('Updated Today, 09:41'), findsOneWidget);
      // The note's own text, as it is written.
      expect(find.textContaining('# Goals', findRichText: true), findsOne);
      expect(
        find.textContaining(
          'Every device keeps a full copy.',
          findRichText: true,
        ),
        findsOneWidget,
      );
      expect(
        find.textContaining('[[Weekly review]]', findRichText: true),
        findsOne,
      );
    });

    testWidgets('styles the text with the runs the theme gives', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final note = await blocs.noteOf(MockIds.syncDesign);
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NotePane(inPane: false), note: note),
        ),
      );
      final heading = _spans(tester).firstWhere(
        (span) => span.text == '# Goals',
      );
      expect(heading.style?.color, const Color(0xFF8A4B14));
      expect(heading.style?.fontWeight, FontWeight.w700);
      final quote = _spans(tester).where(
        (span) => span.text?.startsWith('> Calm') ?? false,
      );
      expect(quote.single.style?.fontStyle, FontStyle.italic);
      // Text between runs is plain: the theme's text colour, in the pane's
      // own style.
      final plain = _spans(tester).firstWhere(
        (span) => span.children?.isNotEmpty ?? false,
      );
      expect(plain.style?.color, isNotNull);
    });

    testWidgets('lays out only the lines that are in view', (tester) async {
      final blocs = await BrowseBlocs.start();
      final body = <String>[
        for (var i = 0; i < 20000; i++) 'Line number $i of a very long note.',
      ].join('\n');
      blocs.library.notes[MockIds.syncDesign] = mockNote(
        title: 'A long note',
        body: body,
      );
      final note = await blocs.noteOf(MockIds.syncDesign);
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NotePane(inPane: false), note: note),
        ),
      );
      expect(find.byType(RichText).evaluate().length, lessThan(120));
      expect(
        find.textContaining('Line number 0 of', findRichText: true),
        findsOneWidget,
      );
      expect(
        find.textContaining('Line number 19999 of', findRichText: true),
        findsNothing,
      );
      // Scrolling to the end brings the last line in.
      await tester.drag(
        find.byType(CustomScrollView),
        const Offset(0, -9000000),
      );
      await tester.pumpAndSettle();
      await tester.fling(
        find.byType(CustomScrollView),
        const Offset(0, -20000),
        30000,
      );
      await tester.pumpAndSettle();
      expect(find.byType(RichText).evaluate().length, lessThan(120));
    });

    testWidgets('titles an untitled note', (tester) async {
      final blocs = await BrowseBlocs.start();
      blocs.library.notes[MockIds.syncDesign] = mockNote(
        title: '  ',
        body: 'Text only.',
      );
      final note = await blocs.noteOf(MockIds.syncDesign);
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NotePane(inPane: false), note: note),
        ),
      );
      expect(find.text('Untitled'), findsOneWidget);
    });

    group('a locked note', () {
      testWidgets('is its title and its lock, and nothing else', (
        tester,
      ) async {
        final blocs = await BrowseBlocs.start();
        final note = await blocs.noteOf(MockIds.privateJournal);
        await pumpRouted(
          tester,
          blocs.provide(const NotePane(inPane: false), note: note),
        );
        expect(find.byType(LockedNotePane), findsOneWidget);
        expect(find.text('Private journal'), findsOneWidget);
        expect(find.text('This note is locked'), findsOneWidget);
        // No facts and no text; the one thing to press is the password.
        expect(find.textContaining('Created'), findsNothing);
        expect(find.textContaining('Updated'), findsNothing);
        expect(find.byType(KansoButton), findsOneWidget);
        expect(find.text('Use password'), findsOneWidget);
        expect(
          find.textContaining(mockSecret, findRichText: true),
          findsNothing,
        );
      });

      testWidgets('shows no text of it even if the library handed it over', (
        tester,
      ) async {
        final blocs = await BrowseBlocs.start();
        blocs.library.notes[MockIds.privateJournal] = mockNote(
          id: MockIds.privateJournal,
          title: 'Private journal',
          body: mockSecret,
          closed: true,
          notebookId: MockIds.journal,
          tags: <String>[MockIds.tagIdea],
          status: 'active',
        );
        final note = await blocs.noteOf(MockIds.privateJournal);
        await pumpRouted(
          tester,
          blocs.provide(const NotePane(inPane: false), note: note),
        );
        expect(
          find.textContaining(mockSecret, findRichText: true),
          findsNothing,
        );
        expect(find.text('#idea'), findsNothing);
        expect(find.text('Active'), findsNothing);
        expect(blocs.renderer.themes, isEmpty);
      });
    });

    testWidgets('says a note is not there', (tester) async {
      final blocs = await BrowseBlocs.start();
      final note = await blocs.noteOf('note:gone');
      await pumpRouted(
        tester,
        blocs.provide(const NotePane(inPane: false), note: note),
      );
      expect(find.byType(NoteNotFoundPane), findsOneWidget);
      expect(find.text("This note isn't here"), findsOneWidget);
    });

    testWidgets('offers to try again after a failure', (tester) async {
      final blocs = await BrowseBlocs.start();
      final broken = _FailingLibrary();
      addTearDown(broken.close);
      final failing = await BrowseBlocs.start(broken);
      final note = await failing.noteOf(MockIds.syncDesign);
      await pumpRouted(
        tester,
        failing.provide(const NotePane(inPane: false), note: note),
      );
      expect(find.text("This note didn't load"), findsOneWidget);
      expect(find.textContaining('Disk error'), findsNothing);
      broken
        ..failing = false
        ..notes[MockIds.syncDesign] = mockOpenNote;
      await tester.tap(find.text('Try again'));
      await tester.pump();
      await tester.pump();
      await tester.pumpAndSettle();
      expect(find.text("This note didn't load"), findsNothing);
      expect(find.text('Sync design notes'), findsOneWidget);
      expect(blocs.library.listQueries, isEmpty);
    });

    testWidgets('shows still bars while it is read, and says so', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      final blocs = await BrowseBlocs.start();
      final note = await blocs.noteOf(MockIds.syncDesign, read: false);
      await pumpRouted(
        tester,
        blocs.provide(const NotePane(inPane: false), note: note),
      );
      expect(find.bySemanticsLabel('Loading the note'), findsOneWidget);
      expect(find.byType(KansoSkeleton), findsWidgets);
      handle.dispose();
    });

    group('the bars', () {
      testWidgets('on a phone: back to the list, and the bottom bar', (
        tester,
      ) async {
        final blocs = await BrowseBlocs.start();
        final note = await blocs.noteOf(MockIds.syncDesign);
        final handle = tester.ensureSemantics();
        final router = await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(const NotePane(inPane: false), note: note),
          ),
        );
        expect(find.bySemanticsLabel('Search'), findsOneWidget);
        expect(find.bySemanticsLabel('Settings'), findsOneWidget);
        await tester.tap(find.bySemanticsLabel('Search'));
        await tester.pumpAndSettle();
        expect(whereIs(router), '/search');
        router.go('/start');
        await tester.pumpAndSettle();
        await tester.tap(find.bySemanticsLabel('Settings'));
        await tester.pumpAndSettle();
        expect(topOf(router), '/settings');
        router.pop();
        await tester.pumpAndSettle();
        // With nothing to go back to, the list of all notes is where back
        // goes.
        await tester.tap(find.bySemanticsLabel('Back').first);
        await tester.pumpAndSettle();
        expect(whereIs(router), '/notes');
        handle.dispose();
      });

      testWidgets('on a tablet: back and forward through the notes opened, '
          'and no bar below', (tester) async {
        final blocs = await BrowseBlocs.start();
        final note = await blocs.noteOf(MockIds.syncDesign);
        final handle = tester.ensureSemantics();
        await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(const NotePane(inPane: true), note: note),
            size: const Size(800, 800),
          ),
        );
        expect(find.bySemanticsLabel('Search'), findsNothing);
        expect(find.bySemanticsLabel('Settings'), findsNothing);
        // Nowhere to go yet: both arrows are dimmed.
        expect(find.bySemanticsLabel('Back'), findsOneWidget);
        expect(find.bySemanticsLabel('Forward'), findsOneWidget);
        blocs.history
          ..add(const NavigationHistoryEvent.visited(MockIds.syncDesign))
          ..add(const NavigationHistoryEvent.visited(MockIds.privateJournal));
        await tester.pump();
        await tester.pump();
        await tester.tap(find.bySemanticsLabel('Back'));
        await tester.pump();
        await tester.pump();
        expect(blocs.history.state.pendingNoteId, MockIds.syncDesign);
        handle.dispose();
      });
    });
  });
}

class _FailingLibrary extends FixtureLibraryRepository {
  bool failing = true;

  @override
  Future<Either<ApplicationError, NoteView>> note(String id) async => failing
      ? const Left<ApplicationError, NoteView>(
          ApplicationError.storage(message: 'Disk error.'),
        )
      : super.note(id);
}
