// Kanso — note_list_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/list_options.dart';
import 'package:kanso/core/models/data/note_drag_data.dart';
import 'package:kanso/core/views/widgets/kanso_bottom_sheet.dart';
import 'package:kanso/core/views/widgets/kanso_drawer.dart';
import 'package:kanso/core/views/widgets/kanso_large_title.dart';
import 'package:kanso/core/views/widgets/kanso_scrim.dart';
import 'package:kanso/core/views/widgets/kanso_skeleton.dart';
import 'package:kanso/core/views/widgets/kanso_top_bar.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';
import 'package:kanso/note_list/views/widgets/list_options_sheet.dart';
import 'package:kanso/note_list/views/widgets/new_note_button.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';
import 'package:kanso/note_list/views/widgets/note_list_row.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';

Widget _scaffold(Widget pane) {
  return Scaffold(resizeToAvoidBottomInset: false, body: pane);
}

void main() {
  group('NoteListPane', () {
    testWidgets('lists the notes under Pinned and each day', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      expect(find.text('All Notes'), findsOneWidget);
      expect(find.text('10 notes'), findsOneWidget);
      expect(find.text('Pinned'), findsOneWidget);
      expect(find.text('Today'), findsOneWidget);
      expect(find.text('Yesterday'), findsWidgets);
      expect(find.text('Weekly review'), findsOneWidget);
      expect(find.text('Sync design notes'), findsOneWidget);
      // A row says where the note is, what it has and when it changed.
      expect(find.text('Projects / Kanso'), findsOneWidget);
      expect(find.text('#idea'), findsWidgets);
      expect(find.text('09:41'), findsOneWidget);
    });

    testWidgets('shows a locked note by its title and lock alone', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      // The library never hands its text on; the row must not show any.
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
          size: const Size(393, 1700),
        ),
      );
      expect(find.text('Private journal'), findsOneWidget);
      expect(find.text('Locked'), findsOneWidget);
      expect(find.textContaining(mockSecret), findsNothing);
      final row = tester.widget<NoteListRow>(
        find.widgetWithText(NoteListRow, 'Private journal'),
      );
      expect(row.row.locked, isTrue);
    });

    testWidgets('selection long press is preserved and selected rows drag', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      final received = <NoteDragData>[];
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(
            DragTarget<NoteDragData>(
              onWillAcceptWithDetails: (_) => true,
              onAcceptWithDetails: (details) => received.add(details.data),
              builder: (context, hovering, rejected) => const NoteListPane(),
            ),
            list: list,
          ),
          size: const Size(393, 1700),
        ),
      );

      final firstPress = await tester.startGesture(
        tester.getCenter(find.text('Sync design notes')),
      );
      await tester.pump(kLongPressTimeout + const Duration(milliseconds: 50));
      await firstPress.up();
      await tester.pumpAndSettle();
      expect(list.state.selected, <String>{MockIds.syncDesign});

      await tester.tap(find.text('Weekly review'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Private journal'));
      await tester.pumpAndSettle();
      expect(
        list.state.selected,
        <String>{
          MockIds.syncDesign,
          MockIds.weeklyReview,
          MockIds.privateJournal,
        },
      );

      final draggable = tester.widget<LongPressDraggable<NoteDragData>>(
        find.ancestor(
          of: find.text('Sync design notes'),
          matching: find.byType(LongPressDraggable<NoteDragData>),
        ),
      );
      final rows = list.state.selectedRows;
      expect(
        draggable.data!.noteIds,
        <String>[for (final row in rows) row.id],
      );
      expect(
        draggable.data!.titles,
        <String>[for (final row in rows) row.locked ? '' : row.title],
      );

      final drag = await tester.startGesture(
        tester.getCenter(find.text('Sync design notes')),
      );
      await tester.pump(kLongPressTimeout + const Duration(milliseconds: 50));
      await drag.moveTo(
        tester.getCenter(find.byType(DragTarget<NoteDragData>)),
      );
      await tester.pump();
      await drag.up();
      await tester.pumpAndSettle();
      expect(received, hasLength(1));
      expect(received.single.noteIds, draggable.data!.noteIds);
      expect(received.single.titles, draggable.data!.titles);
    });

    testWidgets('a tap opens the note in the place being listed', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(
        const ListPlace.notebook(id: MockIds.kanso),
      );
      final router = await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      await tester.tap(find.text('Sync design notes'));
      await tester.pumpAndSettle();
      expect(
        whereIs(router),
        '/notebooks/${Uri.encodeComponent(MockIds.kanso)}/note/'
        '${Uri.encodeComponent(MockIds.syncDesign)}',
      );
      // The history knows the note was visited.
      await tester.pump();
      expect(blocs.history.state.history.current, MockIds.syncDesign);
    });

    testWidgets('marks the note open beside the list', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(
            const NoteListPane(openNoteId: MockIds.syncDesign),
            list: list,
          ),
        ),
      );
      final open = <String>[
        for (final row in tester.widgetList<NoteListRow>(
          find.byType(NoteListRow),
        ))
          if (row.open) row.row.id,
      ];
      expect(open, <String>[MockIds.syncDesign]);
    });

    testWidgets('names a notebook and a tag by their own names', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final notebook = await blocs.listOf(
        const ListPlace.notebook(id: MockIds.garden),
      );
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: notebook),
        ),
      );
      expect(find.text('Garden'), findsOneWidget);
      await tester.pumpWidget(const SizedBox.shrink());
      final tag = await blocs.listOf(const ListPlace.tag(id: MockIds.tagIdea));
      await atMockNow(
        () =>
            pumpRouted(tester, blocs.provide(const NoteListPane(), list: tag)),
      );
      expect(
        find.descendant(
          of: find.byType(KansoLargeTitle),
          matching: find.text('#idea'),
        ),
        findsOneWidget,
      );
    });

    testWidgets('shows still bars while it loads, and says so', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all(), read: false);
      await pumpRouted(
        tester,
        blocs.provide(const NoteListPane(), list: list),
      );
      expect(find.bySemanticsLabel('Loading notes'), findsOneWidget);
      expect(find.byType(KansoSkeleton), findsWidgets);
      expect(find.byType(NoteListRow), findsNothing);
      handle.dispose();
    });

    testWidgets('says what an empty place will hold', (tester) async {
      final blocs = await BrowseBlocs.start();
      blocs.library.listResult = const Right<ApplicationError, NoteList>(
        NoteList(rows: <NoteRow>[], shape: mockShape),
      );
      final notebook = await blocs.listOf(
        const ListPlace.notebook(id: MockIds.someday),
      );
      await pumpRouted(
        tester,
        blocs.provide(const NoteListPane(), list: notebook),
      );
      expect(find.text('No notes in Someday yet'), findsOneWidget);
      expect(find.text('0 notes'), findsOneWidget);
      expect(
        find.text('Notes added here will appear in this list.'),
        findsOneWidget,
      );

      await tester.pumpWidget(const SizedBox.shrink());
      final trash = await blocs.listOf(const ListPlace.trash());
      await pumpRouted(
        tester,
        blocs.provide(const NoteListPane(), list: trash),
      );
      expect(find.text('The trash is empty'), findsOneWidget);
    });

    testWidgets('says the narrowing hides the notes, not that there are none', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      blocs.library.listResult = const Right<ApplicationError, NoteList>(
        NoteList(rows: <NoteRow>[], shape: mockShape),
      );
      list.add(
        const NoteListEvent.optionsChanged(ListOptions(statusOnly: 'dropped')),
      );
      await pumpRouted(tester, blocs.provide(const NoteListPane(), list: list));
      await tester.pump();
      await tester.pump();
      expect(find.text('No notes match'), findsOneWidget);
    });

    testWidgets('offers to try again after a failure', (tester) async {
      final blocs = await BrowseBlocs.start();
      blocs.library.listResult = const Left<ApplicationError, NoteList>(
        ApplicationError.storage(message: 'Disk error.'),
      );
      final list = await blocs.listOf(const ListPlace.all());
      await pumpRouted(tester, blocs.provide(const NoteListPane(), list: list));
      expect(find.text("These notes didn't load"), findsOneWidget);
      expect(find.textContaining('Disk error'), findsNothing);

      blocs.library.listResult = Right<ApplicationError, NoteList>(
        mockNoteList,
      );
      await tester.tap(find.text('Try again'));
      await tester.pump();
      await tester.pump();
      await tester.pumpAndSettle();
      expect(find.text("These notes didn't load"), findsNothing);
      expect(find.text('Weekly review'), findsOneWidget);
    });

    testWidgets('follows the library as it changes', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      expect(find.text('Weekly review'), findsOneWidget);
      blocs.library.listResult = Right<ApplicationError, NoteList>(
        NoteList(rows: mockRows.sublist(1), shape: mockShape),
      );
      blocs.library.changeStream.add(<String>[MockIds.weeklyReview]);
      await tester.pump(const Duration(milliseconds: 150));
      await tester.pumpAndSettle();
      expect(find.text('Weekly review'), findsNothing);
      expect(find.text('9 notes'), findsOneWidget);
    });

    group('the bar', () {
      testWidgets('has no menu or search beside the library', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(const NoteListPane(), list: list),
          ),
        );
        final handle = tester.ensureSemantics();
        await tester.pump();
        expect(find.bySemanticsLabel('Library'), findsNothing);
        expect(find.bySemanticsLabel('Search'), findsNothing);
        expect(find.bySemanticsLabel('More'), findsOneWidget);
        handle.dispose();
      });

      testWidgets('has a menu that opens the library, and search', (
        tester,
      ) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(
          const ListPlace.notebook(id: MockIds.work),
        );
        final handle = tester.ensureSemantics();
        final router = await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(const NoteListPane(), list: list),
            wrap: (page) => KansoDrawer(
              width: 334,
              drawer: const ColoredBox(color: Colors.red),
              child: page,
            ),
          ),
        );
        await tester.tap(find.bySemanticsLabel('Library'));
        await tester.pumpAndSettle();
        expect(
          tester.state<KansoDrawerState>(find.byType(KansoDrawer)).isOpen,
          isTrue,
        );
        tester.state<KansoDrawerState>(find.byType(KansoDrawer)).close();
        await tester.pumpAndSettle();
        // Search opens within the notebook the list is of.
        await tester.tap(find.bySemanticsLabel('Search'));
        await tester.pumpAndSettle();
        expect(
          whereIs(router),
          '/search?scope=notebook&in=${Uri.encodeComponent(MockIds.work)}',
        );
        handle.dispose();
      });
    });

    group('view options', () {
      testWidgets('open from More, and a sort chosen there is read', (
        tester,
      ) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        final handle = tester.ensureSemantics();
        await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(const NoteListPane(), list: list),
          ),
        );
        await tester.tap(find.bySemanticsLabel('More'));
        await tester.pumpAndSettle();
        expect(find.byType(ListOptionsSheet), findsOneWidget);
        expect(find.text('Sort by'), findsOneWidget);
        expect(find.text('Newest First'), findsOneWidget);
        expect(find.text('Pinned Only'), findsOneWidget);

        await tester.tap(find.text('Title'));
        await tester.pump();
        await tester.pumpAndSettle();
        expect(list.state.options.sort, SortKey.title);
        expect(list.state.options.descending, isFalse);
        // For titles the directions read A to Z and Z to A.
        expect(find.text('A to Z'), findsOneWidget);
        expect(find.text('Newest First'), findsNothing);
        expect(blocs.library.listQueries.last.sort, SortKey.title);
        handle.dispose();
      });

      testWidgets('a status and pinned only narrow the list', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        final handle = tester.ensureSemantics();
        await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(const NoteListPane(), list: list),
          ),
        );
        await tester.tap(find.bySemanticsLabel('More'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('On hold'));
        await tester.pump();
        await tester.pump();
        await tester.tap(find.text('Pinned Only'));
        await tester.pump();
        await tester.pump();
        expect(list.state.options.statusOnly, 'onHold');
        expect(list.state.options.pinnedOnly, isTrue);
        final query = blocs.library.listQueries.last;
        expect(query.statusOnly, 'onHold');
        expect(query.pinnedOnly, isTrue);
        await tester.tap(find.text('Any Status'));
        await tester.pump();
        expect(list.state.options.statusOnly, isNull);
        handle.dispose();
      });

      testWidgets('Recent has no sort, and Pinned no Pinned Only', (
        tester,
      ) async {
        final blocs = await BrowseBlocs.start();
        final recent = await blocs.listOf(const ListPlace.recent());
        final handle = tester.ensureSemantics();
        await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(const NoteListPane(), list: recent),
          ),
        );
        await tester.tap(find.bySemanticsLabel('More'));
        await tester.pumpAndSettle();
        expect(find.text('Sort by'), findsNothing);
        expect(find.text('Status'), findsOneWidget);
        expect(find.text('Pinned Only'), findsOneWidget);
        await tester.tap(find.text('Done'));
        await tester.pumpAndSettle();

        await tester.pumpWidget(const SizedBox.shrink());
        final pinned = await blocs.listOf(const ListPlace.pinned());
        await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(const NoteListPane(), list: pinned),
          ),
        );
        await tester.tap(find.bySemanticsLabel('More'));
        await tester.pumpAndSettle();
        expect(find.text('Sort by'), findsOneWidget);
        expect(find.text('Pinned Only'), findsNothing);
        handle.dispose();
      });

      testWidgets('close with a tap on the veil', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        final handle = tester.ensureSemantics();
        await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(const NoteListPane(), list: list),
          ),
        );
        await tester.tap(find.bySemanticsLabel('More'));
        await tester.pumpAndSettle();
        await tester.tapAt(const Offset(200, 40));
        await tester.pumpAndSettle();
        expect(find.byType(ListOptionsSheet), findsNothing);
        handle.dispose();
      });
    });
  });

  group('NoteListPane pictures', () {
    for (final brightness in Brightness.values) {
      final name = brightness.name;

      testWidgets('the list, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(_scaffold(const NoteListPane()), list: list),
            name: 'phone_note_list_$name',
            size: phoneSize,
            brightness: brightness,
          ),
        );
      });

      testWidgets('the list while it loads, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all(), read: false);
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(_scaffold(const NoteListPane()), list: list),
          name: 'phone_note_list_loading_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('an empty notebook, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        blocs.library.listResult = const Right<ApplicationError, NoteList>(
          NoteList(rows: <NoteRow>[], shape: mockShape),
        );
        final list = await blocs.listOf(
          const ListPlace.notebook(id: MockIds.someday),
        );
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(_scaffold(const NoteListPane()), list: list),
          name: 'phone_note_list_empty_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('the list after a failure, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        blocs.library.listResult = const Left<ApplicationError, NoteList>(
          ApplicationError.storage(message: 'Disk error.'),
        );
        final list = await blocs.listOf(const ListPlace.all());
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(_scaffold(const NoteListPane()), list: list),
          name: 'phone_note_list_failed_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('the view options, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(
              const Scaffold(
                resizeToAvoidBottomInset: false,
                body: Stack(
                  children: <Widget>[
                    Positioned.fill(child: NoteListPane()),
                    Positioned.fill(child: KansoScrim()),
                    Positioned.fill(
                      child: KansoBottomSheet(child: ListOptionsSheet()),
                    ),
                  ],
                ),
              ),
              list: list,
            ),
            name: 'phone_note_list_options_$name',
            size: phoneSize,
            brightness: brightness,
          ),
        );
      });
    }
  });

  group('NoteListPane new notes', () {
    testWidgets('a phone has a button afloat that asks for a note in the '
        'place', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.pinned());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      expect(find.byType(NewNoteButton), findsOneWidget);
      // The bar has no button of its own then.
      expect(
        find.descendant(
          of: find.byType(KansoTopBar),
          matching: find.bySemanticsLabel('New note'),
        ),
        findsNothing,
      );
      await tester.tap(find.byType(NewNoteButton));
      await tester.pump();
      expect(blocs.library.created.single.place, const ListPlace.pinned());
    });

    testWidgets('beside the note the button is in the bar, not afloat', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(beside: true), list: list),
        ),
      );
      expect(find.byType(NewNoteButton), findsNothing);
      await tester.tap(find.bySemanticsLabel('New note'));
      await tester.pump();
      expect(blocs.library.created.single.place, const ListPlace.all());
    });

    testWidgets('the button is not afloat while notes are being chosen', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      list.add(const NoteListEvent.selectionStarted(MockIds.syncDesign));
      await tester.pump();
      await tester.pump();
      expect(find.byType(NewNoteButton), findsNothing);
    });

    testWidgets('a locked notebook says it has to be unlocked, until the '
        'line is dismissed', (tester) async {
      final blocs = await BrowseBlocs.start();
      blocs.library.createResult = const Left<ApplicationError, Saved>(
        ApplicationError.locked(message: 'locked'),
      );
      final list = await blocs.listOf(
        const ListPlace.notebook(id: MockIds.kanso),
      );
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      await tester.tap(find.byType(NewNoteButton));
      await tester.pumpAndSettle();
      expect(
        find.text('Unlock this notebook to add notes to it.'),
        findsOneWidget,
      );
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Unlock this notebook'), findsNothing);
    });
  });
}
