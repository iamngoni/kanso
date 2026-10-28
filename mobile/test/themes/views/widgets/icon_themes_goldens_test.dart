// Kanso — icon_themes_goldens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/installed_icon_theme.dart';
import 'package:kanso/core/models/data/installed_theme_origin.dart';
import 'package:kanso/themes/state/icon_themes/icon_themes_bloc.dart';
import 'package:kanso/themes/views/widgets/icon_themes_pane.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_file_picker_repository.dart';
import '../../../helpers/fixture_themes_repository.dart';
import '../../../helpers/golden.dart';

Future<Widget> _page({
  List<InstalledIconTheme> themes = const <InstalledIconTheme>[],
  FixtureThemesRepository? repository,
  FixtureFilePickerRepository? files,
}) async {
  final bloc = IconThemesBloc(
    repository ??
        FixtureThemesRepository(iconThemeList: <InstalledIconTheme>[...themes]),
    files ?? FixtureFilePickerRepository(),
  )..add(const IconThemesEvent.started());
  closeBlocLater(bloc);
  await bloc.stream.firstWhere((state) => state.status.name != 'loading');
  return BlocProvider<IconThemesBloc>.value(
    value: bloc,
    child: const Scaffold(
      resizeToAvoidBottomInset: false,
      body: IconThemesPane(),
    ),
  );
}

const List<InstalledIconTheme> _imported = <InstalledIconTheme>[
  InstalledIconTheme(
    id: 'material',
    name: 'Material Icons',
    author: 'Ana',
    license: 'MIT',
    origin: InstalledThemeOrigin.gallery,
  ),
  InstalledIconTheme(
    id: 'mine',
    name: 'My icons',
    author: '',
    license: '',
    origin: InstalledThemeOrigin.own,
  ),
];

void main() {
  for (final brightness in Brightness.values) {
    final name = brightness.name;

    group('the icon themes pictures, $name', () {
      testWidgets('none yet', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await _page(),
          name: 'phone_icon_themes_empty_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('two icon themes and an icon theme just imported', (
        tester,
      ) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await _page(
            themes: _imported,
            files: FixtureFilePickerRepository(<String?>['/tmp/lucide.tar.gz']),
          ),
          name: 'phone_icon_themes_imported_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await tester.tap(find.text('Import…'));
            await tester.pumpAndSettle();
          },
        );
      });

      testWidgets('a failed import', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await _page(
            repository: FixtureThemesRepository()
              ..importFailure = const ApplicationError.invalid(
                message: 'No icons in that file.',
              ),
            files: FixtureFilePickerRepository(<String?>['/tmp/notes.txt']),
          ),
          name: 'phone_icon_themes_import_failed_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await tester.tap(find.text('Import…'));
            await tester.pumpAndSettle();
          },
        );
      });
    });
  }
}
