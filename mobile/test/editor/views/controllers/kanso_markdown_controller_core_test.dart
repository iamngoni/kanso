// Kanso — kanso_markdown_controller_core_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/editor/utils/editor_timing.dart';

import '../../../helpers/editor_harness.dart';
import '../../../helpers/fake_editor_document.dart';

TextEditingValue _at(String text, int caret) => TextEditingValue(
  text: text,
  selection: TextSelection.collapsed(offset: caret),
);

EditorRun _run(int start, int end, {int? color}) => EditorRun(
  range: Range16(start: start, end: end),
  color: color,
  quiet: false,
);

Future<void> _settle(WidgetTester tester) async {
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 10)),
  );
  await tester.pump();
}

const Completion _offered = Completion(
  kind: CompletionKind.wiki,
  replace: Range16(start: 0, end: 3),
  items: <CompletionItem>[
    CompletionItem(label: 'Plan', insert: '[[Plan]]', newNote: false),
    CompletionItem(label: 'Plot', insert: '[[Plot]]', newNote: false),
  ],
);

void main() {
  group('KansoMarkdownController on a document', () {
    testWidgets('tells the document each change against the version it has', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: 'ab');
      final controller = await pumpFakeEditor(tester, document);
      tester.testTextInput.updateEditingValue(_at('aXb', 2));
      await tester.pump();
      final change = document.applied.single;
      expect(change.baseVersion, 1);
      expect((change.start, change.end, change.text), (1, 1, 'X'));
      expect(change.selection, const Range16(start: 2, end: 2));
      expect(change.composing, isNull);
      expect(controller.version, 2);
      tester.testTextInput.updateEditingValue(_at('aXbY', 4));
      await tester.pump();
      expect(document.applied.last.baseVersion, 2);
    });

    testWidgets('tells the document each selection, and what composes', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: 'abc');
      final contexts = <EditorContext>[];
      await pumpFakeEditor(tester, document, onContext: contexts.add);
      tester.testTextInput.updateEditingValue(_at('abc', 1));
      await tester.pump();
      expect(document.selections.last.$1, const Range16(start: 1, end: 1));
      expect(document.selections.last.$2, isFalse);
      expect(contexts, isNotEmpty);
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'abcn',
          selection: TextSelection.collapsed(offset: 4),
          composing: TextRange(start: 3, end: 4),
        ),
      );
      await tester.pump();
      expect(document.applied.last.composing, const Range16(start: 3, end: 4));
      expect(document.selections.last.$2, isTrue);
    });

    testWidgets('drops colours made for a text that has changed since', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: '# a');
      final controller = await pumpFakeEditor(tester, document);
      await _settle(tester);
      // The next batch is for an older text.
      document
        ..batchVersion = 0
        ..batchRuns = <EditorRun>[_run(0, 3, color: 0xFF0000)];
      tester.testTextInput.updateEditingValue(_at('# ab', 4));
      await tester.pump();
      await _settle(tester);
      expect(
        controller.runs.where((run) => run.color == 0xFF0000),
        isEmpty,
        reason: 'the stale batch is not shown',
      );
      // It asked again, and the answer for the text as it is was kept.
      document.batchRuns = <EditorRun>[_run(0, 4, color: 0x00FF00)];
      tester.testTextInput.updateEditingValue(_at('# abc', 5));
      await tester.pump();
      await _settle(tester);
      await _settle(tester);
      expect(controller.runs.single.color, 0x00FF00);
    });

    testWidgets('shows colours for a text a few keys old, moved along them', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: '# a');
      final controller = await pumpFakeEditor(tester, document);
      await _settle(tester);
      final shown = <List<EditorRun>>[];
      controller.addListener(() => shown.add(controller.runs));
      final fresh = <int>[];
      EditorTiming.observer = (name, micros) {
        if (name == 'kanso.freshRuns') {
          fresh.add(micros);
        }
      };
      addTearDown(() => EditorTiming.observer = null);
      // Two keys, typed before the colours of the text as it was after the
      // first have come back.
      document
        ..batchVersion = 2
        ..batchRuns = <EditorRun>[_run(0, 4, color: 0xFF0000)];
      tester.testTextInput.updateEditingValue(_at('# ab', 4));
      tester.testTextInput.updateEditingValue(_at('# abc', 5));
      await tester.pump();
      await _settle(tester);
      final moved = shown.where(
        (runs) => runs.any((run) => run.color == 0xFF0000),
      );
      expect(moved, isNotEmpty, reason: 'colours of the older text are shown');
      expect(
        moved.first.single.range,
        const Range16(start: 0, end: 5),
        reason: 'moved along the key typed since',
      );
      // The first key is fresh with those colours, the second with the ones
      // asked for next.
      await _settle(tester);
      expect(fresh, hasLength(2));
      expect(document.runRequests, greaterThan(2));
    });

    testWidgets('colours for a text before a window moved are dropped', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: '# a');
      final controller = await pumpFakeEditor(tester, document);
      await _settle(tester);
      tester.testTextInput.updateEditingValue(_at('# ab', 4));
      // The text comes from outside: the keys before it cannot be followed.
      document
        ..text = 'other'
        ..version += 5;
      controller.reload(document.current);
      document
        ..batchVersion = 2
        ..batchRuns = <EditorRun>[_run(0, 4, color: 0xFF0000)];
      await tester.pump();
      await _settle(tester);
      expect(controller.runs.where((run) => run.color == 0xFF0000), isEmpty);
    });

    testWidgets('asks for colours once for a burst of keystrokes in a frame', (
      tester,
    ) async {
      final document = FakeEditorDocument();
      await pumpFakeEditor(tester, document);
      await _settle(tester);
      final before = document.runRequests;
      for (var i = 1; i <= 5; i++) {
        tester.testTextInput.updateEditingValue(_at('a' * i, i));
      }
      await tester.pump();
      await _settle(tester);
      await _settle(tester);
      expect(document.runRequests - before, lessThanOrEqualTo(2));
      expect(document.runRequests - before, greaterThanOrEqualTo(1));
    });

    testWidgets('takes the text of the document when it refused a change', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: 'base');
      final controller = await pumpFakeEditor(tester, document);
      // Rust has a newer text than the field had.
      document
        ..text = 'newer'
        ..version = 5
        ..failNextApply = const ApplicationError.invalid(
          message: 'The text changed meanwhile.',
        );
      tester.testTextInput.updateEditingValue(_at('base!', 5));
      await tester.pump();
      await _settle(tester);
      await _settle(tester);
      expect(controller.text, 'newer');
      expect(controller.version, 5);
    });

    testWidgets('tells when the note is locked and keeps the text it had', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: 'base');
      var locked = 0;
      final controller = await pumpFakeEditor(
        tester,
        document,
        onLocked: () => locked++,
      );
      document.failNextApply = const ApplicationError.locked(message: 'x');
      tester.testTextInput.updateEditingValue(_at('base!', 5));
      await tester.pump();
      expect(locked, 1);
      expect(controller.text, 'base');
    });

    testWidgets('tells of an error that is neither', (tester) async {
      final document = FakeEditorDocument(text: 'base');
      final failures = <ApplicationError>[];
      await pumpFakeEditor(tester, document, onFailure: failures.add);
      document.failNextApply = const ApplicationError.storage(message: 'disk');
      tester.testTextInput.updateEditingValue(_at('base!', 5));
      await tester.pump();
      expect(failures.single, isA<StorageError>());
    });

    testWidgets('applies a replacement from the document in the same turn', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: '- one');
      final edits = <int>[];
      final controller = await pumpFakeEditor(
        tester,
        document,
        onEdited: () => edits.add(1),
      );
      document.nextOutcome = const EditOutcome(
        version: 9,
        replace: Replacement(start: 5, end: 5, text: '\n- '),
        selection: Range16(start: 8, end: 8),
        history: HistoryState(canUndo: true, canRedo: false),
      );
      controller.selection = const TextSelection.collapsed(offset: 5);
      tester.testTextInput.updateEditingValue(_at('- one\n', 6));
      await tester.pump();
      expect(controller.text, '- one\n- ');
      expect(controller.selection.baseOffset, 8);
      expect(controller.version, 9);
      expect(controller.history.canUndo, isTrue);
      // Return was offered as a command and never applied as typing.
      expect(document.commands.single, const EditCommand.enter(plain: false));
      expect(document.applied, isEmpty);
      expect(edits, isNotEmpty);
    });

    testWidgets('types the newline when the document has no command for it', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: 'one');
      final controller = await pumpFakeEditor(tester, document);
      tester.testTextInput.updateEditingValue(_at('one\n', 4));
      await tester.pump();
      expect(document.commands.single, const EditCommand.enter(plain: false));
      expect(document.applied.single.text, '\n');
      expect(controller.text, 'one\n');
    });

    testWidgets('Return chooses the completion that is offered', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: '[[Pl');
      final controller = await pumpFakeEditor(tester, document);
      controller
        ..selection = const TextSelection.collapsed(offset: 4)
        ..completion = const Completion(
          kind: CompletionKind.wiki,
          replace: Range16(start: 0, end: 4),
          items: <CompletionItem>[
            CompletionItem(label: 'Plan', insert: '[[Plan]]', newNote: false),
            CompletionItem(label: 'Plot', insert: '[[Plot]]', newNote: false),
          ],
        )
        ..completionIndex = 1;
      document.nextOutcome = const EditOutcome(
        version: 3,
        replace: Replacement(start: 0, end: 4, text: '[[Plot]]'),
        selection: Range16(start: 8, end: 8),
        history: HistoryState(canUndo: true, canRedo: false),
      );
      tester.testTextInput.updateEditingValue(_at('[[Pl\n', 5));
      await tester.pump();
      expect(
        document.commands.single,
        const EditCommand.insert(
          range: Range16(start: 0, end: 4),
          text: '[[Plot]]',
        ),
      );
      expect(controller.text, '[[Plot]]');
      expect(controller.completion, isNull);
    });

    testWidgets('undo, redo and a command say false for nothing to do', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: 'x');
      final controller = await pumpFakeEditor(tester, document);
      expect(controller.undo(), isFalse);
      expect(controller.redo(), isFalse);
      expect(controller.run(const EditCommand.cycleHeading()), isFalse);
      expect(document.calls, containsAll(<String>['undo', 'redo']));
      expect(controller.text, 'x');
    });

    testWidgets('a text from outside keeps the caret as far as it reaches', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: 'a longer text');
      final controller = await pumpFakeEditor(tester, document);
      controller.selection = const TextSelection.collapsed(offset: 10);
      await tester.pump();
      document
        ..text = 'short'
        ..version = 4;
      controller.reload(document.current);
      await tester.pump();
      expect(controller.text, 'short');
      expect(controller.selection.baseOffset, 5);
      expect(controller.version, 4);
    });

    testWidgets('moves through the completions within them', (tester) async {
      final document = FakeEditorDocument(text: '[[P');
      final controller = await pumpFakeEditor(tester, document);
      var changes = 0;
      controller
        ..addListener(() => changes++)
        ..stepCompletion(1);
      expect(controller.completionIndex, 0, reason: 'nothing offered');
      expect(controller.chooseCompletion(), isFalse);
      controller.offer(_offered);
      expect(changes, 1);
      controller
        ..stepCompletion(1)
        ..stepCompletion(5);
      expect(controller.completionIndex, 1);
      controller.stepCompletion(-9);
      expect(controller.completionIndex, 0);
      // An offer that is the same changes nothing.
      changes = 0;
      controller.offer(_offered);
      expect(changes, 0);
      // A new offer starts at its first row.
      controller
        ..stepCompletion(1)
        ..offer(null);
      expect(controller.completion, isNull);
      expect(controller.completionIndex, 0);
    });

    testWidgets('chooses the active completion as one step, and it is spent', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: '[[P');
      final controller = await pumpFakeEditor(tester, document);
      controller
        ..offer(_offered)
        ..stepCompletion(1);
      document.nextOutcome = const EditOutcome(
        version: 5,
        replace: Replacement(start: 0, end: 3, text: '[[Plot]]'),
        selection: Range16(start: 8, end: 8),
        history: HistoryState(canUndo: true, canRedo: false),
      );
      expect(controller.chooseCompletion(), isTrue);
      expect(controller.text, '[[Plot]]');
      expect(controller.completion, isNull);
      expect(controller.chooseCompletion(), isFalse);
    });

    testWidgets('knows which text is code and is not spell checked', (
      tester,
    ) async {
      final document = FakeEditorDocument(text: 'a `code` b')
        ..batchRuns = const <EditorRun>[
          EditorRun(range: Range16(start: 2, end: 8), quiet: true),
        ];
      final controller = await pumpFakeEditor(tester, document);
      await _settle(tester);
      expect(controller.isQuietAt(1), isFalse);
      expect(controller.isQuietAt(2), isTrue);
      expect(controller.isQuietAt(7), isTrue);
      expect(controller.isQuietAt(8), isFalse);
    });
  });
}
