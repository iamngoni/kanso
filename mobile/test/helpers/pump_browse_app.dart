// Kanso — pump_browse_app.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/app.dart';
import 'package:kanso/backups/state/backups/backups_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/design_gallery_access.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/router/kanso_router.dart';
import 'package:kanso/core/router/layout_class_notifier.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/state/capture_inbox/capture_inbox_bloc.dart';
import 'package:kanso/core/state/device_approval/device_approval_bloc.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/list_navigation/list_navigation_bloc.dart';
import 'package:kanso/core/state/navigation_history/navigation_history_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/crash/state/crash_reports/crash_reports_bloc.dart';
import 'package:kanso/import/models/data/import_pick.dart';
import 'package:kanso/import/models/data/opened_batch.dart';
import 'package:kanso/import/models/repos/abs/import_files_repository.dart';
import 'package:kanso/import/models/repos/abs/import_repository.dart';
import 'package:kanso/import/state/opened_documents/opened_documents_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso/lock/state/privacy_cover/privacy_cover_bloc.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/settings/state/discovery/discovery_settings_bloc.dart';
import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';
import 'package:kanso/sharing/state/invitations/invitations_bloc.dart';
import 'package:kanso/shell/state/library_pane/library_pane_bloc.dart';
import 'package:kanso/shell/state/note_creation/note_creation_bloc.dart';
import 'package:kanso/tabs/state/tabs/tabs_bloc.dart';

import 'account_blocs.dart';
import 'app_feature_fixtures.dart';
import 'browse_blocs.dart';
import 'crash_samples.dart';
import 'fixture_ai_repository.dart';
import 'fixture_device_lock_repository.dart';
import 'fixture_discovery_repository.dart';
import 'fixture_gallery_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_parity_repositories.dart';
import 'fixture_renderer_repository.dart';
import 'fixture_session_repository.dart';
import 'fixture_shared_notes_repository.dart';
import 'fixture_sync_repository.dart';
import 'page_fixtures.dart';
import 'register_page_blocs.dart';
import 'seeded_appearance_bloc.dart';
import 'sync_settings_blocs.dart';

/// The boundary around the app that a picture is taken of.
const Key appBoundaryKey = ValueKey<String>('app-boundary');

/// The app on the mock library, and what a test can look at of it.
class BrowseApp {
  BrowseApp(
    this.router,
    this.blocs,
    this.appearance,
    this.layoutClass,
    this.accounts,
  );

  final GoRouter router;
  final BrowseBlocs blocs;

  /// The BLoCs of the account screens.
  final AccountBlocs accounts;
  final AppearanceBloc appearance;

  /// The layout class the router lays its pages out for.
  final LayoutClassNotifier layoutClass;

  /// Where the router is, as an address.
  String get where => router.routerDelegate.currentConfiguration.uri.toString();
}

/// Pumps the whole app, as `main` builds it but over the mock library, open
/// at [at] in a window of [size], light. [library] is the library to use
/// when a test has made one; [renderer] likewise for the renderer the pages
/// make their BLoCs on.
///
/// [devicePixelRatio] and [safeArea] (in logical pixels) are the window's
/// density and the status bar and home indicator the platform reports.
/// [tabletDevice] says whether the display is a tablet's, which a narrow
/// window of one (a Split View half) is not enough to tell; it defaults to
/// what the window's own size says. [backupNotice] is the notice's BLoC; it
/// defaults to one that is not shown. [accounts] has the account screens'
/// BLoCs; by default the phone has no account. [sync] is the sync the
/// status and the sync settings read. [shared] and [gallery] are
/// the shared notes and the theme gallery the app reads, if a test has made
/// them. [fixtures] supplies the journal, board and reminder fixtures.
/// [session] and [device] are the lock session and the phone's
/// biometrics; by default nothing is locked. The app is inside a repaint
/// boundary, [appBoundaryKey], to take a picture of.
Future<BrowseApp> pumpBrowseApp(
  WidgetTester tester,
  Size size, {
  String at = '/notes',
  AppearanceState appearance = const AppearanceState(),
  FixtureLibraryRepository? library,
  FixtureRendererRepository? renderer,
  FixtureAiRepository? ai,
  double devicePixelRatio = 1,
  EdgeInsets safeArea = EdgeInsets.zero,
  bool? tabletDevice,
  BackupNoticeBloc? backupNotice,
  FixtureSharedNotesRepository? shared,
  FixtureGalleryRepository? gallery,
  AccountBlocs? accounts,
  FixtureSyncRepository? sync,
  FixtureSessionRepository? session,
  FixtureDeviceLockRepository? device,
  OpenedDocumentsBloc? openedDocuments,
  FixtureHistoryRepository? history,
  FixtureReplaceRepository? replace,
  FixtureBackupsRepository? backups,
  PageFixtures? fixtures,
}) async {
  final bloc = SeededAppearanceBloc(appearance);
  addTearDown(bloc.close);
  final blocs = await BrowseBlocs.start(
    library,
    sync,
    shared,
    session,
    device,
  );
  registerPageBlocs(
    blocs.library,
    renderer,
    blocs.undoNotice,
    gallery,
    blocs.shared,
    null,
    history,
    replace,
    backups,
    fixtures,
  );
  final appFeatures = AppFeatureFixtures(ai: ai)..register();
  addTearDown(appFeatures.dispose);
  final accountBlocs = (accounts ?? AccountBlocs())..register();
  final opened =
      openedDocuments ??
      OpenedDocumentsBloc(
        _IdleImportRepository(),
        _IdleImportFilesRepository(),
      );
  if (openedDocuments == null) {
    opened.add(const OpenedDocumentsEvent.started());
    addTearDown(opened.close);
  }
  final padding = FakeViewPadding(
    left: safeArea.left * devicePixelRatio,
    top: safeArea.top * devicePixelRatio,
    right: safeArea.right * devicePixelRatio,
    bottom: safeArea.bottom * devicePixelRatio,
  );
  tester.view
    ..devicePixelRatio = devicePixelRatio
    ..physicalSize = size * devicePixelRatio
    ..padding = padding
    ..viewPadding = padding;
  addTearDown(tester.view.reset);
  final kansoRouter = KansoRouter(
    const DesignGalleryAccess(available: false),
    LayoutClassNotifier.starting(
      KansoLayout.classFor(size),
      tabletDevice:
          tabletDevice ?? size.shortestSide >= KansoLayout.tabletShortestSide,
    ),
  );
  final router = kansoRouter.config;
  addTearDown(router.dispose);
  router.go(at);
  await tester.pumpWidget(
    RepaintBoundary(
      key: appBoundaryKey,
      child: MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<AppearanceBloc>.value(value: bloc),
          BlocProvider<AiStatusBloc>.value(value: appFeatures.status),
          BlocProvider<TabsBloc>.value(value: blocs.tabs),
          BlocProvider<TabsBloc>.value(value: blocs.tabs),
          BlocProvider<NavigationHistoryBloc>.value(value: blocs.history),
          BlocProvider<LibraryIndexBloc>.value(value: blocs.index),
          BlocProvider<InvitationsBloc>.value(value: blocs.invitations),
          BlocProvider<SyncStatusBloc>.value(value: blocs.sync),
          BlocProvider<SyncSettingsBloc>.value(
            value: startedSyncSettingsBloc(blocs.syncRepository),
          ),
          BlocProvider<ListNavigationBloc>.value(value: blocs.listNavigation),
          BlocProvider<LibraryPaneBloc>.value(value: blocs.libraryPane),
          BlocProvider<LockStatusBloc>.value(value: blocs.lock),
          BlocProvider<PrivacyCoverBloc>.value(value: blocs.cover),
          BlocProvider<UndoNoticeBloc>.value(value: blocs.undoNotice),
          BlocProvider<NoteCreationBloc>.value(value: blocs.creation),
          BlocProvider<DefaultNotebookBloc>.value(value: blocs.defaultNotebook),
          BlocProvider<DiscoverySettingsBloc>(
            lazy: false,
            create: (_) => DiscoverySettingsBloc(
              FixtureDiscoveryRepository(),
              SeededPreferencesRepository(const Preferences()),
            )..add(const DiscoverySettingsEvent.started()),
          ),
          BlocProvider<CaptureInboxBloc>.value(value: blocs.captureInbox),
          BlocProvider<BackupNoticeBloc>.value(
            value: backupNotice ?? hiddenBackupNoticeBloc(),
          ),
          BlocProvider<AccountBloc>.value(value: accountBlocs.account),
          BlocProvider<DeviceApprovalBloc>.value(
            value: accountBlocs.approval,
          ),
          BlocProvider<CrashReportsBloc>.value(value: quietCrashReportsBloc()),
          BlocProvider<OpenedDocumentsBloc>.value(value: opened),
          BlocProvider<BackupsBloc>(
            create: (_) =>
                getIt<BackupsBloc>()..add(const BackupsEvent.started()),
          ),
        ],
        child: App(router: kansoRouter),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return BrowseApp(
    router,
    blocs,
    bloc,
    kansoRouter.layoutClass,
    accountBlocs,
  );
}

class _IdleImportRepository implements ImportRepository {
  @override
  List<ImportSource> sources() => const <ImportSource>[];

  @override
  Either<ApplicationError, List<String>> kindsFor(String path) =>
      const Right<ApplicationError, List<String>>(<String>[]);

  @override
  Stream<ImportEvent> plan(String kind, String path) =>
      const Stream<ImportEvent>.empty();

  @override
  Stream<ImportEvent> run(String kind, String path) =>
      const Stream<ImportEvent>.empty();

  @override
  Future<Either<ApplicationError, OpenedDocuments>> openDocuments(
    List<OpenedFile> files,
    String staging,
  ) async => const Right<ApplicationError, OpenedDocuments>(
    OpenedDocuments(problems: <String>[]),
  );
}

class _IdleImportFilesRepository implements ImportFilesRepository {
  @override
  Stream<Unit> get arrivals => const Stream<Unit>.empty();

  @override
  Future<Either<ApplicationError, ImportPick?>> pickFile(
    List<String> extensions,
  ) async => const Right<ApplicationError, ImportPick?>(null);

  @override
  Future<Either<ApplicationError, ImportPick?>> pickFolder() async =>
      const Right<ApplicationError, ImportPick?>(null);

  @override
  Future<Either<ApplicationError, Unit>> clearPicked() async =>
      const Right<ApplicationError, Unit>(unit);

  @override
  Future<Either<ApplicationError, OpenedBatch>> takeOpened() async =>
      const Right<ApplicationError, OpenedBatch>(
        OpenedBatch(files: <OpenedFile>[], staging: ''),
      );

  @override
  Future<Either<ApplicationError, Unit>> discardOpened() async =>
      const Right<ApplicationError, Unit>(unit);
}
