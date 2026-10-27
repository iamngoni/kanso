// Kanso — note_list_actions_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/views/widgets/kanso_checkbox.dart';
import 'package:kanso/core/views/widgets/kanso_selection_bar.dart';
import 'package:kanso/core/views/widgets/kanso_swipe_action_tile.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';
import 'package:kanso/note_list/views/widgets/move_sheet.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';

Widget _scaffold(Widget pane) {
  return Scaffold(resizeToAvoidBottomInset: false, body: pane);
}

Future<GoRouter> _pumpList(
  WidgetTester tester,
  BrowseBlocs blocs,
  NoteListBloc list,
) {
  return atMockNow(
    () => pumpRouted(
      tester,
      blocs.provide(const NoteListPane(), list: list),
      size: const Size(393, 1700),
    ),
  );
}

/// A label in the sheet that asks where to move notes to.
Finder inSheet(String label) => find.descendant(
  of: find.byType(MoveSheet),
  matching: find.text(label),
);

void main() {
  group('choosing several notes', () {
    testWidgets('a long press starts it, and a tap chooses or lets go', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      final router = await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Sync design notes'));
      await tester.pump();
      expect(find.text('1 selected'), findsOneWidget);
      expect(find.byType(KansoSelectionBar), findsOneWidget);
      expect(find.byType(KansoCheckbox), findsWidgets);
      expect(blocs.haptics.selections, 1);

      await tester.tap(find.text('Sourdough starter'));
      await tester.pump();
      expect(find.text('2 selected'), findsOneWidget);
      await tester.tap(find.text('Sync design notes'));
      await tester.pump();
      expect(find.text('1 selected'), findsOneWidget);
      expect(blocs.haptics.selections, 3);
      // Nothing was opened by those taps.
      expect(whereIs(router), '/start');
    });

    testWidgets('done goes back to the list bar', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Sync design notes'));
      await tester.pump();
      await tester.tap(find.text('Done'));
      await tester.pump();
      expect(find.byType(KansoSelectionBar), findsNothing);
      expect(find.text('All Notes'), findsOneWidget);
      expect(find.byType(KansoCheckbox), findsNothing);
    });

    testWidgets('the system back ends it before it leaves', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Sync design notes'));
      await tester.pump();
      await tester.binding.handlePopRoute();
      await tester.pump();
      expect(find.byType(KansoSelectionBar), findsNothing);
    });

    testWidgets('pinning pins them all, then ends choosing', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      final handle = tester.ensureSemantics();
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Sync design notes'));
      await tester.pump();
      await tester.tap(find.text('Sourdough starter'));
      await tester.pump();
      await tester.tap(find.bySemanticsLabel('Pin'));
      await tester.pump();
      await tester.pump();
      final (ids, change) = blocs.library.changesMade.single;
      expect(ids, <String>[MockIds.syncDesign, MockIds.sourdough]);
      expect(change, const NoteChange.togglePin());
      expect(find.byType(KansoSelectionBar), findsNothing);
      expect(blocs.haptics.impacts, 1);
      handle.dispose();
    });

    testWidgets('says Unpin when everything chosen is pinned', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      final handle = tester.ensureSemantics();
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Weekly review'));
      await tester.pump();
      expect(find.bySemanticsLabel('Unpin'), findsOneWidget);
      expect(find.bySemanticsLabel('Pin'), findsNothing);
      handle.dispose();
    });

    testWidgets('trashing moves them to the trash', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      final handle = tester.ensureSemantics();
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Lisbon in spring'));
      await tester.pump();
      await tester.tap(find.bySemanticsLabel('Trash'));
      await tester.pump();
      final (ids, change) = blocs.library.changesMade.single;
      expect(ids, <String>[MockIds.lisbon]);
      expect(change, const NoteChange.trash());
      handle.dispose();
    });

    testWidgets('moving asks for a notebook, and moves them there', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      final handle = tester.ensureSemantics();
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Lisbon in spring'));
      await tester.pump();
      await tester.tap(find.text('Reading list 2026'));
      await tester.pump();
      await tester.tap(find.bySemanticsLabel('Move'));
      await tester.pumpAndSettle();
      expect(find.byType(MoveSheet), findsOneWidget);
      expect(find.text('Move 2 notes to'), findsOneWidget);
      // The notebooks, in the library's order, a sub-notebook indented.
      expect(inSheet('Inbox'), findsOneWidget);
      expect(inSheet('Kanso'), findsOneWidget);

      await tester.tap(inSheet('Work'));
      await tester.pumpAndSettle();
      final (ids, change) = blocs.library.changesMade.single;
      expect(ids, <String>[MockIds.lisbon, MockIds.readingList]);
      expect(change, const NoteChange.move(notebookId: MockIds.work));
      expect(find.byType(MoveSheet), findsNothing);
      handle.dispose();
    });

    testWidgets('a notebook that holds the note is checked, and a locked one '
        'cannot be chosen', (tester) async {
      final library = FixtureLibraryRepository()
        ..notebookRows = <NotebookRow>[
          ...mockNotebooks,
          const NotebookRow(
            id: 'book:vault',
            name: 'Vault',
            count: 1,
            locked: true,
            depth: 0, hasChildren: false,
          ),
        ];
      final blocs = await BrowseBlocs.start(library);
      final list = await blocs.listOf(const ListPlace.all());
      final handle = tester.ensureSemantics();
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Lisbon in spring'));
      await tester.pump();
      await tester.tap(find.bySemanticsLabel('Move'));
      await tester.pumpAndSettle();
      expect(
        tester.getSemantics(inSheet('Travel')),
        isSemantics(isChecked: true, label: 'Travel'),
      );
      await tester.tap(inSheet('Vault'), warnIfMissed: false);
      await tester.pumpAndSettle();
      expect(blocs.library.changesMade, isEmpty);
      expect(find.byType(MoveSheet), findsOneWidget);
      handle.dispose();
    });

    testWidgets('has nothing to pin or trash in the trash', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.trash());
      final handle = tester.ensureSemantics();
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Lisbon in spring'));
      await tester.pump();
      expect(find.bySemanticsLabel('Move'), findsOneWidget);
      expect(find.bySemanticsLabel('Pin'), findsNothing);
      expect(find.bySemanticsLabel('Trash'), findsNothing);
      handle.dispose();
    });

    testWidgets('says why a change did not happen, and keeps the choice', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      blocs.library.changeResult = const Left<ApplicationError, Changed>(
        ApplicationError.locked(message: 'Locked.'),
      );
      final list = await blocs.listOf(const ListPlace.all());
      final handle = tester.ensureSemantics();
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Lisbon in spring'));
      await tester.pump();
      await tester.tap(find.bySemanticsLabel('Trash'));
      await tester.pump();
      await tester.pump();
      expect(
        find.text("Locked notebooks can't take or give up notes here yet."),
        findsOneWidget,
      );
      expect(find.byType(KansoSelectionBar), findsOneWidget);
      await tester.tap(find.text('OK'));
      await tester.pump();
      expect(find.textContaining('Locked notebooks'), findsNothing);
      handle.dispose();
    });
  });

  group('swiping a row', () {
    testWidgets('a full swipe towards the end pins the note', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await _pumpList(tester, blocs, list);
      await tester.drag(find.text('Sourdough starter'), const Offset(300, 0));
      await tester.pumpAndSettle();
      final (ids, change) = blocs.library.changesMade.single;
      expect(ids, <String>[MockIds.sourdough]);
      expect(change, const NoteChange.togglePin());
      // A bump when the swipe reached the line, and one for the pin.
      expect(blocs.haptics.impacts, 2);
    });

    testWidgets('a full swipe the other way trashes it', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await _pumpList(tester, blocs, list);
      await tester.drag(find.text('Sourdough starter'), const Offset(-300, 0));
      await tester.pumpAndSettle();
      final (ids, change) = blocs.library.changesMade.single;
      expect(ids, <String>[MockIds.sourdough]);
      expect(change, const NoteChange.trash());
      expect(blocs.haptics.impacts, 1);
    });

    testWidgets('a half swipe shows Move and Trash, and Move asks where', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await _pumpList(tester, blocs, list);
      await tester.drag(find.text('Sourdough starter'), const Offset(-120, 0));
      await tester.pumpAndSettle();
      expect(blocs.library.changesMade, isEmpty);
      await tester.tap(find.text('Move'));
      await tester.pumpAndSettle();
      expect(find.byType(MoveSheet), findsOneWidget);
      await tester.tap(inSheet('Garden'));
      await tester.pumpAndSettle();
      final (ids, change) = blocs.library.changesMade.single;
      expect(ids, <String>[MockIds.sourdough]);
      expect(change, const NoteChange.move(notebookId: MockIds.garden));
    });

    testWidgets('a pinned note offers Unpin', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await _pumpList(tester, blocs, list);
      await tester.drag(find.text('Weekly review'), const Offset(100, 0));
      await tester.pumpAndSettle();
      expect(find.text('Unpin'), findsOneWidget);
      await tester.tap(find.text('Unpin'));
      await tester.pumpAndSettle();
      expect(blocs.library.changesMade.single.$1, <String>[
        MockIds.weeklyReview,
      ]);
    });

    testWidgets('in the trash a swipe only offers Move', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.trash());
      await _pumpList(tester, blocs, list);
      await tester.drag(find.text('Sourdough starter'), const Offset(300, 0));
      await tester.pumpAndSettle();
      expect(blocs.library.changesMade, isEmpty);
      await tester.drag(find.text('Sourdough starter'), const Offset(-300, 0));
      await tester.pumpAndSettle();
      // A full swipe does not trash what is in the trash.
      expect(blocs.library.changesMade, isEmpty);
      expect(find.widgetWithText(KansoSwipeActionTile, 'Move'), findsOneWidget);
      expect(find.widgetWithText(KansoSwipeActionTile, 'Trash'), findsNothing);
    });

    testWidgets('a locked note can be swiped like any other', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await _pumpList(tester, blocs, list);
      await tester.drag(find.text('Private journal'), const Offset(300, 0));
      await tester.pumpAndSettle();
      expect(blocs.library.changesMade.single.$1, <String>[
        MockIds.privateJournal,
      ]);
      expect(find.textContaining(mockSecret), findsNothing);
    });

    testWidgets('there are no swipes while choosing', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await _pumpList(tester, blocs, list);
      await tester.longPress(find.text('Sync design notes'));
      await tester.pump();
      await tester.drag(find.text('Sourdough starter'), const Offset(-300, 0));
      await tester.pumpAndSettle();
      expect(blocs.library.changesMade, isEmpty);
    });
  });

  group('choosing and swiping pictures', () {
    for (final brightness in Brightness.values) {
      final name = brightness.name;

      testWidgets('several chosen, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(_scaffold(const NoteListPane()), list: list),
            name: 'phone_note_list_selecting_$name',
            size: phoneSize,
            brightness: brightness,
            interact: (tester) async {
              if (!list.state.selecting) {
                await tester.longPress(find.text('Sync design notes'));
                await tester.pumpAndSettle();
                await tester.tap(find.text('Sourdough starter'));
                await tester.pumpAndSettle();
              }
            },
          ),
        );
      });

      testWidgets('the notebooks to move to, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(_scaffold(const NoteListPane()), list: list),
            name: 'phone_note_list_move_$name',
            size: phoneSize,
            brightness: brightness,
            interact: (tester) async {
              if (!list.state.selecting) {
                await tester.longPress(find.text('Lisbon in spring'));
                await tester.pumpAndSettle();
              }
              await tester.tap(find.bySemanticsLabel('Move'));
              await tester.pumpAndSettle();
            },
          ),
        );
      });

      testWidgets('a swipe towards the end, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(_scaffold(const NoteListPane()), list: list),
            name: 'phone_note_list_swipe_start_$name',
            size: phoneSize,
            brightness: brightness,
            interact: (tester) async {
              await tester.drag(
                find.text('Sourdough starter'),
                const Offset(120, 0),
              );
            },
          ),
        );
      });

      testWidgets('a swipe the other way, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(_scaffold(const NoteListPane()), list: list),
            name: 'phone_note_list_swipe_end_$name',
            size: phoneSize,
            brightness: brightness,
            interact: (tester) async {
              await tester.drag(
                find.text('Sourdough starter'),
                const Offset(-120, 0),
              );
            },
          ),
        );
      });
    }
  });
}
