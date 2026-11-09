// Kanso — kanso_time_sheet_test.dart
// Created 2026-11-09.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_time_sheet.dart';

import '../../../helpers/pump.dart';

void main() {
  Future<void> open(WidgetTester tester, void Function(String?) done) async {
    await pumpKanso(
      tester,
      Builder(
        builder: (context) => TextButton(
          onPressed: () async => done(
            await KansoTimeSheet.show(
              context,
              title: 'Daily reminder',
              initial: DateTime(2026, 11, 9, 9, 30),
            ),
          ),
          child: const Text('Open'),
        ),
      ),
    );
    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
  }

  testWidgets('shows the iOS wheel and no Material time picker', (
    tester,
  ) async {
    await open(tester, (_) {});
    expect(find.byType(CupertinoDatePicker), findsOneWidget);
    expect(find.byType(TimePickerDialog), findsNothing);
  });

  testWidgets('Done returns the shown time as HH:mm', (tester) async {
    String? result;
    await open(tester, (v) => result = v);
    await tester.tap(find.text('Done'));
    await tester.pumpAndSettle();
    expect(result, '09:30');
  });

  testWidgets('Cancel returns nothing', (tester) async {
    String? result = 'unset';
    await open(tester, (v) => result = v);
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(result, isNull);
  });
}
