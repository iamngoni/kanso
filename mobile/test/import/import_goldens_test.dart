// Kanso: import_goldens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/router/kanso_routes.dart';
import 'package:kanso/import/state/import_flow/import_flow_bloc.dart';
import 'package:kanso/injection.dart';

import '../helpers/app_golden.dart';
import '../helpers/kanso_viewport.dart';
import 'fixture_import_repositories.dart';

/// A steady import screen shown in the whole-app viewport goldens.
class _ImportScreen {
  const _ImportScreen(this.name, {this.failed = false, this.finished = false});

  final String name;
  final bool failed;
  final bool finished;
}

const List<_ImportScreen> _screens = <_ImportScreen>[
  _ImportScreen('sources'),
  _ImportScreen('summary'),
  _ImportScreen('finished', finished: true),
  _ImportScreen('failed', failed: true),
];

const List<KansoViewport> _viewports = <KansoViewport>[
  KansoViewport.phone,
  KansoViewport.iPadPortrait,
  KansoViewport.iPadLandscape,
  KansoViewport.androidTabletPortrait,
  KansoViewport.androidTablet,
];

void _registerImport(FixtureImportRepository repository) {
  if (getIt.isRegistered<ImportFlowBloc>()) {
    getIt.unregister<ImportFlowBloc>();
  }
  getIt.registerFactory<ImportFlowBloc>(
    () => ImportFlowBloc(repository, FixtureImportFilesRepository()),
  );
  addTearDown(() {
    if (getIt.isRegistered<ImportFlowBloc>()) {
      getIt.unregister<ImportFlowBloc>();
    }
  });
}

void main() {
  for (final viewport in _viewports) {
    for (final brightness in Brightness.values) {
      for (final screen in _screens) {
        testWidgets(
          '${screen.name}, ${viewport.slug}, ${brightness.name}',
          (tester) async {
            final repository = FixtureImportRepository(
              planEvents: screen.failed
                  ? const <ImportEvent>[
                      ImportEvent.failed('The archive is damaged.'),
                    ]
                  : const <ImportEvent>[
                      ImportEvent.done(sampleImportSummary),
                    ],
            );
            _registerImport(repository);

            await expectAppGoldenOnBothPlatforms(
              tester,
              name: 'import_${screen.name}_${viewport.slug}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              at: screen.name == 'sources'
                  ? KansoRoutes.import
                  : KansoRoutes.importFile(
                      sampleImportPick.path,
                      kind: 'obsidian',
                    ),
              interact: screen.finished
                  ? (tester, _) async {
                      await tester.tap(find.text('Import').last);
                    }
                  : null,
            );
          },
        );
      }
    }
  }
}
