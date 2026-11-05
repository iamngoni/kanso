// Kanso — sketch_editor_extras_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/canvas/utils/sketch_editor_extras.dart';
import 'package:kanso/core/views/widgets/kanso_feedback.dart';

import '../../helpers/fake_canvas.dart';
import '../../helpers/fake_editor_target.dart';
import '../../helpers/kanso_test_app.dart';

Widget _app(void Function(BuildContext) onTap) => KansoTestApp(
  home: KansoFeedbackHost(
    child: Builder(
      builder: (context) => Center(
        child: TextButton(
          onPressed: () => onTap(context),
          child: const Text('Insert'),
        ),
      ),
    ),
  ),
);

void main() {
  testWidgets('a shared note takes no sketch, and no size is asked for', (
    tester,
  ) async {
    final repository = FakeCanvasRepository()..sharedNotes.add('shared');
    final extras = SketchEditorExtras(repository: repository, label: 'Sketch');
    final target = FakeEditorTarget('Hello', noteId: 'shared');
    await tester.pumpWidget(_app((context) => extras.insert(context, target)));
    await tester.tap(find.text('Insert'));
    await tester.pumpAndSettle();
    expect(
      find.text(
        "Sketches can't be added to a shared note. Add one to a private "
        'note, or draw on a canvas.',
      ),
      findsOneWidget,
    );
    expect(find.text('Medium, 480 by 320'), findsNothing);
    expect(repository.inserts, isEmpty);
    expect(target.value.text, 'Hello');
    expect(target.edits, 0);
  });

  testWidgets('a private note takes the sketch at the caret, one undo step', (
    tester,
  ) async {
    final repository = FakeCanvasRepository();
    final extras = SketchEditorExtras(repository: repository, label: 'Sketch');
    final target = FakeEditorTarget(
      'Hello world',
      selection: const TextSelection.collapsed(offset: 5),
      noteId: 'mine',
    );
    await tester.pumpWidget(_app((context) => extras.insert(context, target)));
    await tester.tap(find.text('Insert'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Medium, 480 by 320'));
    await tester.pumpAndSettle();
    expect(repository.inserts, <String>['mine']);
    expect(target.edits, 1);
    expect(
      target.value.text,
      'Hello\n```sketch\n{"w":480,"h":320}\n```\n world',
    );
  });
}
