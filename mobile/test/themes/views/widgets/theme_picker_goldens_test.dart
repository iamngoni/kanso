// Kanso — theme_picker_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/theme_file_notice.dart';
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/themes/views/widgets/theme_picker_pane.dart';

import '../../../helpers/fixture_file_picker_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/real_themes.dart';
import '../../../helpers/theme_picker_blocs.dart';

Widget _page(Widget child) {
  return Scaffold(resizeToAvoidBottomInset: false, body: child);
}

void main() {
  loadRealThemes();

  for (final brightness in Brightness.values) {
    final name = brightness.name;

    group('the theme picker pictures, $name', () {
      testWidgets('following the system', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await withThemePicker(_page(const ThemePickerPane())),
          name: 'phone_theme_picker_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('one theme, the system not followed', (tester) async {
        final nord = realPalette('nord');
        await expectGoldenOnBothPlatforms(
          tester,
          await withThemePicker(_page(const ThemePickerPane())),
          name: 'phone_theme_picker_one_theme_$name',
          size: phoneSize,
          brightness: brightness,
          palette: nord,
          appearance: AppearanceState(
            selection: const ThemeSelection(followSystem: false, ui: 'nord'),
            light: nord,
            dark: nord,
          ),
        );
      });

      testWidgets('all three sections and the Theme files, with a syntax '
          'and a preview theme chosen and a file imported', (tester) async {
        final nord = realPalette('nord');
        await expectGoldenOnBothPlatforms(
          tester,
          await withThemePicker(
            _page(const ThemePickerPane()),
            themes: showcaseThemesRepository(
              notices: const <ThemeFileNotice>[
                ThemeFileNotice(
                  file: 'broken.json',
                  sentence: 'Isn’t a theme file Kanso can read.',
                ),
              ],
            ),
            files: FixtureFilePickerRepository(<String?>[
              '/tmp/solarized.tgz',
            ]),
          ),
          name: 'phone_theme_picker_sections_$name',
          size: const Size(393, 1560),
          brightness: brightness,
          palette: nord,
          appearance: AppearanceState(
            selection: const ThemeSelection(
              followSystem: false,
              ui: 'nord',
              syntax: 'dracula',
              preview: 'github-light',
            ),
            light: nord,
            dark: nord,
          ),
          interact: (tester) async {
            await tester.tap(find.text('Import Theme…'));
            await tester.pumpAndSettle();
          },
        );
      });

      testWidgets('a failed import', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await withThemePicker(
            _page(const ThemePickerPane()),
            themes: showcaseThemesRepository()
              ..importFailure = const ApplicationError.invalid(
                message: 'That isn’t a theme package or a Zed theme.',
              ),
            files: FixtureFilePickerRepository(<String?>['/tmp/notes.txt']),
          ),
          name: 'phone_theme_picker_import_failed_$name',
          size: const Size(393, 1560),
          brightness: brightness,
          interact: (tester) async {
            await tester.tap(find.text('Import Theme…'));
            await tester.pumpAndSettle();
          },
        );
      });
    });
  }
}
