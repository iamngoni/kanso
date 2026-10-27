// Kanso — completion_popup_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/editor/views/widgets/completion_popup.dart';
import 'package:kanso/search/views/widgets/highlighted_text.dart';

import '../../../helpers/pump.dart';

Completion _completion(int rows, {CompletionKind kind = CompletionKind.wiki}) {
  return Completion(
    kind: kind,
    replace: const Range16(start: 0, end: 2),
    items: <CompletionItem>[
      for (var i = 0; i < rows; i++)
        CompletionItem(
          label: 'Note $i',
          detail: i.isEven ? 'Projects / Kanso' : null,
          highlight: const Range16(start: 0, end: 4),
          insert: '[[Note $i]]',
          newNote: false,
        ),
    ],
  );
}

/// The row that says [title].
Finder rowOf(String title) => find.byWidgetPredicate(
  (widget) => widget is HighlightedText && widget.text == title,
);

void main() {
  group('CompletionPopup', () {
    Future<List<CompletionItem>> pumpPopup(
      WidgetTester tester, {
      required Completion completion,
      int active = 0,
    }) async {
      final chosen = <CompletionItem>[];
      await pumpKanso(
        tester,
        SizedBox(
          width: 361,
          child: CompletionPopup(
            completion: completion,
            activeIndex: active,
            onChoose: chosen.add,
          ),
        ),
      );
      return chosen;
    }

    testWidgets('has a row for each choice, with its title and notebook', (
      tester,
    ) async {
      await pumpPopup(tester, completion: _completion(3));
      expect(find.byType(HighlightedText), findsNWidgets(3));
      expect(find.text('Projects / Kanso'), findsNWidgets(2));
      final box = tester.getSize(find.byType(CompletionPopup));
      expect(box.height, CompletionPopup.heightFor(3));
      expect(CompletionPopup.heightFor(3), 3 * 44 + 2 + 2);
    });

    testWidgets('highlights what was typed', (tester) async {
      await pumpPopup(tester, completion: _completion(1));
      final text = tester.widget<HighlightedText>(find.byType(HighlightedText));
      expect(text.ranges, const <Range16>[Range16(start: 0, end: 4)]);
    });

    testWidgets('shows no highlight where the core names none', (tester) async {
      await pumpPopup(
        tester,
        completion: const Completion(
          kind: CompletionKind.tag,
          replace: Range16(start: 0, end: 1),
          items: <CompletionItem>[
            CompletionItem(label: '#todo', insert: '#todo', newNote: false),
          ],
        ),
      );
      final text = tester.widget<HighlightedText>(find.byType(HighlightedText));
      expect(text.ranges, isEmpty);
      expect(find.text('#todo', findRichText: true), findsOneWidget);
    });

    testWidgets('marks the active row', (tester) async {
      await pumpPopup(tester, completion: _completion(3), active: 1);
      final rows = tester
          .widgetList<Container>(
            find.descendant(
              of: find.byType(CompletionPopup),
              matching: find.byType(Container),
            ),
          )
          .where((box) => box.constraints?.maxHeight == 44)
          .toList();
      expect(rows, hasLength(3));
      expect(rows[0].color, isNull);
      expect(rows[1].color, isNotNull);
      expect(rows[2].color, isNull);
    });

    testWidgets('shows five rows and scrolls for the rest', (tester) async {
      await pumpPopup(tester, completion: _completion(9));
      expect(
        tester.getSize(find.byType(CompletionPopup)).height,
        5 * 44 + 4 + 2,
      );
      expect(rowOf('Note 4'), findsOneWidget);
      expect(rowOf('Note 7'), findsNothing);
      await tester.drag(find.byType(ListView), const Offset(0, -300));
      await tester.pump();
      expect(rowOf('Note 8'), findsOneWidget);
    });

    testWidgets('keeps the active row in view as it moves', (tester) async {
      final completion = _completion(9);
      Future<void> show(int active) async {
        await pumpKanso(
          tester,
          SizedBox(
            width: 361,
            child: CompletionPopup(
              completion: completion,
              activeIndex: active,
              onChoose: (_) {},
            ),
          ),
        );
        await tester.pump();
      }

      await show(0);
      await show(7);
      final list = tester.widget<ListView>(find.byType(ListView));
      expect(list.controller!.offset, greaterThan(0));
      await show(0);
      expect(
        tester.widget<ListView>(find.byType(ListView)).controller!.offset,
        0,
      );
    });

    testWidgets('a tap chooses a row', (tester) async {
      final chosen = await pumpPopup(tester, completion: _completion(3));
      await tester.tap(rowOf('Note 2'));
      expect(chosen.single.insert, '[[Note 2]]');
    });

    testWidgets('a screen reader hears how many there are and the one '
        'selected', (tester) async {
      final handle = tester.ensureSemantics();
      await pumpPopup(tester, completion: _completion(3), active: 1);
      expect(
        find.bySemanticsLabel('3 suggestions, Note 1 selected'),
        findsOneWidget,
      );
      final node = tester.getSemantics(
        find.bySemanticsLabel('3 suggestions, Note 1 selected'),
      );
      expect(node.flagsCollection.isLiveRegion, isTrue);
      handle.dispose();
    });
  });
}
