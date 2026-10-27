// Kanso — properties_section_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';
import 'package:kanso/editor/views/widgets/new_property_row.dart';
import 'package:kanso/editor/views/widgets/properties_row.dart';
import 'package:kanso/editor/views/widgets/properties_section.dart';
import 'package:kanso/editor/views/widgets/property_row.dart';

import '../../../helpers/pump.dart';

const _owner = Property(
  key: 'owner',
  kind: PropertyKind.text,
  text: 'Ada',
  items: <String>[],
);
const _rank = Property(
  key: 'rank',
  kind: PropertyKind.number,
  text: '2',
  items: <String>[],
);
const _done = Property(
  key: 'done',
  kind: PropertyKind.bool,
  text: 'false',
  items: <String>[],
);

class _Calls {
  final List<String> sets = <String>[];
  final List<int> removed = <int>[];
  int toggles = 0;
}

Future<_Calls> _pump(
  WidgetTester tester, {
  List<Property> properties = const <Property>[_owner, _rank, _done],
  bool open = true,
  bool failed = false,
}) async {
  final calls = _Calls();
  await pumpKanso(
    tester,
    SizedBox(
      width: 361,
      child: PropertiesSection(
        properties: properties,
        open: open,
        failed: failed,
        onToggle: () => calls.toggles++,
        onSet: (key, value, {index}) => calls.sets.add('$index $key $value'),
        onRemove: calls.removed.add,
      ),
    ),
  );
  return calls;
}

Finder _fieldOf(String key) => find.descendant(
  of: find.byWidgetPredicate(
    (widget) => widget is PropertyRow && widget.property.key == key,
  ),
  matching: find.byType(EditableText),
);

void main() {
  group('PropertiesSection', () {
    testWidgets('is the row with how many, and nothing else while closed', (
      tester,
    ) async {
      final calls = await _pump(tester, open: false);
      expect(find.text('Properties · 3'), findsOneWidget);
      expect(find.byType(PropertyRow), findsNothing);
      await tester.tap(find.byType(PropertiesRow));
      expect(calls.toggles, 1);
    });

    testWidgets('open, shows every property with its name and value', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.byType(PropertyRow), findsNWidgets(3));
      expect(find.text('owner'), findsOneWidget);
      expect(find.text('Ada'), findsOneWidget);
      expect(find.text('2'), findsOneWidget);
      expect(find.text('Add property'), findsOneWidget);
    });

    testWidgets('a yes or no is a switch, and a tap on it sets the value', (
      tester,
    ) async {
      final calls = await _pump(tester);
      expect(find.byType(KansoSwitch), findsOneWidget);
      await tester.tap(find.byType(KansoSwitch));
      expect(calls.sets, <String>['2 done true']);
    });

    testWidgets('a value typed is set with the return key', (tester) async {
      final calls = await _pump(tester);
      await tester.enterText(_fieldOf('owner'), 'Grace');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();
      expect(calls.sets, <String>['0 owner Grace']);
    });

    testWidgets('a value typed is set when the field is left', (tester) async {
      final calls = await _pump(tester);
      await tester.enterText(_fieldOf('rank'), '5');
      await tester.pump();
      expect(calls.sets, isEmpty);
      FocusManager.instance.primaryFocus?.unfocus();
      await tester.pump();
      expect(calls.sets, <String>['1 rank 5']);
    });

    testWidgets('a value left as it was is not set', (tester) async {
      final calls = await _pump(tester);
      await tester.tap(_fieldOf('owner'));
      await tester.pump();
      FocusManager.instance.primaryFocus?.unfocus();
      await tester.pump();
      expect(calls.sets, isEmpty);
    });

    testWidgets('a number gets the number keyboard', (tester) async {
      await _pump(tester);
      final field = tester.widget<EditableText>(_fieldOf('rank'));
      expect(field.keyboardType.index, TextInputType.number.index);
    });

    testWidgets('takes a property away by its button', (tester) async {
      final handle = tester.ensureSemantics();
      final calls = await _pump(tester);
      await tester.tap(find.bySemanticsLabel('Remove rank'));
      expect(calls.removed, <int>[1]);
      handle.dispose();
    });

    testWidgets('takes the value the core says when it is not being typed', (
      tester,
    ) async {
      await _pump(tester);
      await _pump(
        tester,
        properties: const <Property>[
          Property(
            key: 'owner',
            kind: PropertyKind.text,
            text: 'Grace',
            items: <String>[],
          ),
          _rank,
          _done,
        ],
      );
      expect(find.text('Grace'), findsOneWidget);
    });

    testWidgets('adds a property: a name, a value and the check', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      final calls = await _pump(tester);
      await tester.tap(find.text('Add property'));
      await tester.pump();
      await tester.pump();
      expect(find.byType(NewPropertyRow), findsOneWidget);
      expect(find.text('Add property'), findsNothing);
      final fields = find.descendant(
        of: find.byType(NewPropertyRow),
        matching: find.byType(EditableText),
      );
      await tester.enterText(fields.first, ' status ');
      await tester.enterText(fields.last, 'open');
      await tester.tap(find.bySemanticsLabel('Add'));
      await tester.pump();
      expect(calls.sets, <String>['null status open']);
      expect(find.byType(NewPropertyRow), findsNothing);
      expect(find.text('Add property'), findsOneWidget);
      handle.dispose();
    });

    testWidgets('a property with no name is not added', (tester) async {
      final handle = tester.ensureSemantics();
      final calls = await _pump(tester);
      await tester.tap(find.text('Add property'));
      await tester.pump();
      await tester.pump();
      await tester.tap(find.bySemanticsLabel('Add'));
      await tester.pump();
      expect(calls.sets, isEmpty);
      expect(find.byType(NewPropertyRow), findsOneWidget);
      handle.dispose();
    });

    testWidgets('gives up on a new property', (tester) async {
      final handle = tester.ensureSemantics();
      final calls = await _pump(tester);
      await tester.tap(find.text('Add property'));
      await tester.pump();
      await tester.tap(find.bySemanticsLabel('Cancel'));
      await tester.pump();
      expect(calls.sets, isEmpty);
      expect(find.byType(NewPropertyRow), findsNothing);
      handle.dispose();
    });

    testWidgets('says when a change did not work, as a live region', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await _pump(tester, failed: true);
      expect(find.text("That property couldn't be set."), findsOneWidget);
      handle.dispose();
    });
  });
}
