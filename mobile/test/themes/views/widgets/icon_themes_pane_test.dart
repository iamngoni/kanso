// Kanso — icon_themes_pane_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/installed_icon_theme.dart';
import 'package:kanso/core/models/data/installed_theme_origin.dart';
import 'package:kanso/themes/state/icon_themes/icon_themes_bloc.dart';
import 'package:kanso/themes/views/widgets/icon_theme_row.dart';
import 'package:kanso/themes/views/widgets/icon_themes_pane.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_file_picker_repository.dart';
import '../../../helpers/fixture_themes_repository.dart';
import '../../../helpers/pump_routed.dart';

Future<FixtureThemesRepository> _pump(
  WidgetTester tester, {
  FixtureThemesRepository? themes,
  FixtureFilePickerRepository? files,
}) async {
  final repository = themes ?? FixtureThemesRepository();
  final bloc = IconThemesBloc(
    repository,
    files ?? FixtureFilePickerRepository(),
  )..add(const IconThemesEvent.started());
  closeBlocLater(bloc);
  await pumpRouted(
    tester,
    BlocProvider<IconThemesBloc>.value(
      value: bloc,
      child: const IconThemesPane(),
    ),
  );
  for (var i = 0; i < 4; i++) {
    await tester.pump();
  }
  return repository;
}

Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 5; i++) {
    await tester.pump();
  }
}

void main() {
  group('IconThemesPane', () {
    testWidgets('says there are none yet, and offers the import', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Icon themes'), findsOneWidget);
      expect(find.text('No icon themes yet'), findsOneWidget);
      expect(find.byType(IconThemeRow), findsNothing);
      expect(find.text('Import icon theme'), findsOneWidget);
      expect(find.text('Import…'), findsOneWidget);
      expect(
        find.text(
          'A Zed icon theme (its extension folder or .tar.gz), a theme '
          'package carrying one, or a Kanso icon theme folder, converted '
          'into the icon themes folder.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('lists the icon themes with who made them and where from', (
      tester,
    ) async {
      await _pump(
        tester,
        themes: FixtureThemesRepository(
          iconThemeList: <InstalledIconTheme>[
            const InstalledIconTheme(
              id: 'material',
              name: 'Material',
              author: 'Ana',
              license: 'MIT',
              origin: InstalledThemeOrigin.gallery,
            ),
            const InstalledIconTheme(
              id: 'mine',
              name: 'Mine',
              author: '',
              license: '',
              origin: InstalledThemeOrigin.own,
            ),
          ],
        ),
      );
      expect(find.byType(IconThemeRow), findsNWidgets(2));
      expect(find.text('Material · By Ana'), findsOneWidget);
      expect(find.text('From the gallery'), findsOneWidget);
      expect(find.text('Mine'), findsOneWidget);
      expect(find.text('Yours'), findsOneWidget);
      expect(find.text('No icon themes yet'), findsNothing);
    });

    testWidgets('importing adds the icon theme to the list and says what '
        'was in it', (tester) async {
      final themes = await _pump(
        tester,
        files: FixtureFilePickerRepository(<String?>['/tmp/lucide.tar.gz']),
      );
      await tester.tap(find.text('Import…'));
      await _settle(tester);
      expect(themes.importedIconThemePaths, <String>['/tmp/lucide.tar.gz']);
      expect(find.text('Imported Lucide. 12 icons imported.'), findsOneWidget);
      expect(find.byType(IconThemeRow), findsOneWidget);
      expect(find.text('No icon themes yet'), findsNothing);
    });

    testWidgets('shows why an import failed', (tester) async {
      final themes = FixtureThemesRepository()
        ..importFailure = const ApplicationError.invalid(
          message: 'No icons in it.',
        );
      await _pump(
        tester,
        themes: themes,
        files: FixtureFilePickerRepository(<String?>['/tmp/bad.zip']),
      );
      await tester.tap(find.text('Import…'));
      await _settle(tester);
      expect(
        find.text('Couldn’t import the icon theme. No icons in it.'),
        findsOneWidget,
      );
      expect(find.byType(IconThemeRow), findsNothing);
    });

    testWidgets('does nothing when the picker is closed', (tester) async {
      final themes = await _pump(tester);
      await tester.tap(find.text('Import icon theme'));
      await _settle(tester);
      expect(themes.importedIconThemePaths, isEmpty);
      expect(
        find.byKey(const ValueKey<String>('theme-import-outcome')),
        findsNothing,
      );
    });
  });
}
