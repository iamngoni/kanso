// Kanso — icons_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'helpers/flow_driver.dart';

/// Choosing an icon theme as a person does: Settings › Appearance › Icons,
/// a theme chosen from the list, and every icon in the app drawn in it at
/// once, then Kanso's own icons chosen again.
///
/// Run it with `flutter test integration_test/icons_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  /// How many icons on screen are drawn from a theme's SVG rather than from
  /// the bundled Lucide files.
  int themedIcons(WidgetTester tester) => tester
      .widgetList<SvgPicture>(find.byType(SvgPicture))
      .where((picture) => picture.bytesLoader is SvgStringLoader)
      .length;

  testWidgets('choose an icon theme and see every icon change', (tester) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    await pumpUntil(tester, find.byType(SvgPicture));

    // Kanso's own icons are in use, and nothing is drawn from a theme.
    expect(themedIcons(tester), 0);

    // Settings, from the library.
    await openLibrary(tester);
    await tester.tap(find.bySemanticsLabel('Settings').first);
    await tester.pumpAndSettle();
    expect(find.text('Appearance'), findsOneWidget);
    expect(find.text('Icons'), findsOneWidget);
    expect(find.text('Lucide'), findsOneWidget);

    // The Icons page lists the themes, with Lucide in use.
    await tester.tap(find.text('Icons'));
    await pumpUntil(tester, find.text('Phosphor'));
    await tester.scrollUntilVisible(
      find.text('Phosphor'),
      200,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.pumpAndSettle();
    expect(themedIcons(tester), greaterThan(0), reason: 'the previews');
    final previews = themedIcons(tester);

    // Choosing Phosphor redraws the page's own icons in it at once.
    await tester.tap(find.text('Phosphor'));
    await tester.pumpAndSettle();
    expect(themedIcons(tester), greaterThan(previews));

    // Back in Settings the row names the theme, and the rows' icons are its.
    await tester.tap(find.bySemanticsLabel('Back').first);
    await tester.pumpAndSettle();
    expect(find.text('Phosphor'), findsOneWidget);
    expect(find.text('Lucide'), findsNothing);
    expect(themedIcons(tester), greaterThan(0));

    // Kanso's own icons again: nothing is drawn from a theme.
    await tester.tap(find.text('Icons'));
    await pumpUntil(tester, find.text('Lucide'));
    await tester.tap(find.text('Lucide'));
    await tester.pumpAndSettle();
    await tester.tap(find.bySemanticsLabel('Back').first);
    await tester.pumpAndSettle();
    expect(find.text('Lucide'), findsOneWidget);
    expect(themedIcons(tester), 0);

    semantics.dispose();
  });
}
