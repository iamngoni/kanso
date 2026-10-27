// Kanso — kanso_surfaces_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/core/utils/contrast.dart';
import 'package:kanso/core/views/widgets/kanso_badge.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';
import 'package:kanso/core/views/widgets/kanso_grouped_section.dart';
import 'package:kanso/core/views/widgets/kanso_scrim.dart';

import '../../../helpers/pump.dart';

Color? _fillOf(WidgetTester tester, Finder within) {
  final boxes = tester.widgetList<Container>(
    find.descendant(of: within, matching: find.byType(Container)),
  );
  for (final box in boxes) {
    final decoration = box.decoration;
    if (decoration is BoxDecoration && decoration.color != null) {
      return decoration.color;
    }
  }
  return null;
}

/// The colour of the outline of the first outlined box in [within].
Color? _outlineOf(WidgetTester tester, Finder within) {
  final boxes = tester.widgetList<Container>(
    find.descendant(of: within, matching: find.byType(Container)),
  );
  for (final box in boxes) {
    final decoration = box.decoration;
    if (decoration is BoxDecoration && decoration.border is Border) {
      return (decoration.border! as Border).top.color;
    }
  }
  return null;
}

void main() {
  group('grouped sections', () {
    for (final brightness in Brightness.values) {
      testWidgets('are a calm step from the page in $brightness', (
        tester,
      ) async {
        late BuildContext captured;
        await pumpKanso(
          tester,
          Builder(
            builder: (context) {
              captured = context;
              return const SizedBox();
            },
          ),
          brightness: brightness,
        );
        final ratio = contrastRatio(
          captured.groupedSurface,
          captured.colors.app,
        );
        // Visible, but not a slab: about the same step in both themes.
        expect(ratio, greaterThan(1.1));
        expect(ratio, lessThan(1.3));
      });
    }

    testWidgets('draw a footer note under the rows', (tester) async {
      await pumpKanso(
        tester,
        const KansoGroupedSection(
          title: 'Title',
          footer: 'A footnote.',
          children: <Widget>[Text('Row')],
        ),
      );
      expect(find.text('A footnote.'), findsOneWidget);
      expect(
        tester.getTopLeft(find.text('A footnote.')).dy,
        greaterThan(tester.getBottomLeft(find.text('Row')).dy),
      );
    });
  });

  group('badges and buttons on a dark card', () {
    testWidgets('a badge is visible against the card', (tester) async {
      await pumpKanso(
        tester,
        const KansoBadge(label: 'Built in'),
        brightness: Brightness.dark,
      );
      final badge = _fillOf(tester, find.byType(KansoBadge))!;
      final over = Color.alphaBlend(badge, KansoColors.dark.elevated);
      expect(contrastRatio(over, KansoColors.dark.elevated), greaterThan(1.05));
    });

    testWidgets('a secondary button is visible against the card', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const KansoButton(
          label: 'Use password',
          variant: KansoButtonVariant.secondary,
        ),
        brightness: Brightness.dark,
      );
      // It is raised in the card's own colour, set apart by its outline.
      final outline = _outlineOf(tester, find.byType(KansoButton))!;
      final over = Color.alphaBlend(outline, KansoColors.dark.elevated);
      expect(contrastRatio(over, KansoColors.dark.elevated), greaterThan(1.05));
    });

    testWidgets('a compact button has a 44 point touch target', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const KansoButton(label: 'Install', compact: true, expanded: false),
      );
      expect(tester.getSize(find.byType(KansoButton)).height, 44);
    });
  });

  group('KansoScrim', () {
    for (final brightness in Brightness.values) {
      testWidgets('is the same veil wherever it is used in $brightness', (
        tester,
      ) async {
        await pumpKanso(
          tester,
          const SizedBox(width: 50, height: 50, child: KansoScrim()),
          brightness: brightness,
        );
        final first = tester
            .widget<ColoredBox>(
              find.descendant(
                of: find.byType(KansoScrim),
                matching: find.byType(ColoredBox),
              ),
            )
            .color;
        expect(first.a, brightness == Brightness.dark ? 0.6 : 0.34);
      });
    }
  });
}
