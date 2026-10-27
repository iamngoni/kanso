// Kanso — editor_semantics_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui' show Tristate;

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/editor/views/widgets/accessory_key.dart';
import 'package:kanso/editor/views/widgets/completion_popup.dart';
import 'package:kanso/editor/views/widgets/editor_conflict_strip.dart';
import 'package:kanso/editor/views/widgets/editor_failure_strip.dart';
import 'package:kanso/editor/views/widgets/keyboard_accessory_bar.dart';

import '../../../helpers/editor_harness.dart';
import '../../../helpers/pump.dart';

const _anywhere = EditorContext(
  inList: false,
  inTable: false,
  inCode: false,
  completionHere: false,
  history: HistoryState(canUndo: true, canRedo: true),
);

/// What VoiceOver and TalkBack are given of the editor's parts.
void main() {
  group('the editor for a screen reader', () {
    testWidgets('the text is a multiline text field with its value and '
        'selection', (tester) async {
      final handle = tester.ensureSemantics();
      final editor = await pumpEditor(tester, text: 'Hello\nworld');
      await editor.settle(tester);
      editor.controller.selection = const TextSelection(
        baseOffset: 2,
        extentOffset: 5,
      );
      await tester.pump();
      final data = find.semantics
          .byPredicate((node) => node.flagsCollection.isTextField)
          .evaluate()
          .single
          .getSemanticsData();
      expect(data.flagsCollection.isTextField, isTrue);
      expect(data.flagsCollection.isMultiline, isTrue);
      expect(data.flagsCollection.isFocused, Tristate.isTrue);
      expect(data.value, 'Hello\nworld');
      expect(data.textSelection?.baseOffset, 2);
      expect(data.textSelection?.extentOffset, 5);
      expect(data.hasAction(SemanticsAction.setSelection), isTrue);
      expect(data.hasAction(SemanticsAction.setText), isTrue);
      handle.dispose();
    });

    testWidgets('every key of the keyboard bar has a label and is a button', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await pumpKanso(
        tester,
        SizedBox(
          width: 393,
          child: KeyboardAccessoryBar(
            editorContext: _anywhere,
            onCommand: (_) {},
            onUndo: () {},
            onRedo: () {},
          ),
        ),
      );
      final keys = tester.widgetList<AccessoryKey>(find.byType(AccessoryKey));
      expect(keys, isNotEmpty);
      for (final key in keys) {
        expect(key.semanticLabel, isNotEmpty);
        final node = tester.getSemantics(
          find.byWidgetPredicate(
            (widget) =>
                widget is AccessoryKey &&
                widget.semanticLabel == key.semanticLabel,
          ),
        );
        expect(node.label, key.semanticLabel);
        // A key with nothing to do now is dimmed, and no button.
        expect(
          node.getSemanticsData().flagsCollection.isButton,
          key.onTap != null,
        );
      }
      // No two keys say the same.
      final labels = keys.map((key) => key.semanticLabel).toList();
      expect(labels.toSet(), hasLength(labels.length));
      handle.dispose();
    });

    testWidgets('the undo key offers Redo as an action', (tester) async {
      final handle = tester.ensureSemantics();
      await pumpKanso(
        tester,
        SizedBox(
          width: 393,
          child: KeyboardAccessoryBar(
            editorContext: _anywhere,
            onCommand: (_) {},
            onUndo: () {},
            onRedo: () {},
          ),
        ),
      );
      final undo = tester.getSemantics(
        find.byWidgetPredicate(
          (widget) => widget is AccessoryKey && widget.semanticLabel == 'Undo',
        ),
      );
      final actions = undo.getSemanticsData().customSemanticsActionIds;
      expect(
        actions!.map((id) => CustomSemanticsAction.getAction(id)!.label),
        <String>['Redo'],
      );
      handle.dispose();
    });

    testWidgets('the completions are a live region that names the active row', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await pumpKanso(
        tester,
        SizedBox(
          width: 361,
          child: CompletionPopup(
            completion: Completion(
              kind: CompletionKind.wiki,
              replace: const Range16(start: 0, end: 2),
              items: <CompletionItem>[
                for (final title in <String>['Alpha', 'Beta', 'Gamma'])
                  CompletionItem(
                    label: title,
                    insert: '[[$title]]',
                    newNote: false,
                  ),
              ],
            ),
            activeIndex: 1,
            onChoose: (_) {},
          ),
        ),
      );
      final popup = tester
          .getSemantics(find.byType(CompletionPopup))
          .getSemanticsData();
      expect(popup.flagsCollection.isLiveRegion, isTrue);
      expect(popup.label, allOf(contains('3'), contains('Beta')));
      handle.dispose();
    });

    testWidgets(
      'the strips for a conflict and a failed save are live regions',
      (
        tester,
      ) async {
        final handle = tester.ensureSemantics();
        await pumpKanso(
          tester,
          Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              EditorConflictStrip(
                deleted: false,
                gutter: 16,
                onKeepMine: () {},
                onUseTheirs: () {},
              ),
              EditorConflictStrip(
                deleted: true,
                gutter: 16,
                onKeepMine: () {},
                onUseTheirs: () {},
              ),
              EditorFailureStrip(gutter: 16, onRetry: () {}),
            ],
          ),
        );
        final strips = <Type>[EditorConflictStrip, EditorFailureStrip];
        var live = 0;
        for (final type in strips) {
          for (final element in find.byType(type).evaluate()) {
            final data = tester
                .getSemantics(find.byElementPredicate((e) => e == element))
                .getSemanticsData();
            expect(data.flagsCollection.isLiveRegion, isTrue);
            expect(data.label, isNotEmpty);
            live++;
          }
        }
        expect(live, 3);
        handle.dispose();
      },
    );
  });
}
