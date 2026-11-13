// Kanso — when_hint_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/editor/models/repos/abs/when_repository.dart';
import 'package:kanso/editor/utils/when_date_order.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/controllers/when_assist.dart';
import 'package:kanso/editor/views/widgets/completion_host.dart';
import 'package:kanso/editor/views/widgets/completion_popup.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';
import 'package:kanso/editor/views/widgets/when_hint_chip.dart';

import '../../../helpers/fake_editor_document.dart';
import '../../../helpers/kanso_test_app.dart';

/// A parser that knows `fri 3pm every week` and the `@` list, and counts
/// what it was asked.
class _FakeWhen implements WhenRepository {
  final List<String> lines = <String>[];
  bool monthFirstSeen = false;

  @override
  WhenHint? hint(String line, {required bool monthFirst}) {
    lines.add(line);
    monthFirstSeen = monthFirst;
    const phrase = 'fri 3pm every week';
    if (!line.startsWith('- [ ] ') || !line.endsWith(phrase)) {
      return null;
    }
    return WhenHint(
      start: line.length - phrase.length,
      end: line.length,
      label: 'Fri 14 Nov, 15:00 · every week',
      canonical: '@2026-11-14T15:00 every week',
      repeats: true,
    );
  }

  @override
  DateChoices? choices(String beforeCaret) {
    final at = beforeCaret.lastIndexOf('@');
    if (!beforeCaret.startsWith('- [ ] ') || at < 0) {
      return null;
    }
    return DateChoices(
      start: at,
      choices: const <DateChoice>[
        DateChoice(
          label: 'Today',
          detail: '2026-11-13',
          insert: '@2026-11-13',
        ),
        DateChoice(
          label: 'Tomorrow · Sat 14 Nov',
          detail: '2026-11-14',
          insert: '@2026-11-14',
        ),
      ],
    );
  }
}

class _Rig {
  _Rig(this.document, this.controller, this.assist, this.focus);

  final FakeEditorDocument document;
  final KansoMarkdownController controller;
  final WhenAssist assist;
  final FocusNode focus;
}

Future<_Rig> _pump(
  WidgetTester tester, {
  String text = '',
  _FakeWhen? when,
  bool Function()? enabled,
}) async {
  tester.view
    ..physicalSize = const Size(393, 700)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final document = FakeEditorDocument(text: text);
  final controller = KansoMarkdownController(
    document: document,
    snapshot: document.current,
    themeId: 'kanso-light',
  );
  final focus = FocusNode();
  final assist = WhenAssist(
    controller: controller,
    repository: when ?? _FakeWhen(),
    monthFirst: false,
    enabled: enabled,
  );
  addTearDown(() {
    assist.dispose();
    focus.dispose();
    controller.dispose();
  });
  final key = GlobalKey<EditableTextState>();
  await tester.pumpWidget(
    KansoTestApp(
      home: Scaffold(
        resizeToAvoidBottomInset: false,
        body: Column(
          children: <Widget>[
            Expanded(
              child: SingleChildScrollView(
                child: CompletionHost(
                  controller: controller,
                  editableKey: key,
                  child: MarkdownEditor(
                    controller: controller,
                    focusNode: focus,
                    editableKey: key,
                  ),
                ),
              ),
            ),
            WhenHintBar(assist: assist),
          ],
        ),
      ),
    ),
  );
  focus.requestFocus();
  await tester.pump();
  return _Rig(document, controller, assist, focus);
}

Future<void> _type(WidgetTester tester, String text, {int? caret}) async {
  tester.testTextInput.updateEditingValue(
    TextEditingValue(
      text: text,
      selection: TextSelection.collapsed(offset: caret ?? text.length),
    ),
  );
  await tester.pump();
  // The hint grows into place.
  await tester.pump(const Duration(milliseconds: 200));
}

void main() {
  group('the typed-date hint', () {
    testWidgets('shows the date for a phrase at the end of a task', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.byType(WhenHintChip), findsNothing);
      await _type(tester, '- [ ] film Pixel review fri 3pm every week');
      expect(find.byType(WhenHintChip), findsOneWidget);
      expect(find.text('Fri 14 Nov, 15:00 · every week'), findsOneWidget);
      expect(find.text('Set'), findsOneWidget);
    });

    testWidgets(
      'has nothing to show for prose, and goes when the phrase does',
      (
        tester,
      ) async {
        await _pump(tester);
        await _type(tester, 'film Pixel review fri 3pm every week');
        expect(find.byType(WhenHintChip), findsNothing);
        await _type(tester, '- [ ] film Pixel review fri 3pm every week');
        expect(find.byType(WhenHintChip), findsOneWidget);
        // Typing on leaves the text alone, and the hint goes with the phrase.
        await _type(tester, '- [ ] film Pixel review fri 3pm every weekend');
        expect(find.byType(WhenHintChip), findsNothing);
      },
    );

    testWidgets('waits for the caret to be at the end of the line', (
      tester,
    ) async {
      await _pump(tester);
      const line = '- [ ] film fri 3pm every week';
      await _type(tester, line, caret: 8);
      expect(find.byType(WhenHintChip), findsNothing);
      await _type(tester, line);
      expect(find.byType(WhenHintChip), findsOneWidget);
    });

    testWidgets('asks about the line the caret is on, not the note', (
      tester,
    ) async {
      final when = _FakeWhen();
      await _pump(tester, when: when);
      await _type(tester, 'first line\n- [ ] edit fri 3pm every week\nlast');
      expect(find.byType(WhenHintChip), findsNothing);
      await _type(
        tester,
        'first line\n- [ ] edit fri 3pm every week\nlast',
        caret: 'first line\n- [ ] edit fri 3pm every week'.length,
      );
      expect(find.byType(WhenHintChip), findsOneWidget);
      expect(when.lines.last, '- [ ] edit fri 3pm every week');
    });

    testWidgets(
      'tapping it writes the canonical token in place of the phrase',
      (
        tester,
      ) async {
        final rig = await _pump(tester);
        const line = 'one\n- [ ] film Pixel review fri 3pm every week';
        await _type(tester, line);
        const phraseStart = 'one\n- [ ] film Pixel review '.length;
        rig.document.nextOutcome = const EditOutcome(
          version: 5,
          replace: Replacement(
            start: phraseStart,
            end: line.length,
            text: '@2026-11-14T15:00 every week',
          ),
          selection: Range16(start: 56, end: 56),
          history: HistoryState(canUndo: true, canRedo: false),
        );
        await tester.tap(find.text('Fri 14 Nov, 15:00 \u00b7 every week'));
        await tester.pump();
        await tester.pump();
        expect(
          rig.document.commands.single,
          const EditCommand.insert(
            range: Range16(start: phraseStart, end: line.length),
            text: '@2026-11-14T15:00 every week',
          ),
        );
        expect(
          rig.controller.text,
          'one\n- [ ] film Pixel review @2026-11-14T15:00 every week',
        );
        expect(find.byType(WhenHintChip), findsNothing);
      },
    );

    testWidgets('nothing is written until it is tapped', (tester) async {
      final rig = await _pump(tester);
      await _type(tester, '- [ ] film Pixel review fri 3pm every week');
      await tester.pump(const Duration(seconds: 2));
      expect(rig.document.commands, isEmpty);
      expect(
        rig.controller.text,
        '- [ ] film Pixel review fri 3pm every week',
      );
    });

    testWidgets('is not offered where the field is not for typing', (
      tester,
    ) async {
      await _pump(tester, enabled: () => false);
      await _type(tester, '- [ ] film Pixel review fri 3pm every week');
      expect(find.byType(WhenHintChip), findsNothing);
    });

    testWidgets('speaks for a screen reader', (tester) async {
      final handle = tester.ensureSemantics();
      await _pump(tester);
      await _type(tester, '- [ ] film Pixel review fri 3pm every week');
      expect(
        find.bySemanticsLabel(RegExp(r'^Fri 14 Nov, 15:00 .* every week\.')),
        findsOneWidget,
      );
      handle.dispose();
    });
  });

  group('the @ date list', () {
    testWidgets('appears after an @ in a task', (tester) async {
      await _pump(tester);
      await _type(tester, '- [ ] ring mum');
      expect(find.byType(CompletionPopup), findsNothing);
      await _type(tester, '- [ ] ring mum @');
      expect(find.byType(CompletionPopup), findsOneWidget);
      expect(find.text('Today'), findsOneWidget);
      expect(find.text('Tomorrow · Sat 14 Nov'), findsOneWidget);
      expect(find.text('2026-11-14'), findsOneWidget);
    });

    testWidgets('tapping a row writes its date in place of the @', (
      tester,
    ) async {
      final rig = await _pump(tester);
      await _type(tester, 'a\n- [ ] ring mum @');
      expect(rig.controller.completion?.kind, CompletionKind.date);
      const at = 'a\n- [ ] ring mum '.length;
      rig.document.nextOutcome = const EditOutcome(
        version: 5,
        replace: Replacement(start: at, end: at + 1, text: '@2026-11-14'),
        selection: Range16(start: at + 11, end: at + 11),
        history: HistoryState(canUndo: true, canRedo: false),
      );
      await tester.tap(find.text('Tomorrow · Sat 14 Nov'));
      await tester.pump();
      await tester.pump();
      expect(
        rig.document.commands.single,
        const EditCommand.insert(
          range: Range16(start: at, end: at + 1),
          text: '@2026-11-14',
        ),
      );
      expect(rig.controller.text, 'a\n- [ ] ring mum @2026-11-14');
    });

    testWidgets('goes when the @ does, and when it is dismissed', (
      tester,
    ) async {
      final rig = await _pump(tester);
      await _type(tester, '- [ ] ring mum @');
      expect(find.byType(CompletionPopup), findsOneWidget);
      await _type(tester, '- [ ] ring mum ');
      expect(find.byType(CompletionPopup), findsNothing);
      await _type(tester, '- [ ] ring mum @');
      rig.assist.dismissCompletion();
      await tester.pump();
      expect(find.byType(CompletionPopup), findsNothing);
      // Moving the caret does not bring it back; typing does.
      await tester.pump();
      expect(find.byType(CompletionPopup), findsNothing);
      await _type(tester, '- [ ] ring mum @t');
      expect(find.byType(CompletionPopup), findsOneWidget);
    });

    testWidgets('does not offer a note-link completion away', (tester) async {
      final rig = await _pump(tester);
      rig.controller.offer(
        const Completion(
          kind: CompletionKind.wiki,
          replace: Range16(start: 0, end: 2),
          items: <CompletionItem>[
            CompletionItem(label: 'Plan', insert: '[[Plan]]', newNote: false),
          ],
        ),
      );
      await _type(tester, '- [ ] see [[Pl @');
      expect(rig.controller.completion?.kind, CompletionKind.wiki);
    });
  });

  group('the date order', () {
    test('is month first where the region writes it so', () {
      expect(monthFirstFor(const Locale('en', 'US')), isTrue);
      expect(monthFirstFor(const Locale('en', 'GB')), isFalse);
      expect(monthFirstFor(const Locale('en')), isFalse);
    });

    testWidgets('is the one the assist was given', (tester) async {
      final when = _FakeWhen();
      await _pump(tester, when: when);
      await _type(tester, '- [ ] a fri');
      expect(when.monthFirstSeen, isFalse);
    });
  });
}
