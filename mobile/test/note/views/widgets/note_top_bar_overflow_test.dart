// Kanso — note_top_bar_overflow_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_icon_button.dart';
import 'package:kanso/journal/views/widgets/calendar_sheet.dart';
import 'package:kanso/note/views/widgets/note_info_sheet.dart';
import 'package:kanso/note/views/widgets/note_mode_toggle.dart';
import 'package:kanso/note/views/widgets/note_top_bar.dart';
import 'package:kanso/sharing/views/widgets/comments_sheet.dart';
import 'package:kanso/sharing/views/widgets/share_note_sheet.dart';

import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/page_fixtures.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/sharing_fixtures.dart';

final String _dailyAt =
    '/notebooks/${Uri.encodeComponent(MockIds.journal)}/note/'
    '${Uri.encodeComponent(mockDailyNote.id)}';

/// The labels of the bar's actions that can give way, in the order they
/// do, and what each is called in the More menu.
const Map<String, String> _movable = <String, String>{
  'Note info': 'Note info',
  'Comments, 1 open': 'Comments',
  'Share': 'Share',
  'Calendar': 'Calendar',
  'Next day': 'Next day',
  'Previous day': 'Previous day',
};

/// The daily note, shared and with an open comment, in a window [width]
/// points wide.
Future<(BrowseApp, PageFixtures)> _pump(
  WidgetTester tester,
  double width, {
  bool tablet = false,
}) async {
  final shared = FixtureSharedNotesRepository();
  shared.shares[mockDailyNote.id] = shareInfo(mockDailyNote.id);
  shared.commentLists[mockDailyNote.id] = [commentRow('c1')];
  final fixtures = PageFixtures();
  final app = await withClock(
    Clock.fixed(mockNow),
    () => pumpBrowseApp(
      tester,
      Size(width, tablet ? 820 : 852),
      at: _dailyAt,
      shared: shared,
      fixtures: fixtures,
      tabletDevice: tablet,
    ),
  );
  // The comments are read after the first frames.
  for (var i = 0; i < 3; i++) {
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 5)),
    );
    await tester.pumpAndSettle();
  }
  return (app, fixtures);
}

/// The bar's buttons that can be seen and touched.
Finder get _barButtons => find
    .descendant(
      of: find.byType(NoteTopBar),
      matching: find.byType(KansoIconButton),
    )
    .hitTestable();

/// Whether the bar shows the button labelled [label].
bool _inBar(WidgetTester tester, String label) => _barButtons.evaluate().any(
  (e) => (e.widget as KansoIconButton).semanticLabel == label,
);

/// The labels the bar left out, which the More menu must have.
List<String> _hidden(WidgetTester tester) => <String>[
  for (final label in _movable.keys)
    if (!_inBar(tester, label)) label,
];

Future<void> _openMore(WidgetTester tester) async {
  await tester.tap(
    find.descendant(
      of: find.byType(NoteTopBar),
      matching: find.bySemanticsLabel('More'),
    ),
  );
  await tester.pumpAndSettle();
}

/// Checks the bar at [width]: nothing runs over, every button is whole and
/// beside the back button, and what gave way is in More, the least used
/// first.
Future<List<String>> _checkBar(WidgetTester tester, double width) async {
  expect(tester.takeException(), isNull);
  final buttons = _barButtons.evaluate().toList();
  expect(buttons, isNotEmpty);
  // The first button of the bar: Back on a phone, and on a tablet the
  // button that puts the library and the list away.
  final firstLabel = _inBar(tester, 'Hide sidebar and list')
      ? 'Hide sidebar and list'
      : 'Back';
  final back = tester.getRect(
    find.descendant(
      of: find.byType(NoteTopBar),
      matching: find.bySemanticsLabel(firstLabel),
    ),
  );
  for (final button in buttons) {
    final rect = tester.getRect(find.byWidget(button.widget));
    expect(rect.width, greaterThanOrEqualTo(44));
    expect(rect.height, greaterThanOrEqualTo(44));
    expect(rect.right, lessThanOrEqualTo(width + 0.01));
    if (rect != back) {
      expect(rect.left, greaterThanOrEqualTo(back.right - 0.01));
    }
  }
  // The mode switch and More are always there, at their own size.
  final toggle = tester.getRect(find.byType(NoteModeToggle));
  expect(toggle.left, greaterThanOrEqualTo(back.right - 0.01));
  expect(_inBar(tester, 'More'), isTrue);
  final hidden = _hidden(tester);
  // What gave way is the start of the order.
  expect(hidden, _movable.keys.take(hidden.length).toList());
  await _openMore(tester);
  for (final label in _movable.keys) {
    expect(
      find.text(_movable[label]!),
      hidden.contains(label) ? findsOneWidget : findsNothing,
      reason: label,
    );
  }
  await tester.tapAt(const Offset(10, 10));
  await tester.pumpAndSettle();
  return hidden;
}

void main() {
  group('the bar of a journal note on a phone', () {
    for (final width in <double>[320, 393, 430]) {
      testWidgets('at $width points fits, with whole buttons and the rest '
          'in More', (tester) async {
        await _pump(tester, width);
        final hidden = await _checkBar(tester, width);
        expect(_inBar(tester, 'Previous day'), isTrue);
        expect(_inBar(tester, 'Next day'), isTrue);
        if (width == 320) {
          expect(hidden, containsAll(<String>['Note info', 'Share']));
        }
      });
    }

    testWidgets('info, comments and share from More open their sheets', (
      tester,
    ) async {
      await _pump(tester, 320);
      final hidden = _hidden(tester);
      expect(
        hidden,
        containsAll(<String>['Note info', 'Comments, 1 open', 'Share']),
      );
      await _openMore(tester);
      await tester.tap(find.text('Note info'));
      await tester.pumpAndSettle();
      expect(find.byType(NoteInfoSheet), findsOneWidget);
      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();

      await _openMore(tester);
      await tester.tap(find.text('Comments'));
      await tester.pumpAndSettle();
      expect(find.byType(CommentsSheet), findsOneWidget);
      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();

      await _openMore(tester);
      await tester.tap(find.text('Share'));
      await tester.pumpAndSettle();
      expect(find.byType(ShareNoteSheet), findsOneWidget);
    });

    testWidgets('narrower still, the days go to More too, and work there', (
      tester,
    ) async {
      final (app, fixtures) = await _pump(tester, 240);
      final hidden = await _checkBar(tester, 240);
      expect(
        hidden,
        containsAll(<String>['Calendar', 'Next day', 'Previous day']),
      );
      await _openMore(tester);
      await tester.tap(find.text('Next day'));
      await tester.pumpAndSettle();
      expect(fixtures.journal.calls, <String>['step:forward']);
      expect(app.where, contains('note%3Adaily-2026-10-27'));
      await _openMore(tester);
      await tester.tap(find.text('Calendar'));
      await tester.pumpAndSettle();
      expect(find.byType(CalendarSheet), findsOneWidget);
    });
  });

  testWidgets('on an iPad in portrait, where the note pane is narrow, the '
      'bar gives way rather than run off the pane', (tester) async {
    await _pump(tester, 820, tablet: true);
    final hidden = await _checkBar(tester, 820);
    expect(hidden, isNotEmpty);
  });

  testWidgets('on a wide tablet only the note info gives way, and with the '
      'library and list put away nothing does', (tester) async {
    await _pump(tester, 1180, tablet: true);
    expect(tester.takeException(), isNull);
    expect(_hidden(tester), <String>['Note info']);
    await tester.tap(find.bySemanticsLabel('Hide sidebar and list'));
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(_hidden(tester), isEmpty);
  });
}
