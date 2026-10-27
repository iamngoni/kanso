// Kanso — theme_picker_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/installed_theme.dart';
import 'package:kanso/core/models/data/installed_theme_origin.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/models/data/theme_file_notice.dart';
import 'package:kanso/themes/models/data/theme_import_outcome.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';
import 'package:kanso/themes/utils/theme_preview_from_installed.dart';

import '../../../helpers/fixture_file_picker_repository.dart';
import '../../../helpers/fixture_themes_repository.dart';
import '../../../helpers/theme_tokens.dart';

InstalledTheme _theme(
  String id, {
  bool dark = false,
  InstalledThemeOrigin origin = InstalledThemeOrigin.builtin,
}) {
  return InstalledTheme(
    id: id,
    name: id.toUpperCase(),
    author: 'Someone',
    license: 'MIT',
    dark: dark,
    origin: origin,
    tokens: tokensFor(fakePalette(id, dark: dark)),
  );
}

void main() {
  late FixtureThemesRepository themes;
  late FixtureFilePickerRepository files;
  final installed = <InstalledTheme>[
    _theme('kanso-light'),
    _theme('nord', dark: true),
    _theme('mine', origin: InstalledThemeOrigin.own),
    _theme('gallery-one', origin: InstalledThemeOrigin.gallery),
  ];
  const notice = ThemeFileNotice(
    file: 'broken.json',
    sentence: 'Isn’t a theme file.',
  );

  setUp(() {
    themes = FixtureThemesRepository(
      themes: List<InstalledTheme>.of(installed),
      themeNotices: const <ThemeFileNotice>[notice],
    );
    files = FixtureFilePickerRepository();
  });

  ThemePickerBloc build() => ThemePickerBloc(themes, files);

  group('ThemePickerBloc', () {
    test('starts loading, with no themes', () {
      final state = build().state;
      expect(state.status, LoadStatus.loading);
      expect(state.themes, isEmpty);
      expect(state.notices, isEmpty);
      expect(state.importing, isFalse);
      expect(state.outcome, isNull);
    });

    blocTest<ThemePickerBloc, ThemePickerState>(
      'started offers every installed theme in order, previewed from their '
      'tokens, with the files that were left out',
      build: build,
      act: (bloc) => bloc.add(const ThemePickerEvent.started()),
      expect: () => <ThemePickerState>[
        const ThemePickerState(),
        ThemePickerState(
          status: LoadStatus.ready,
          themes: installed.map(themePreviewFromInstalled).toList(),
          notices: const <ThemeFileNotice>[notice],
        ),
      ],
      verify: (bloc) {
        expect(bloc.state.themes.map((theme) => theme.id), <String>[
          'kanso-light',
          'nord',
          'mine',
          'gallery-one',
        ]);
        expect(bloc.state.themes[1].dark, isTrue);
        expect(bloc.state.themes.map((theme) => theme.origin), <Object>[
          InstalledThemeOrigin.builtin,
          InstalledThemeOrigin.builtin,
          InstalledThemeOrigin.own,
          InstalledThemeOrigin.gallery,
        ]);
      },
    );

    blocTest<ThemePickerBloc, ThemePickerState>(
      'started fails when the themes cannot be read, and reads them again '
      'when asked to try again',
      build: build,
      act: (bloc) async {
        themes.listFailure = const ApplicationError.internal(
          message: 'No themes.',
        );
        bloc.add(const ThemePickerEvent.started());
        await Future<void>.delayed(Duration.zero);
        themes.listFailure = null;
        bloc.add(const ThemePickerEvent.started());
      },
      expect: () => <ThemePickerState>[
        const ThemePickerState(),
        const ThemePickerState(status: LoadStatus.failed),
        const ThemePickerState(),
        ThemePickerState(
          status: LoadStatus.ready,
          themes: installed.map(themePreviewFromInstalled).toList(),
          notices: const <ThemeFileNotice>[notice],
        ),
      ],
    );

    group('importing', () {
      blocTest<ThemePickerBloc, ThemePickerState>(
        'imports the file picked, reloads the themes and says what was '
        'added',
        build: build,
        seed: () => ThemePickerState(
          status: LoadStatus.ready,
          themes: installed.map(themePreviewFromInstalled).toList(),
        ),
        setUp: () => files.willPick('/tmp/solarized.tgz'),
        act: (bloc) => bloc.add(const ThemePickerEvent.importRequested()),
        verify: (bloc) {
          expect(themes.importedThemePaths, <String>['/tmp/solarized.tgz']);
          final state = bloc.state;
          expect(state.importing, isFalse);
          expect(state.themes.map((theme) => theme.id), contains('solarized'));
          expect(state.themes.last.origin, InstalledThemeOrigin.own);
          expect(
            state.outcome,
            const ThemeImportOutcome.imported(names: <String>['Solarized']),
          );
        },
        expect: () => <Matcher>[
          isA<ThemePickerState>()
              .having((s) => s.importing, 'importing', isTrue)
              .having((s) => s.outcome, 'outcome', isNull),
          isA<ThemePickerState>().having(
            (s) => s.outcome,
            'outcome',
            isA<ThemeImportedOutcome>(),
          ),
        ],
      );

      blocTest<ThemePickerBloc, ThemePickerState>(
        'says why when the file cannot be imported, and keeps the themes',
        build: build,
        seed: () => ThemePickerState(
          status: LoadStatus.ready,
          themes: installed.map(themePreviewFromInstalled).toList(),
        ),
        setUp: () {
          files.willPick('/tmp/nothing.json');
          themes.importFailure = const ApplicationError.invalid(
            message: 'That isn’t a Zed theme.',
          );
        },
        act: (bloc) => bloc.add(const ThemePickerEvent.importRequested()),
        verify: (bloc) {
          expect(
            bloc.state.outcome,
            const ThemeImportOutcome.failed(message: 'That isn’t a Zed theme.'),
          );
          expect(bloc.state.importing, isFalse);
          expect(bloc.state.themes, hasLength(installed.length));
        },
      );

      blocTest<ThemePickerBloc, ThemePickerState>(
        'does nothing when the picker is closed without a file',
        build: build,
        setUp: () => files.willPick(null),
        act: (bloc) => bloc.add(const ThemePickerEvent.importRequested()),
        expect: () => <ThemePickerState>[],
        verify: (_) {
          expect(files.asked, 1);
          expect(themes.importedThemePaths, isEmpty);
        },
      );

      blocTest<ThemePickerBloc, ThemePickerState>(
        'says so when the picker itself fails',
        build: build,
        setUp: () => files.failure = const ApplicationError.unexpected(
          message: 'No access to the Files app.',
        ),
        act: (bloc) => bloc.add(const ThemePickerEvent.importRequested()),
        expect: () => const <ThemePickerState>[
          ThemePickerState(
            outcome: ThemeImportOutcome.failed(
              message: 'No access to the Files app.',
            ),
          ),
        ],
        verify: (_) => expect(themes.importedThemePaths, isEmpty),
      );

      blocTest<ThemePickerBloc, ThemePickerState>(
        'a new import clears the last outcome while it runs',
        build: build,
        seed: () => const ThemePickerState(
          status: LoadStatus.ready,
          outcome: ThemeImportOutcome.failed(message: 'Old.'),
        ),
        setUp: () => files.willPick('/tmp/ok.json'),
        act: (bloc) => bloc.add(const ThemePickerEvent.importRequested()),
        verify: (bloc) =>
            expect(bloc.state.outcome, isA<ThemeImportedOutcome>()),
        expect: () => <Matcher>[
          isA<ThemePickerState>().having((s) => s.outcome, 'outcome', isNull),
          isA<ThemePickerState>().having(
            (s) => s.outcome,
            'outcome',
            isA<ThemeImportedOutcome>(),
          ),
        ],
      );
    });
  });
}
