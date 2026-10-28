// Kanso — kanso_router_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/app.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/models/data/design_gallery_access.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/search_scope_kind.dart';
import 'package:kanso/core/router/kanso_router.dart';
import 'package:kanso/core/router/kanso_routes.dart';
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
import 'package:kanso/design/views/pages/design_gallery_page.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso/lock/state/privacy_cover/privacy_cover_bloc.dart';
import 'package:kanso/note/state/note_assist_intent/note_assist_intent_bloc.dart';
import 'package:kanso/note/state/note_mode_intent/note_mode_intent_bloc.dart';
import 'package:kanso/note/views/pages/note_page.dart';
import 'package:kanso/note_list/views/pages/note_list_page.dart';
import 'package:kanso/search/views/pages/search_page.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/settings/state/discovery/discovery_settings_bloc.dart';
import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';
import 'package:kanso/settings/views/pages/fonts_page.dart';
import 'package:kanso/settings/views/pages/licences_page.dart';
import 'package:kanso/settings/views/pages/settings_page.dart';
import 'package:kanso/sharing/state/invitations/invitations_bloc.dart';
import 'package:kanso/shell/state/library_pane/library_pane_bloc.dart';
import 'package:kanso/shell/state/note_creation/note_creation_bloc.dart';
import 'package:kanso/tabs/state/tabs/tabs_bloc.dart';
import 'package:kanso/themes/views/pages/theme_picker_page.dart';

import '../../helpers/account_blocs.dart';
import '../../helpers/app_feature_fixtures.dart';
import '../../helpers/browse_blocs.dart';
import '../../helpers/crash_samples.dart';
import '../../helpers/fixture_discovery_repository.dart';
import '../../helpers/register_page_blocs.dart';
import '../../helpers/seeded_appearance_bloc.dart';
import '../../helpers/sync_settings_blocs.dart';

Future<GoRouter> _pump(
  WidgetTester tester, {
  bool gallery = false,
  String? at,
  Size size = const Size(393, 852),
}) async {
  final bloc = SeededAppearanceBloc(const AppearanceState());
  addTearDown(bloc.close);
  final blocs = await BrowseBlocs.start();
  registerPageBlocs(blocs.library, undo: blocs.undoNotice);
  final appFeatures = AppFeatureFixtures()..register();
  addTearDown(appFeatures.dispose);
  final history = blocs.history;
  tester.view
    ..physicalSize = size
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final kansoRouter = KansoRouter(
    DesignGalleryAccess(available: gallery),
    LayoutClassNotifier.starting(KansoLayout.classFor(size)),
  );
  final router = kansoRouter.config;
  addTearDown(router.dispose);
  if (at != null) {
    router.go(at);
  }
  final accounts = AccountBlocs();
  await tester.pumpWidget(
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: bloc),
        BlocProvider<AiStatusBloc>.value(value: appFeatures.status),
        BlocProvider<TabsBloc>.value(value: blocs.tabs),
        BlocProvider<NavigationHistoryBloc>.value(value: history),
        BlocProvider<LibraryIndexBloc>.value(value: blocs.index),
        BlocProvider<InvitationsBloc>.value(value: blocs.invitations),
        BlocProvider<SyncStatusBloc>.value(value: blocs.sync),
        BlocProvider<SyncSettingsBloc>.value(value: startedSyncSettingsBloc()),
        BlocProvider<ListNavigationBloc>.value(value: blocs.listNavigation),
        BlocProvider<LibraryPaneBloc>.value(value: blocs.libraryPane),
        BlocProvider<LockStatusBloc>.value(value: blocs.lock),
        BlocProvider<NoteAssistIntentBloc>(
          create: (_) => NoteAssistIntentBloc(),
        ),
        BlocProvider<NoteModeIntentBloc>(
          create: (_) => NoteModeIntentBloc(),
        ),
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
        BlocProvider<BackupNoticeBloc>.value(value: hiddenBackupNoticeBloc()),
        BlocProvider<AccountBloc>.value(value: accounts.account),
        BlocProvider<DeviceApprovalBloc>.value(value: accounts.approval),
        BlocProvider<CrashReportsBloc>.value(value: quietCrashReportsBloc()),
      ],
      child: App(router: kansoRouter),
    ),
  );
  await tester.pumpAndSettle();
  return router;
}

String _where(GoRouter router) =>
    router.routerDelegate.currentConfiguration.uri.toString();

/// A link from outside the app, as the platform hands it over.
Future<void> _openLink(WidgetTester tester, String link) async {
  final message = const JSONMethodCodec().encodeMethodCall(
    MethodCall('pushRouteInformation', <String, dynamic>{'location': link}),
  );
  await tester.binding.defaultBinaryMessenger.handlePlatformMessage(
    'flutter/navigation',
    message,
    (_) {},
  );
  await tester.pumpAndSettle();
}

void main() {
  group('KansoRouter', () {
    testWidgets('starts on All Notes', (tester) async {
      final router = await _pump(tester);
      expect(_where(router), '/notes');
      final page = tester.widget<NoteListPage>(find.byType(NoteListPage));
      expect(page.place, const ListPlace.all());
      expect(find.text('All Notes'), findsOneWidget);
    });

    testWidgets('lists each place, notebook and tag', (tester) async {
      final router = await _pump(tester);
      final places = <String, ListPlace>{
        '/pinned': const ListPlace.pinned(),
        '/recent': const ListPlace.recent(),
        '/trash': const ListPlace.trash(),
        '/notebooks/b%3A1': const ListPlace.notebook(id: 'b:1'),
        '/tags/t1': const ListPlace.tag(id: 't1'),
      };
      for (final entry in places.entries) {
        router.go(entry.key);
        await tester.pumpAndSettle();
        expect(
          tester.widget<NoteListPage>(find.byType(NoteListPage)).place,
          entry.value,
          reason: entry.key,
        );
      }
    });

    testWidgets('opens a note over its list and goes back to it', (
      tester,
    ) async {
      final router = await _pump(tester, at: '/notebooks/b1');
      router.go('/notebooks/b1/note/n%3A1');
      await tester.pumpAndSettle();
      expect(
        tester.widget<NotePage>(find.byType(NotePage)).noteId,
        'n:1',
      );
      // The list is below the note: back reveals it.
      expect(find.byType(NoteListPage, skipOffstage: false), findsOneWidget);
      router.pop();
      await tester.pumpAndSettle();
      expect(find.byType(NotePage), findsNothing);
      expect(_where(router), '/notebooks/b1');
    });

    testWidgets('opens a search with its words, scope and notebook', (
      tester,
    ) async {
      await _pump(tester, at: '/search?q=tea+time&scope=notebook&in=b1');
      final page = tester.widget<SearchPage>(find.byType(SearchPage));
      expect(page.query, 'tea time');
      expect(page.scope, SearchScopeKind.notebook);
      expect(page.notebookId, 'b1');
    });

    testWidgets('opens a note found by a search', (tester) async {
      final router = await _pump(tester, at: '/search/note/n1?q=tea');
      expect(tester.widget<NotePage>(find.byType(NotePage)).noteId, 'n1');
      expect(
        tester
            .widget<SearchPage>(find.byType(SearchPage, skipOffstage: false))
            .query,
        'tea',
      );
      router.pop();
      await tester.pumpAndSettle();
      expect(find.byType(NotePage), findsNothing);
    });

    testWidgets('sends the short forms to their places', (tester) async {
      final router = await _pump(tester);
      router.go('/note/n1');
      await tester.pumpAndSettle();
      expect(_where(router), '/notes/note/n1');
      router.go('/notebook/b1');
      await tester.pumpAndSettle();
      expect(_where(router), '/notebooks/b1');
      router.go('/');
      await tester.pumpAndSettle();
      expect(_where(router), '/notes');
    });

    testWidgets('sends an address that is nothing to All Notes', (
      tester,
    ) async {
      final router = await _pump(tester, at: '/pinned');
      router.go('/nowhere/at/all');
      await tester.pumpAndSettle();
      expect(_where(router), '/notes');
      expect(find.byType(NoteListPage), findsOneWidget);
    });

    testWidgets('opens a note from a kanso:// link', (tester) async {
      final router = await _pump(tester);
      await _openLink(tester, 'kanso://open/note/n%3A9');
      expect(_where(router), '/notes/note/n%3A9');
      expect(tester.widget<NotePage>(find.byType(NotePage)).noteId, 'n:9');
    });

    testWidgets('opens a notebook and a search from kanso:// links', (
      tester,
    ) async {
      final router = await _pump(tester);
      await _openLink(tester, 'kanso://open/notebook/b2');
      expect(_where(router), '/notebooks/b2');
      expect(
        tester.widget<NoteListPage>(find.byType(NoteListPage)).place,
        const ListPlace.notebook(id: 'b2'),
      );
      await _openLink(tester, 'kanso://open/search?q=soup');
      expect(_where(router), '/search?q=soup');
      expect(tester.widget<SearchPage>(find.byType(SearchPage)).query, 'soup');
    });

    testWidgets('settings opens over the page it was opened from', (
      tester,
    ) async {
      final router = await _pump(tester, at: '/pinned');
      unawaited(router.push(KansoRoutes.settings));
      await tester.pumpAndSettle();
      expect(find.byType(SettingsPage), findsOneWidget);
      // What was open is still there, below.
      expect(find.byType(NoteListPage, skipOffstage: false), findsOneWidget);

      unawaited(router.push(KansoRoutes.settingsTheme));
      await tester.pumpAndSettle();
      expect(find.byType(ThemePickerPage), findsOneWidget);
      router.pop();
      await tester.pumpAndSettle();
      unawaited(router.push(KansoRoutes.settingsLicences));
      await tester.pumpAndSettle();
      expect(find.byType(LicencesPage), findsOneWidget);
      router.pop();
      await tester.pumpAndSettle();
      expect(find.byType(SettingsPage), findsOneWidget);

      // Closing settings goes back to the list.
      router.pop();
      await tester.pumpAndSettle();
      expect(find.byType(SettingsPage), findsNothing);
      expect(
        tester.widget<NoteListPage>(find.byType(NoteListPage)).place,
        const ListPlace.pinned(),
      );
    });

    testWidgets('Fonts opens at its text size section when asked', (
      tester,
    ) async {
      final router = await _pump(tester);
      unawaited(router.push(KansoRoutes.settings));
      await tester.pumpAndSettle();
      unawaited(router.push(KansoRoutes.fonts()));
      await tester.pumpAndSettle();
      expect(
        tester.widget<FontsPage>(find.byType(FontsPage)).atTextSize,
        isFalse,
      );
      router.pop();
      await tester.pumpAndSettle();
      unawaited(router.push(KansoRoutes.fonts(atSize: true)));
      await tester.pumpAndSettle();
      expect(
        tester.widget<FontsPage>(find.byType(FontsPage)).atTextSize,
        isTrue,
      );
    });

    testWidgets('a settings page can be opened by a link', (tester) async {
      final router = await _pump(tester);
      await _openLink(tester, 'kanso://open/settings/theme');
      expect(_where(router), '/settings/theme');
      expect(find.byType(ThemePickerPage), findsOneWidget);
    });

    testWidgets('the design gallery is there in development only', (
      tester,
    ) async {
      final router = await _pump(tester, gallery: true, at: '/design');
      expect(find.byType(DesignGalleryPage), findsOneWidget);
      expect(_where(router), '/design');
    });

    testWidgets('and not otherwise', (tester) async {
      final router = await _pump(tester, at: '/design');
      expect(find.byType(DesignGalleryPage), findsNothing);
      expect(_where(router), '/notes');
    });
  });
}
