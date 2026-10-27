// Kanso: import_flow_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/router/kanso_routes.dart';
import 'package:kanso/import/models/data/opened_batch.dart';
import 'package:kanso/import/state/import_flow/import_flow_bloc.dart';
import 'package:kanso/import/state/opened_documents/opened_documents_bloc.dart';
import 'package:kanso/injection.dart';

import '../helpers/kanso_viewport.dart';
import '../helpers/pump_browse_app.dart';
import 'fixture_import_repositories.dart';

void _registerImport(
  FixtureImportRepository repository,
  FixtureImportFilesRepository files,
) {
  if (getIt.isRegistered<ImportFlowBloc>()) {
    getIt.unregister<ImportFlowBloc>();
  }
  getIt.registerFactory<ImportFlowBloc>(
    () => ImportFlowBloc(repository, files),
  );
  addTearDown(() {
    if (getIt.isRegistered<ImportFlowBloc>()) {
      getIt.unregister<ImportFlowBloc>();
    }
  });
}

void main() {
  group('ImportPage', () {
    testWidgets('Settings starts an import', (tester) async {
      final repository = FixtureImportRepository();
      final files = FixtureImportFilesRepository();
      _registerImport(repository, files);

      final app = await pumpBrowseApp(
        tester,
        KansoViewport.phone.size,
        at: '/settings',
        devicePixelRatio: KansoViewport.phone.devicePixelRatio,
        safeArea: KansoViewport.phone.safeArea,
      );
      final importRow = find.text('Import notes').first;
      await tester.ensureVisible(importRow);
      await tester.tap(importRow);
      await tester.pumpAndSettle();
      expect(app.router.state.uri.toString(), KansoRoutes.import);
      expect(find.text('Import from'), findsOneWidget);
    });

    testWidgets('choosing a file previews it before importing it', (
      tester,
    ) async {
      final repository = FixtureImportRepository();
      final files = FixtureImportFilesRepository();
      _registerImport(repository, files);

      final app = await pumpBrowseApp(
        tester,
        KansoViewport.phone.size,
        at: KansoRoutes.import,
        devicePixelRatio: KansoViewport.phone.devicePixelRatio,
        safeArea: KansoViewport.phone.safeArea,
      );

      expect(find.text('Obsidian'), findsOneWidget);
      expect(find.text('Markdown'), findsOneWidget);
      await tester.tap(find.text('Obsidian'));
      await tester.pumpAndSettle();
      expect(find.text('Import from Obsidian'), findsOneWidget);

      await tester.tap(find.text('Choose a file'));
      await tester.pumpAndSettle();
      expect(files.calls, <String>['pickFile zip']);
      expect(repository.calls, <String>[
        'plan obsidian ${sampleImportPick.path}',
      ]);
      expect(find.text('Ready to import'), findsOneWidget);
      expect(find.text('From Export.zip'), findsOneWidget);

      await tester.tap(find.text('Import').last);
      await tester.pumpAndSettle();
      expect(repository.calls.last, 'run obsidian ${sampleImportPick.path}');
      expect(find.text('Import finished'), findsOneWidget);

      await tester.tap(find.text('Open the first note'));
      await tester.pumpAndSettle();
      expect(app.where, '/notes/note/first-note');
    });

    testWidgets('choosing a folder uses the folder picker', (tester) async {
      final repository = FixtureImportRepository();
      final files = FixtureImportFilesRepository();
      _registerImport(repository, files);

      await pumpBrowseApp(
        tester,
        KansoViewport.phone.size,
        at: KansoRoutes.import,
        devicePixelRatio: KansoViewport.phone.devicePixelRatio,
        safeArea: KansoViewport.phone.safeArea,
      );
      await tester.tap(find.text('Obsidian'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Choose a folder'));
      await tester.pumpAndSettle();

      expect(files.calls, <String>['pickFolder']);
      expect(find.text('Ready to import'), findsOneWidget);
    });

    testWidgets('a cancelled picker stays on the source list', (tester) async {
      final repository = FixtureImportRepository();
      final files = FixtureImportFilesRepository(pick: null);
      _registerImport(repository, files);

      await pumpBrowseApp(
        tester,
        KansoViewport.phone.size,
        at: KansoRoutes.import,
        devicePixelRatio: KansoViewport.phone.devicePixelRatio,
        safeArea: KansoViewport.phone.safeArea,
      );
      await tester.tap(find.text('Obsidian'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Choose a file'));
      await tester.pumpAndSettle();

      expect(find.text('Obsidian'), findsOneWidget);
      expect(repository.calls, isEmpty);
    });

    testWidgets('a failed read can be retried', (tester) async {
      final repository = FixtureImportRepository(
        planEvents: const <ImportEvent>[
          ImportEvent.failed('The archive is damaged.'),
        ],
      );
      final files = FixtureImportFilesRepository();
      _registerImport(repository, files);

      await pumpBrowseApp(
        tester,
        KansoViewport.phone.size,
        at: KansoRoutes.import,
        devicePixelRatio: KansoViewport.phone.devicePixelRatio,
        safeArea: KansoViewport.phone.safeArea,
      );
      await tester.tap(find.text('Obsidian'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Choose a file'));
      await tester.pumpAndSettle();
      expect(find.text('The import didn’t finish'), findsOneWidget);
      expect(find.text('The archive is damaged.'), findsOneWidget);

      repository.planEvents = const <ImportEvent>[
        ImportEvent.done(sampleImportSummary),
      ];
      await tester.tap(find.text('Try again'));
      await tester.pumpAndSettle();
      expect(repository.calls, <String>[
        'plan obsidian ${sampleImportPick.path}',
        'plan obsidian ${sampleImportPick.path}',
      ]);
      expect(find.text('Ready to import'), findsOneWidget);
    });

    testWidgets('an opened document becomes a note and shows problems', (
      tester,
    ) async {
      final files = FixtureImportFilesRepository(
        openedBatch: const OpenedBatch(
          files: <OpenedFile>[
            OpenedFile(
              path: '/cache/opened/Note.md',
              name: 'Note.md',
              identity: 'content://provider/note',
            ),
          ],
          staging: '/cache/opened-staging',
        ),
      );
      final repository = FixtureImportRepository(
        openedResult: const Right<ApplicationError, OpenedDocuments>(
          OpenedDocuments(
            noteId: 'opened-note',
            problems: <String>['broken.md: The file could not be read.'],
          ),
        ),
      );
      final opened = OpenedDocumentsBloc(repository, files);
      addTearDown(opened.close);

      final app = await pumpBrowseApp(
        tester,
        KansoViewport.phone.size,
        openedDocuments: opened,
        devicePixelRatio: KansoViewport.phone.devicePixelRatio,
        safeArea: KansoViewport.phone.safeArea,
      );

      opened.add(const OpenedDocumentsEvent.started());
      await tester.pumpAndSettle();
      expect(repository.calls, <String>['openDocuments 1']);
      expect(app.router.state.uri.toString(), '/notes/note/opened-note');
      expect(find.text('Some files weren’t opened'), findsOneWidget);
      expect(
        find.text('broken.md: The file could not be read.'),
        findsOneWidget,
      );
      expect(files.calls, <String>['takeOpened', 'discardOpened']);

      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();
      expect(find.text('Some files weren’t opened'), findsNothing);
    });

    testWidgets('a supplied export narrows the choices and reads directly', (
      tester,
    ) async {
      final repository = FixtureImportRepository();
      final files = FixtureImportFilesRepository();
      _registerImport(repository, files);

      await pumpBrowseApp(
        tester,
        KansoViewport.phone.size,
        at: KansoRoutes.importFile(sampleImportPick.path, kind: 'obsidian'),
        devicePixelRatio: KansoViewport.phone.devicePixelRatio,
        safeArea: KansoViewport.phone.safeArea,
      );

      expect(find.text('Ready to import'), findsOneWidget);
      expect(files.calls, isEmpty);
      expect(repository.calls, <String>[
        'plan obsidian ${sampleImportPick.path}',
      ]);
    });
  });
}
