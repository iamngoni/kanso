// Kanso — keyboard_accessory_bar_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/editor/views/widgets/accessory_key.dart';
import 'package:kanso/editor/views/widgets/keyboard_accessory_bar.dart';

import '../../../helpers/pump.dart';

const _anywhere = EditorContext(
  inList: false,
  inTable: false,
  inCode: false,
  completionHere: false,
  history: HistoryState(canUndo: true, canRedo: false),
);

/// The key that says [label] to a screen reader.
Finder keyLabelled(String label) => find.byWidgetPredicate(
  (widget) => widget is AccessoryKey && widget.semanticLabel == label,
);

void main() {
  group('KeyboardAccessoryBar', () {
    Future<List<Object>> pumpBar(
      WidgetTester tester, {
      EditorContext? context = _anywhere,
      double width = 393,
    }) async {
      final taps = <Object>[];
      await pumpKanso(
        tester,
        SizedBox(
          width: width,
          child: KeyboardAccessoryBar(
            editorContext: context,
            onCommand: taps.add,
            onUndo: () => taps.add('undo'),
            onRedo: () => taps.add('redo'),
          ),
        ),
      );
      return taps;
    }

    testWidgets('each key asks for its command', (tester) async {
      final taps = await pumpBar(
        tester,
        context: const EditorContext(
          inList: true,
          inTable: false,
          inCode: false,
          completionHere: false,
          history: HistoryState(canUndo: true, canRedo: false),
        ),
      );
      for (final label in <String>[
        'Heading',
        'List',
        'Task',
        'Bold',
        'Link',
        'Image',
        'Link to a note',
        'Indent',
        'Outdent',
        'Undo',
      ]) {
        await tester.tap(keyLabelled(label));
      }
      expect(taps, <Object>[
        const EditCommand.cycleHeading(),
        const EditCommand.format(format: FormatKind.bulleted),
        const EditCommand.format(format: FormatKind.task),
        const EditCommand.format(format: FormatKind.bold),
        const EditCommand.format(format: FormatKind.link),
        const EditCommand.format(format: FormatKind.image),
        const EditCommand.wikiLink(),
        const EditCommand.tab(out: false),
        const EditCommand.tab(out: true),
        'undo',
      ]);
    });

    testWidgets('indent and outdent are dimmed outside a list or a table', (
      tester,
    ) async {
      final taps = await pumpBar(tester);
      await tester.tap(keyLabelled('Indent'));
      await tester.tap(keyLabelled('Outdent'));
      expect(taps, isEmpty);
      final keys = tester.widgetList<AccessoryKey>(find.byType(AccessoryKey));
      expect(
        keys.where((key) => key.onTap == null).map((key) => key.semanticLabel),
        <String>['Indent', 'Outdent'],
      );
    });

    testWidgets('they are there in a list and in a table', (tester) async {
      for (final (inList, inTable) in <(bool, bool)>[
        (true, false),
        (false, true),
      ]) {
        final taps = await pumpBar(
          tester,
          context: EditorContext(
            inList: inList,
            inTable: inTable,
            inCode: false,
            completionHere: false,
            history: const HistoryState(canUndo: false, canRedo: false),
          ),
        );
        await tester.tap(keyLabelled('Indent'));
        expect(taps, <Object>[const EditCommand.tab(out: false)]);
      }
    });

    testWidgets('undo is dimmed when there is nothing to undo', (tester) async {
      final taps = await pumpBar(
        tester,
        context: const EditorContext(
          inList: false,
          inTable: false,
          inCode: false,
          completionHere: false,
          history: HistoryState(canUndo: false, canRedo: true),
        ),
      );
      await tester.tap(keyLabelled('Undo'));
      expect(taps, isEmpty);
    });

    testWidgets('holding undo redoes, even when there is nothing to undo', (
      tester,
    ) async {
      final taps = await pumpBar(tester, context: null);
      await tester.longPress(keyLabelled('Undo'));
      expect(taps, <Object>['redo']);
    });

    testWidgets('a screen reader is offered Redo on the undo key', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      final taps = await pumpBar(tester);
      final node = tester.getSemantics(keyLabelled('Undo'));
      final actions = node.getSemanticsData().customSemanticsActionIds;
      expect(actions, isNotNull);
      final redo = CustomSemanticsAction.getAction(actions!.single);
      expect(redo?.label, 'Redo');
      tester.semantics.performAction(
        find.semantics.byLabel('Undo'),
        SemanticsAction.customAction,
        args: actions.single,
      );
      expect(taps, <Object>['redo']);
      handle.dispose();
    });

    testWidgets('every key is on the screen of a phone, and keeps a 44 point '
        'target', (tester) async {
      await pumpBar(tester);
      final keys = find.byType(AccessoryKey);
      expect(keys, findsNWidgets(10));
      final box = tester.getRect(find.byType(KeyboardAccessoryBar));
      for (final element in keys.evaluate()) {
        final rect = tester.getRect(find.byWidget(element.widget));
        expect(rect.left, greaterThanOrEqualTo(box.left));
        expect(rect.right, lessThanOrEqualTo(box.right));
        expect(rect.height, greaterThanOrEqualTo(44));
        expect(rect.width, greaterThanOrEqualTo(32));
      }
    });
  });
}
