// Kanso — kanso_button_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';

import '../../../helpers/pump.dart';

void main() {
  group('KansoButton', () {
    for (final variant in KansoButtonVariant.values) {
      testWidgets('${variant.name} shows its label', (tester) async {
        await pumpKanso(
          tester,
          KansoButton(label: 'Go on', variant: variant, onPressed: () {}),
        );
        expect(find.text('Go on'), findsOneWidget);
      });
    }

    testWidgets('primary is filled with the button colour', (tester) async {
      await pumpKanso(
        tester,
        KansoButton(label: 'Go', onPressed: () {}, expanded: false),
      );
      final box = tester.widget<Container>(
        find.descendant(
          of: find.byType(KansoButton),
          matching: find.byType(Container),
        ),
      );
      expect(
        (box.decoration! as BoxDecoration).color,
        KansoColors.light.button,
      );
    });

    testWidgets('quiet has no fill', (tester) async {
      await pumpKanso(
        tester,
        KansoButton(
          label: 'Go',
          variant: KansoButtonVariant.quiet,
          onPressed: () {},
        ),
      );
      final box = tester.widget<Container>(
        find.descendant(
          of: find.byType(KansoButton),
          matching: find.byType(Container),
        ),
      );
      expect((box.decoration! as BoxDecoration).color, isNull);
    });

    testWidgets('is 52 points tall, a 44 point target when compact', (
      tester,
    ) async {
      await pumpKanso(tester, KansoButton(label: 'Go', onPressed: () {}));
      expect(tester.getSize(find.byType(KansoButton)).height, 52);
      await pumpKanso(
        tester,
        KansoButton(label: 'Go', compact: true, onPressed: () {}),
      );
      expect(tester.getSize(find.byType(KansoButton)).height, 44);
      expect(
        tester
            .getSize(
              find.descendant(
                of: find.byType(KansoButton),
                matching: find.byType(Container),
              ),
            )
            .height,
        36,
      );
    });

    testWidgets('a tap calls onPressed', (tester) async {
      var taps = 0;
      await pumpKanso(
        tester,
        KansoButton(label: 'Go', onPressed: () => taps++),
      );
      await tester.tap(find.byType(KansoButton));
      expect(taps, 1);
    });

    testWidgets('does not outgrow a narrow space', (tester) async {
      await pumpKanso(
        tester,
        SizedBox(
          width: 90,
          child: KansoButton(label: 'A rather long label', onPressed: () {}),
        ),
      );
      expect(tester.takeException(), isNull);
    });
  });
}
