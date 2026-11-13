// Kanso — board_property_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/board/views/widgets/board_pane.dart';
import 'package:kanso/board/views/widgets/board_settings_sheet.dart';
import 'package:kanso/core/bridge/api/types.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/fixture_library_repository.dart';
import '../../helpers/library_fixtures.dart';
import '../../helpers/page_fixtures.dart';
import '../../helpers/pump_browse_app.dart';

const Size _phone = Size(393, 852);
const Size _ipad = Size(1180, 820);
const String _board = '/notebooks/book%3Aprojects/board';

PageFixtures _studio() => PageFixtures()..boards.studio = true;

Future<BrowseApp> _pump(
  WidgetTester tester,
  PageFixtures fixtures, {
  Size size = _phone,
  FixtureLibraryRepository? library,
}) => atMockNow(
  () => pumpBrowseApp(
    tester,
    size,
    at: _board,
    fixtures: fixtures,
    library: library,
  ),
);

Finder _inBoard(String label) =>
    find.descendant(of: find.byType(BoardPane), matching: find.text(label));

Finder _inSettings(String label) => find.descendant(
  of: find.byType(BoardSettingsSheet),
  matching: find.text(label),
);

Future<void> _openSettings(WidgetTester tester) async {
  await tester.tap(find.bySemanticsLabel('Board settings'));
  await tester.pumpAndSettle();
}

/// The text fields of the lanes in the sheet, in order.
Finder get _laneFields => find.descendant(
  of: find.byType(BoardSettingsSheet),
  matching: find.byType(EditableText),
);

void main() {
  group('a notebook board grouped by a property', () {
    testWidgets('has a lane for each value, worded by the core', (
      tester,
    ) async {
      await _pump(tester, _studio(), size: _ipad);
      expect(find.bySemanticsLabel('Idea, 1 card'), findsOneWidget);
      // The lanes scroll sideways beside the list.
      for (final lane in <String>[
        'Idea',
        'Scripting',
        'Shooting',
        // The lane of cards with no value is the core's own words.
        'No stage',
      ]) {
        await tester.dragUntilVisible(
          _inBoard(lane),
          find.byType(Scrollable).first,
          const Offset(-200, 0),
        );
        expect(_inBoard(lane), findsOneWidget);
      }
      expect(find.text('Weekly review'), findsOneWidget);
      // None of the status lanes.
      expect(find.text('Active'), findsNothing);
      expect(find.text('On hold'), findsNothing);
    });

    testWidgets('the hint says what a touch and hold changes', (tester) async {
      await _pump(tester, _studio());
      expect(
        find.text('Touch and hold a card to change its stage.'),
        findsOneWidget,
      );
      expect(_inBoard('Idea'), findsOneWidget);
    });

    testWidgets('a board with no setting has no settings button', (
      tester,
    ) async {
      await atMockNow(
        () => pumpBrowseApp(
          tester,
          _phone,
          at: _board,
          fixtures: PageFixtures(),
        ),
      );
      expect(find.bySemanticsLabel('Board settings'), findsNothing);
      expect(find.text('Active'), findsOneWidget);
    });

    testWidgets('the tablet drags a card to a lane and says so', (
      tester,
    ) async {
      await _pump(tester, _studio(), size: _ipad);
      expect(
        find.text('Drag a card to change its stage.'),
        findsOneWidget,
      );
    });

    group('Move to…', () {
      testWidgets('lists the lanes and writes the one chosen', (tester) async {
        final fixtures = _studio();
        final library = FixtureLibraryRepository();
        await _pump(tester, fixtures, library: library);
        await tester.longPress(find.text('Weekly review'));
        await tester.pumpAndSettle();
        expect(find.text('Move to'), findsOneWidget);
        // Every lane of the board, the one the card is in checked.
        for (final lane in <String>[
          'Idea',
          'Scripting',
          'Shooting',
          'No stage',
        ]) {
          expect(find.text(lane), findsWidgets);
        }
        await tester.tap(find.text('Shooting').last);
        await tester.pumpAndSettle();
        expect(find.text('Move to'), findsNothing);
        expect(fixtures.boards.moves.single, (
          const CardKey.note(id: MockIds.weeklyReview),
          const BoardLane.value(value: 'Shooting'),
        ));
        // The board was read again with the card in its new lane.
        expect(fixtures.boards.laneCards['Shooting'], hasLength(2));
        expect(find.text('Moved to Shooting.'), findsOneWidget);
        // The notice takes the move back.
        await tester.tap(find.text('Undo'));
        await tester.pumpAndSettle();
        expect(library.undone, <int>[5]);
        expect(find.text('Moved to Shooting.'), findsNothing);
      });

      testWidgets('choosing the lane it is in moves nothing', (tester) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await tester.longPress(find.text('Weekly review'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Idea').last);
        await tester.pumpAndSettle();
        expect(fixtures.boards.moves, isEmpty);
      });

      testWidgets('can move a card to the lane of cards with no value', (
        tester,
      ) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await tester.longPress(find.text('Weekly review'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('No stage').last);
        await tester.pumpAndSettle();
        expect(fixtures.boards.moves.single.$2, const BoardLane.unset());
        expect(find.text('Moved to No stage.'), findsOneWidget);
      });
    });

    group('settings', () {
      testWidgets('open from the more button, with the group choices', (
        tester,
      ) async {
        await _pump(tester, _studio());
        await _openSettings(tester);
        expect(find.byType(BoardSettingsSheet), findsOneWidget);
        expect(_inSettings('Board settings'), findsOneWidget);
        expect(_inSettings('Group by'), findsOneWidget);
        expect(_inSettings('Status'), findsOneWidget);
        expect(_inSettings('stage'), findsOneWidget);
        // The lanes, each in a field of its own.
        expect(_inSettings('Lanes'), findsOneWidget);
        expect(_laneFields, findsNWidgets(4)); // three lanes and the new one
        expect(
          find.descendant(
            of: find.byType(BoardSettingsSheet),
            matching: find.text('Idea'),
          ),
          findsOneWidget,
        );
        expect(_inSettings('Show cards without a stage'), findsOneWidget);
        await tester.tap(_inSettings('Done'));
        await tester.pumpAndSettle();
        expect(find.byType(BoardSettingsSheet), findsNothing);
      });

      testWidgets('group by Status hides the lanes, a property brings '
          'them back', (tester) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.tap(_inSettings('Status'));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, <String>['group -']);
        expect(_inSettings('Lanes'), findsNothing);
        expect(
          _inSettings(
            'Pick a select property to lay the notes out in a lane for '
            'each of its values.',
          ),
          findsOneWidget,
        );
        // The board behind is by status now.
        await tester.tap(_inSettings('Done'));
        await tester.pumpAndSettle();
        expect(find.text('Active'), findsOneWidget);

        await _openSettings(tester);
        await tester.tap(_inSettings('stage'));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls.last, 'group stage');
        expect(_inSettings('Lanes'), findsOneWidget);
      });

      testWidgets('show cards without a value is a switch', (tester) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.tap(_inSettings('Show cards without a stage'));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, <String>['show false']);
        await tester.tap(_inSettings('Show cards without a stage'));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls.last, 'show true');
      });

      testWidgets('renaming a lane writes it and says what it did', (
        tester,
      ) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.enterText(_laneFields.at(2), 'Filming');
        await tester.testTextInput.receiveAction(TextInputAction.done);
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, <String>[
          'rename Shooting>Filming',
        ]);
        expect(fixtures.boards.lanes, <String>['Idea', 'Scripting', 'Filming']);
        // Once, though the field then loses focus.
        await tester.tap(_inSettings('Lanes'));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, hasLength(1));
        expect(_inSettings('Filming'), findsOneWidget);
      });

      testWidgets('a refused rename says why and puts the name back', (
        tester,
      ) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.enterText(_laneFields.at(2), 'Idea');
        await tester.testTextInput.receiveAction(TextInputAction.done);
        await tester.pumpAndSettle();
        expect(
          find.text('There is a lane called Idea already.'),
          findsOneWidget,
        );
        expect(fixtures.boards.lanes, <String>[
          'Idea',
          'Scripting',
          'Shooting',
        ]);
        expect(
          tester.widget<EditableText>(_laneFields.at(2)).controller.text,
          'Shooting',
        );
      });

      testWidgets('reordering a lane moves it', (tester) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await _openSettings(tester);
        // The handle of the last lane, dragged above the first.
        final handle = find.bySemanticsLabel('Move Shooting');
        await tester.drag(handle, const Offset(0, -220));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, <String>['move 2>0']);
        expect(fixtures.boards.lanes.first, 'Shooting');
      });

      testWidgets('a screen reader can move a lane up or down', (
        tester,
      ) async {
        final semantics = tester.ensureSemantics();
        await _pump(tester, _studio());
        await _openSettings(tester);
        final node = tester.getSemantics(
          find.byKey(const ValueKey<String>('lane:Scripting')),
        );
        final actions = <String>[
          for (final id in node.getSemanticsData().customSemanticsActionIds!)
            CustomSemanticsAction.getAction(id)!.label!,
        ];
        expect(actions, containsAll(<String>['Move up', 'Move down']));
        semantics.dispose();
      });

      testWidgets('adding a lane puts it at the end, and clears the field', (
        tester,
      ) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.enterText(_laneFields.last, 'Editing');
        await tester.tap(find.bySemanticsLabel('Add lane'));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, <String>['add Editing']);
        expect(
          fixtures.boards.lanes,
          <String>['Idea', 'Scripting', 'Shooting', 'Editing'],
        );
        expect(
          tester.widget<EditableText>(_laneFields.last).controller.text,
          isEmpty,
        );
        // Nothing typed, nothing added.
        await tester.tap(find.bySemanticsLabel('Add lane'));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, hasLength(1));
      });

      testWidgets('a refused add shows the core’s words', (tester) async {
        final fixtures = _studio()
          ..boards.refusal = 'There is a lane called Idea already.';
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.enterText(_laneFields.last, 'Idea');
        await tester.tap(find.bySemanticsLabel('Add lane'));
        await tester.pumpAndSettle();
        expect(
          find.text('There is a lane called Idea already.'),
          findsOneWidget,
        );
      });

      testWidgets('removing an empty lane needs no question', (tester) async {
        final fixtures = _studio();
        fixtures.boards.laneCards['Scripting'] = <BoardCard>[];
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.tap(find.bySemanticsLabel('Remove Scripting'));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, <String>['remove Scripting>-']);
        expect(fixtures.boards.lanes, <String>['Idea', 'Shooting']);
      });

      testWidgets('removing a lane with notes asks where they go', (
        tester,
      ) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.tap(find.bySemanticsLabel('Remove Scripting'));
        await tester.pumpAndSettle();
        // Nothing is written until it is answered.
        expect(fixtures.boards.settingsCalls, isEmpty);
        expect(
          find.text('1 note is in Scripting. Move it to:'),
          findsOneWidget,
        );
        // The next lane is the default; another can be chosen, or no value.
        await tester.tap(_inSettings('Idea').last);
        await tester.pumpAndSettle();
        await tester.tap(find.text('Remove').last);
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, <String>[
          'remove Scripting>Idea',
        ]);
        expect(fixtures.boards.lanes, <String>['Idea', 'Shooting']);
        expect(find.text('Removed Scripting.'), findsOneWidget);
      });

      testWidgets('a lane’s notes can be left with no value', (tester) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.tap(find.bySemanticsLabel('Remove Scripting'));
        await tester.pumpAndSettle();
        await tester.tap(_inSettings('No stage').last);
        await tester.pumpAndSettle();
        await tester.tap(find.text('Remove').last);
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, <String>['remove Scripting>-']);
      });

      testWidgets('cancelling the question removes nothing', (tester) async {
        final fixtures = _studio();
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.tap(find.bySemanticsLabel('Remove Scripting'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Cancel').last);
        await tester.pumpAndSettle();
        expect(find.text('1 note is in Scripting. Move it to:'), findsNothing);
        expect(fixtures.boards.settingsCalls, isEmpty);
        expect(fixtures.boards.lanes, hasLength(3));
      });

      testWidgets('the only lane cannot be removed', (tester) async {
        final fixtures = _studio();
        fixtures.boards.lanes = <String>['Only'];
        await _pump(tester, fixtures);
        await _openSettings(tester);
        await tester.tap(find.bySemanticsLabel('Remove Only'));
        await tester.pumpAndSettle();
        expect(fixtures.boards.settingsCalls, isEmpty);
      });

      testWidgets('a change can be undone from the notice', (tester) async {
        final library = FixtureLibraryRepository();
        final fixtures = _studio();
        await _pump(tester, fixtures, library: library);
        await _openSettings(tester);
        await tester.enterText(_laneFields.last, 'Editing');
        await tester.tap(find.bySemanticsLabel('Add lane'));
        await tester.pumpAndSettle();
        await tester.tap(_inSettings('Done'));
        await tester.pumpAndSettle();
        expect(find.text('Added the lane Editing.'), findsOneWidget);
        await tester.tap(find.text('Undo'));
        await tester.pumpAndSettle();
        expect(library.undone, <int>[9]);
      });
    });
  });
}
