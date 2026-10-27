// Kanso — design_gallery_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/design/models/data/gallery_appearance.dart';
import 'package:kanso/design/views/pages/design_gallery_page.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

import '../../../helpers/seeded_appearance_bloc.dart';

Future<AppearanceBloc> _pump(WidgetTester tester) async {
  final bloc = SeededAppearanceBloc(const AppearanceState());
  addTearDown(bloc.close);
  tester.view
    ..physicalSize = const Size(393, 2400)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(
    BlocProvider<AppearanceBloc>.value(
      value: bloc,
      child: MaterialApp(
        theme: KansoTheme.light,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: const DesignGalleryPage(),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return bloc;
}

/// Lets the appearance BLoC finish a choice: it reads the themes and saves
/// before the next one, in microtasks that a single pump runs.
Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 5; i++) {
    await tester.pump();
  }
  await tester.pumpAndSettle();
}

void main() {
  group('DesignGalleryPage', () {
    testWidgets(
      'lists the screens that are not built yet, on a phone and a tablet',
      (tester) async {
        await _pump(tester);
        expect(find.text('Phone'), findsOneWidget);
        expect(find.text('Tablet'), findsOneWidget);
        // Every screen that is not built yet is listed once per layout.
        expect(find.text('Note, edit'), findsNothing);
        expect(find.text('Error card'), findsNWidgets(2));
        // The screens that are built are not mocks any more.
        expect(find.text('Note list'), findsNothing);
        expect(find.text('Library'), findsNothing);
      },
    );

    testWidgets('the appearance control dispatches the chosen mode', (
      tester,
    ) async {
      final bloc = await _pump(tester);
      expect(GalleryAppearance.of(bloc.state), GalleryAppearance.system);
      await tester.tap(find.text('Dark'));
      await _settle(tester);
      expect(GalleryAppearance.of(bloc.state), GalleryAppearance.dark);
      expect(bloc.state.selection.ui, 'kanso-dark');
      await tester.tap(find.text('Light'));
      await _settle(tester);
      expect(GalleryAppearance.of(bloc.state), GalleryAppearance.light);
      await tester.tap(find.text('System'));
      await _settle(tester);
      expect(GalleryAppearance.of(bloc.state), GalleryAppearance.system);
    });

    testWidgets('opens a screen and comes back', (tester) async {
      await _pump(tester);
      await tester.tap(find.text('Error card').first);
      await tester.pumpAndSettle();
      expect(find.text('Signed out on this device'), findsOneWidget);
      tester.state<NavigatorState>(find.byType(Navigator)).pop();
      await tester.pumpAndSettle();
      expect(find.text('Design proposal'), findsOneWidget);
    });
  });
}
