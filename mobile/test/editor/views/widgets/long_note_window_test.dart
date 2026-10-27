// Kanso — long_note_window_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/editor/utils/editor_window.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/editor/views/widgets/outside_text.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

import '../../../helpers/editor_fixture.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

/// A note of 400 lines of about 50 units: some 20,000, more than two windows,
/// with a heading every ten lines.
final String longBody = List<String>.generate(
  400,
  (i) => i % 10 == 0
      ? '## Section ${i.toString().padLeft(3, '0')} of the long note'
      : 'Line ${i.toString().padLeft(3, '0')} of a long note, with words.',
).join('\n');

KansoMarkdownController controllerOf(WidgetTester tester) =>
    tester
            .widget<EditableText>(
              find.byType(KansoEditableText, skipOffstage: false),
            )
            .controller
        as KansoMarkdownController;

/// Draws the fixture's note page on a phone.
Future<void> pumpPhone(WidgetTester tester, EditorFixture fixture) async {
  tester.view
    ..physicalSize = phoneSize * 3
    ..devicePixelRatio = 3;
  addTearDown(tester.view.reset);
  final bloc = SeededAppearanceBloc(const AppearanceState());
  addTearDown(bloc.close);
  await tester.pumpWidget(
    MaterialApp(
      theme: KansoTheme.light,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: BlocProvider<AppearanceBloc>.value(
        value: bloc,
        child: fixture.phone,
      ),
    ),
  );
  await fixture.settle(tester);
}

/// The page's scrollable.
Finder get page => find
    .descendant(
      of: find.byType(CustomScrollView),
      matching: find.byType(Scrollable),
    )
    .first;

/// Where on the screen the caret is.
double caretY(WidgetTester tester) {
  final state = tester.state<KansoEditableTextState>(
    find.byType(KansoEditableText, skipOffstage: false),
  );
  final render = state.renderEditable;
  final rect = render.getLocalRectForCaret(
    TextPosition(offset: controllerOf(tester).selection.extentOffset),
  );
  return render.localToGlobal(rect.center).dy;
}

void main() {
  setUp(() => EditableText.debugDeterministicCursor = true);
  tearDown(() => EditableText.debugDeterministicCursor = false);

  group('a long note on a phone', () {
    testWidgets('is edited a window at a time, with the rest drawn below', (
      tester,
    ) async {
      final fixture = await EditorFixture.open(
        tester,
        brightness: Brightness.light,
        body: longBody,
      );
      await pumpPhone(tester, fixture);
      final controller = controllerOf(tester);
      expect(controller.isWindowed, isTrue);
      expect(controller.text.length, lessThanOrEqualTo(WindowSizes.cap));
      expect(controller.noteText, longBody);
      expect(
        find.byType(OutsideText, skipOffstage: false),
        findsWidgets,
      );
      // The text after the window comes into view as the page is scrolled.
      await tester.scrollUntilVisible(
        find.textContaining('Line 381', findRichText: true),
        500,
        scrollable: page,
      );
      expect(
        find.textContaining('Line 381', findRichText: true),
        findsOneWidget,
      );
    });

    testWidgets('a tap below the window moves it there and keeps the place', (
      tester,
    ) async {
      final fixture = await EditorFixture.open(
        tester,
        brightness: Brightness.light,
        body: longBody,
      );
      await pumpPhone(tester, fixture);
      final controller = controllerOf(tester);
      await tester.scrollUntilVisible(
        find.textContaining('Line 301', findRichText: true),
        500,
        scrollable: page,
      );
      await tester.pump();
      final line = find.textContaining('Line 301', findRichText: true);
      final before = tester.getTopLeft(line).dy;
      await tester.tap(line);
      await fixture.settle(tester);
      await tester.pump();
      // The caret is on that line, in a window around it.
      final offset = longBody.indexOf('Line 301');
      expect(controller.noteSelection.baseOffset, greaterThanOrEqualTo(offset));
      expect(controller.noteSelection.baseOffset, lessThan(offset + 56));
      expect(controller.windowStart, greaterThan(longBody.indexOf('Line 151')));
      expect(controller.noteText, longBody);
      // And it is where the tap was, on the screen.
      expect(caretY(tester), closeTo(before + 10, 24));
    });

    testWidgets('Select All takes the whole note into the field', (
      tester,
    ) async {
      final fixture = await EditorFixture.open(
        tester,
        brightness: Brightness.light,
        body: longBody,
      );
      await pumpPhone(tester, fixture);
      final controller = controllerOf(tester);
      tester
          .state<KansoEditableTextState>(
            find.byType(KansoEditableText, skipOffstage: false),
          )
          .selectAll(SelectionChangedCause.toolbar);
      await tester.pump();
      expect(controller.isWindowed, isFalse);
      expect(controller.text, longBody);
      expect(controller.selection.start, 0);
      expect(controller.selection.end, longBody.length);
      expect(controller.after, isEmpty);
    });

    testWidgets('typing at the edge of the window does not move the page', (
      tester,
    ) async {
      final fixture = await EditorFixture.open(
        tester,
        brightness: Brightness.light,
        body: longBody,
      );
      await pumpPhone(tester, fixture);
      final controller = controllerOf(tester);
      // Put the caret near the end of the window, and type on.
      final near = controller.text.length - WindowSizes.margin - 40;
      controller.selection = TextSelection.collapsed(offset: near);
      await fixture.settle(tester);
      final revision = controller.windowRevision;
      // Scroll the page to the caret.
      await tester.drag(page, Offset(0, -(caretY(tester) - 400)));
      await fixture.settle(tester);
      var y = caretY(tester);
      expect(y, inInclusiveRange(0, 852));
      for (var i = 0; i < 40; i++) {
        final local = controller.selection.baseOffset;
        tester.testTextInput.updateEditingValue(
          TextEditingValue(
            text: controller.text.replaceRange(local, local, 'word '),
            selection: TextSelection.collapsed(offset: local + 5),
          ),
        );
        await tester.pump();
        await tester.pump();
        // However the window moves, the caret stays about where it was on
        // the screen: a line or two lower at most, for the typing.
        expect(caretY(tester), closeTo(y, 70), reason: 'after $i words');
        y = caretY(tester);
      }
      expect(controller.windowRevision, greaterThan(revision));
    });

    for (final brightness in Brightness.values) {
      testWidgets('the edge of the window, ${brightness.name}', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
          body: longBody,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          fixture.phone,
          name: 'phone_window_edge_${brightness.name}',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            // Scroll to where the field ends and the rest of the note goes
            // on.
            final controller = controllerOf(tester);
            final first = controller.after.substring(1).split('\n').first;
            final position = tester.state<ScrollableState>(page).position;
            // Put the first line after the window half way down the page.
            for (var i = 0; i < 40; i++) {
              final line = find.textContaining(first, findRichText: true);
              if (line.evaluate().isEmpty) {
                position.jumpTo(position.pixels + 400);
                await tester.pump();
                continue;
              }
              final top = tester.getTopLeft(line).dy;
              if ((top - 450).abs() < 15) {
                break;
              }
              position.jumpTo(position.pixels + (top - 450));
              await tester.pump();
            }
            await fixture.settle(tester);
            expect(controller.isWindowed, isTrue);
          },
        );
      });
    }
  });
}
