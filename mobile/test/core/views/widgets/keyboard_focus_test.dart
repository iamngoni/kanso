// Kanso — keyboard_focus_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_bottom_sheet_route.dart';
import 'package:kanso/core/views/widgets/kanso_page.dart';
import 'package:kanso/core/views/widgets/kanso_text_field.dart';

import '../../../helpers/pump.dart';

/// iPad portrait and landscape (13 inch), and the 11 inch in landscape.
const List<Size> _ipads = <Size>[
  Size(1024, 1366),
  Size(1366, 1024),
  Size(1180, 820),
];

bool _focused(WidgetTester tester) =>
    tester.widget<EditableText>(find.byType(EditableText)).focusNode.hasFocus;

/// The keyboard opening: the view's bottom inset grows in steps.
Future<void> _openKeyboard(WidgetTester tester, {double height = 420}) async {
  for (final step in <double>[0.2, 0.5, 1]) {
    tester.view.viewInsets = FakeViewPadding(bottom: height * step);
    await tester.pump(const Duration(milliseconds: 16));
  }
  await tester.pumpAndSettle();
}

void main() {
  for (final size in _ipads) {
    group('on an iPad of ${size.width.toInt()}x${size.height.toInt()}', () {
      setUp(() {});

      testWidgets(
        'KansoPage keeps the focus of a field as the keyboard opens',
        (
          tester,
        ) async {
          tester.view
            ..physicalSize = size
            ..devicePixelRatio = 1;
          addTearDown(tester.view.reset);
          final controller = TextEditingController();
          addTearDown(controller.dispose);
          await pumpKanso(
            tester,
            SizedBox(
              width: size.width,
              height: size.height,
              child: KansoPage(
                body: Align(
                  alignment: Alignment.topCenter,
                  child: SizedBox(
                    width: 400,
                    child: KansoTextField(
                      controller: controller,
                      label: 'Server',
                    ),
                  ),
                ),
              ),
            ),
          );

          await tester.tap(find.byType(EditableText));
          await tester.pump();
          expect(_focused(tester), isTrue);

          await _openKeyboard(tester);

          expect(find.byType(EditableText), findsOneWidget);
          expect(_focused(tester), isTrue);
        },
      );

      testWidgets('a sheet keeps the focus of a field in a scrolling body', (
        tester,
      ) async {
        tester.view
          ..physicalSize = size
          ..devicePixelRatio = 1;
        addTearDown(tester.view.reset);
        final controller = TextEditingController();
        addTearDown(controller.dispose);
        await pumpKanso(
          tester,
          Builder(
            builder: (context) => TextButton(
              onPressed: () => showKansoBottomSheet<void>(
                context,
                builder: (_) => SingleChildScrollView(
                  child: Column(
                    children: <Widget>[
                      const SizedBox(height: 300),
                      KansoTextField(controller: controller, label: 'Password'),
                      const SizedBox(height: 600),
                    ],
                  ),
                ),
              ),
              child: const Text('Open sheet'),
            ),
          ),
        );
        await tester.tap(find.text('Open sheet'));
        await tester.pumpAndSettle();

        await tester.tap(find.byType(EditableText));
        await tester.pump();
        expect(_focused(tester), isTrue);

        await _openKeyboard(tester);
        // The field is brought into view by a scroll no finger made.
        await tester.pumpAndSettle();

        expect(_focused(tester), isTrue);
      });
    });
  }
}
