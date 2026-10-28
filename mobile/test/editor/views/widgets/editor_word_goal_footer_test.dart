// Kanso — editor_word_goal_footer_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/editor/views/widgets/editor_word_goal_footer.dart';

import '../../../helpers/pump.dart';

void main() {
  group('EditorWordGoalFooter', () {
    testWidgets('offers to set a goal when none applies', (tester) async {
      var tapped = false;
      await pumpKanso(
        tester,
        SizedBox(
          width: 320,
          child: EditorWordGoalFooter(
            words: 12,
            goal: null,
            onTap: () => tapped = true,
          ),
        ),
      );

      expect(find.text('Set word goal'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.tap(find.byType(EditorWordGoalFooter));
      expect(tapped, isTrue);
    });

    testWidgets('shows progress and disables the action for a reader', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const SizedBox(
          width: 320,
          child: EditorWordGoalFooter(
            words: 640,
            goal: 1000,
            onTap: null,
          ),
        ),
      );

      expect(find.text('640 / 1,000'), findsOneWidget);
      expect(tester.takeException(), isNull);
      expect(tester.widget<InkWell>(find.byType(InkWell)).onTap, isNull);
    });
  });
}
