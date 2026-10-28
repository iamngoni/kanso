// Kanso — icon_themes_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/installed_icon_theme.dart';
import 'package:kanso/core/models/data/installed_theme_origin.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/themes/models/data/theme_import_outcome.dart';
import 'package:kanso/themes/state/icon_themes/icon_themes_bloc.dart';

import '../../../helpers/fixture_file_picker_repository.dart';
import '../../../helpers/fixture_themes_repository.dart';

const InstalledIconTheme _material = InstalledIconTheme(
  id: 'material',
  name: 'Material',
  author: 'Ana',
  license: 'MIT',
  origin: InstalledThemeOrigin.gallery,
);

void main() {
  late FixtureThemesRepository themes;
  late FixtureFilePickerRepository files;

  setUp(() {
    themes = FixtureThemesRepository(
      iconThemeList: <InstalledIconTheme>[_material],
    );
    files = FixtureFilePickerRepository();
  });

  IconThemesBloc build() => IconThemesBloc(themes, files);

  group('IconThemesBloc', () {
    test('starts loading, with no icon themes', () {
      expect(build().state, const IconThemesState());
    });

    blocTest<IconThemesBloc, IconThemesState>(
      'started lists the icon themes',
      build: build,
      act: (bloc) => bloc.add(const IconThemesEvent.started()),
      expect: () => const <IconThemesState>[
        IconThemesState(),
        IconThemesState(
          status: LoadStatus.ready,
          themes: <InstalledIconTheme>[_material],
        ),
      ],
    );

    blocTest<IconThemesBloc, IconThemesState>(
      'started with no icon themes is ready and empty',
      build: () => IconThemesBloc(FixtureThemesRepository(), files),
      act: (bloc) => bloc.add(const IconThemesEvent.started()),
      expect: () => const <IconThemesState>[
        IconThemesState(),
        IconThemesState(status: LoadStatus.ready),
      ],
    );

    blocTest<IconThemesBloc, IconThemesState>(
      'imports the file picked, lists the result and says what was added',
      build: build,
      setUp: () => files.willPick('/tmp/lucide.tar.gz'),
      act: (bloc) => bloc.add(const IconThemesEvent.importRequested()),
      verify: (bloc) {
        expect(themes.importedIconThemePaths, <String>['/tmp/lucide.tar.gz']);
        expect(bloc.state.themes.map((theme) => theme.id), <String>[
          'material',
          'lucide',
        ]);
        expect(
          bloc.state.outcome,
          const ThemeImportOutcome.imported(
            names: <String>['Lucide'],
            summary: '12 icons imported.',
          ),
        );
        expect(bloc.state.importing, isFalse);
      },
    );

    blocTest<IconThemesBloc, IconThemesState>(
      'says why when the file cannot be imported',
      build: build,
      setUp: () {
        files.willPick('/tmp/bad.zip');
        themes.importFailure = const ApplicationError.invalid(
          message: 'No icons in it.',
        );
      },
      act: (bloc) => bloc.add(const IconThemesEvent.importRequested()),
      verify: (bloc) {
        expect(
          bloc.state.outcome,
          const ThemeImportOutcome.failed(message: 'No icons in it.'),
        );
        expect(bloc.state.importing, isFalse);
      },
    );

    blocTest<IconThemesBloc, IconThemesState>(
      'does nothing when the picker is closed without a file',
      build: build,
      setUp: () => files.willPick(null),
      act: (bloc) => bloc.add(const IconThemesEvent.importRequested()),
      expect: () => <IconThemesState>[],
    );

    blocTest<IconThemesBloc, IconThemesState>(
      'fails when the icon themes cannot be read',
      build: () => IconThemesBloc(_FailingIconThemes(), files),
      act: (bloc) => bloc.add(const IconThemesEvent.started()),
      expect: () => const <IconThemesState>[
        IconThemesState(),
        IconThemesState(status: LoadStatus.failed),
      ],
    );
  });
}

class _FailingIconThemes extends FixtureThemesRepository {
  @override
  Future<Either<ApplicationError, List<InstalledIconTheme>>>
  iconThemes() async => const Left<ApplicationError, List<InstalledIconTheme>>(
    ApplicationError.internal(message: 'Gone.'),
  );
}
