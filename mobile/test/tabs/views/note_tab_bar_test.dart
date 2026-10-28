// Kanso — note_tab_bar_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/models/data/note_drag_data.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/tabs/models/data/tab_meta.dart';
import 'package:kanso/tabs/views/widgets/note_tab_bar.dart';

import '../../helpers/pump.dart';

/// What the bar asked for.
class _Asked {
  final List<String> log = <String>[];
  final List<(int, int)> reorders = <(int, int)>[];
  final List<List<String>> drops = <List<String>>[];
}

const List<TabMeta> _three = <TabMeta>[
  TabMeta(id: 'a', title: 'Note a'),
  TabMeta(id: 'b', title: 'Note b', locked: true),
  TabMeta(id: 'c', title: ''),
];

NoteTabBar _bar(
  _Asked asked, {
  List<TabMeta> tabs = _three,
  int active = 0,
  bool windows = false,
}) {
  return NoteTabBar(
    tabs: tabs,
    active: active,
    onActivate: (index) => asked.log.add('activate $index'),
    onClose: (index) => asked.log.add('close $index'),
    onCloseOthers: (index) => asked.log.add('others $index'),
    onCloseRight: (index) => asked.log.add('right $index'),
    onReorder: (from, to) => asked.reorders.add((from, to)),
    onDropNotes: asked.drops.add,
    onOpenInNewWindow: windows ? (id) => asked.log.add('window $id') : null,
  );
}

Finder _closeIcon() => find.byWidgetPredicate(
  (widget) => widget is KansoIcon && widget.icon == KansoIcons.close,
);

void main() {
  group('NoteTabBar', () {
    testWidgets('shows each title, Untitled for a blank one, a lock', (
      tester,
    ) async {
      await pumpKanso(tester, _bar(_Asked()));
      expect(find.text('Note a'), findsOneWidget);
      expect(find.text('Note b'), findsOneWidget);
      expect(find.text('Untitled'), findsOneWidget);
      final locks = find.byWidgetPredicate(
        (widget) => widget is KansoIcon && widget.icon == KansoIcons.lock,
      );
      expect(locks, findsOneWidget);
      expect(
        tester.getCenter(locks).dx,
        lessThan(tester.getCenter(find.text('Note b')).dx),
        reason: 'before the title',
      );
    });

    testWidgets("the bar is as tall as the desktop's", (tester) async {
      await pumpKanso(tester, _bar(_Asked()));
      expect(tester.getSize(find.byType(NoteTabBar)).height, 38);
    });

    testWidgets('a tap shows the tab', (tester) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked));
      await tester.tap(find.text('Note b'));
      await tester.pump();
      expect(asked.log, <String>['activate 1']);
    });

    testWidgets('the active tab is selected for a screen reader', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await pumpKanso(tester, _bar(_Asked(), active: 1));
      expect(
        tester.getSemantics(find.bySemanticsLabel('Note b, locked')),
        isSemantics(isSelected: true, isButton: true, hasTapAction: true),
      );
      expect(
        tester.getSemantics(find.bySemanticsLabel('Note a')),
        isNot(isSemantics(isSelected: true)),
      );
      expect(find.bySemanticsLabel('Tabs'), findsOneWidget);
      handle.dispose();
    });

    testWidgets('the close button is on the showing tab', (tester) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked, active: 1));
      expect(_closeIcon(), findsNWidgets(3));
      // Only the showing tab's is drawn and answers a touch.
      final opacities = tester
          .widgetList<Opacity>(
            find.ancestor(of: _closeIcon(), matching: find.byType(Opacity)),
          )
          .map((opacity) => opacity.opacity)
          .toList();
      expect(opacities, <double>[0, 1, 0]);
      await tester.tap(_closeIcon().at(1));
      // Where a hidden one is, a touch is on the tab.
      await tester.tap(_closeIcon().at(0), warnIfMissed: false);
      expect(asked.log, <String>['close 1', 'activate 0']);
    });

    testWidgets('under a pointer the close button is on the tab it is over', (
      tester,
    ) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked));
      final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: Offset.zero);
      addTearDown(mouse.removePointer);
      await mouse.moveTo(tester.getCenter(find.text('Note b')));
      await tester.pump();
      await tester.tap(_closeIcon().at(1));
      expect(asked.log, <String>['close 1']);
    });

    testWidgets("a right click opens the menu with the desktop's labels", (
      tester,
    ) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked, windows: true));
      await tester.tap(find.text('Note b'), buttons: kSecondaryButton);
      await tester.pumpAndSettle();
      expect(find.text('Close'), findsOneWidget);
      expect(find.text('Close Others'), findsOneWidget);
      expect(find.text('Close Tabs to the Right'), findsOneWidget);
      expect(find.text('Open in New Window'), findsOneWidget);
      await tester.tap(find.text('Close Others'));
      await tester.pumpAndSettle();
      expect(asked.log, <String>['others 1']);
      expect(find.text('Close Others'), findsNothing);
    });

    testWidgets('the menu closes a tab and the tabs to the right', (
      tester,
    ) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked));
      await tester.tap(find.text('Note a'), buttons: kSecondaryButton);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Close Tabs to the Right'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Note a'), buttons: kSecondaryButton);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(asked.log, <String>['right 0', 'close 0']);
    });

    testWidgets('the last tab has no tabs to its right to close', (
      tester,
    ) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked));
      await tester.tap(find.text('Untitled'), buttons: kSecondaryButton);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Close Tabs to the Right'));
      await tester.pumpAndSettle();
      expect(asked.log, isEmpty);
      await tester.tapAt(Offset.zero);
      await tester.pumpAndSettle();
    });

    testWidgets('a long press let go in place opens the menu', (tester) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked, windows: true));
      await tester.longPress(find.text('Note b'));
      await tester.pumpAndSettle();
      expect(find.text('Open in New Window'), findsOneWidget);
      await tester.tap(find.text('Open in New Window'));
      await tester.pumpAndSettle();
      expect(asked.log, <String>['window b']);
      expect(asked.reorders, isEmpty);
    });

    testWidgets('Open in New Window is not offered where it is not there', (
      tester,
    ) async {
      await pumpKanso(tester, _bar(_Asked()));
      await tester.tap(find.text('Note b'), buttons: kSecondaryButton);
      await tester.pumpAndSettle();
      expect(find.text('Close'), findsOneWidget);
      expect(find.text('Open in New Window'), findsNothing);
    });

    testWidgets('a held tab dragged onto another moves there', (tester) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked));
      final gesture = await tester.startGesture(
        tester.getCenter(find.text('Note a')),
      );
      await tester.pump(kLongPressTimeout + const Duration(milliseconds: 100));
      await gesture.moveTo(tester.getCenter(find.text('Untitled')));
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();
      expect(asked.reorders, <(int, int)>[(0, 2)]);
      expect(find.text('Close'), findsNothing, reason: 'no menu after a drag');
    });

    testWidgets('a tab dropped on itself moves nowhere', (tester) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked));
      final gesture = await tester.startGesture(
        tester.getCenter(find.text('Note a')),
      );
      await tester.pump(kLongPressTimeout + const Duration(milliseconds: 100));
      await gesture.moveBy(const Offset(40, 0));
      await tester.pump();
      await gesture.moveBy(const Offset(-40, 0));
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();
      expect(asked.reorders, isEmpty);
    });

    testWidgets('notes dropped on the bar are handed over to open', (
      tester,
    ) async {
      final asked = _Asked();
      await pumpKanso(
        tester,
        Column(
          children: <Widget>[
            _bar(asked),
            const Draggable<NoteDragData>(
              data: NoteDragData(
                noteIds: <String>['x', 'y'],
                titles: <String>['X', 'Y'],
              ),
              feedback: SizedBox.square(dimension: 20),
              child: SizedBox(width: 100, height: 60, child: Text('Drag me')),
            ),
          ],
        ),
      );
      final gesture = await tester.startGesture(
        tester.getCenter(find.text('Drag me')),
      );
      await gesture.moveTo(tester.getCenter(find.byType(NoteTabBar)));
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();
      expect(asked.drops, <List<String>>[
        <String>['x', 'y'],
      ]);
    });

    testWidgets('the row scrolls and keeps the showing tab in view', (
      tester,
    ) async {
      tester.view
        ..physicalSize = const Size(320, 600)
        ..devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final tabs = <TabMeta>[
        for (var n = 0; n < 9; n++)
          TabMeta(id: 'n$n', title: 'A long note title $n'),
      ];
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked, tabs: tabs));
      final bar = find.byType(NoteTabBar);
      expect(
        tester.getRect(find.text('A long note title 8')).left,
        greaterThan(
          tester.getRect(bar).right,
        ),
      );
      await pumpKanso(tester, _bar(asked, tabs: tabs, active: 8));
      await tester.pumpAndSettle();
      final shown = tester.getRect(find.text('A long note title 8'));
      expect(shown.right, lessThanOrEqualTo(tester.getRect(bar).right));
      expect(shown.left, greaterThanOrEqualTo(tester.getRect(bar).left));
    });

    testWidgets('the button at the end lists every tab and shows one', (
      tester,
    ) async {
      final asked = _Asked();
      await pumpKanso(tester, _bar(asked, active: 1));
      await tester.tap(find.bySemanticsLabel('All tabs'));
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.done), findsOneWidget);
      await tester.tap(
        find.widgetWithText(CheckedPopupMenuItem<int>, 'Note a'),
      );
      await tester.pumpAndSettle();
      expect(asked.log, <String>['activate 0']);
    });
  });
}
