// Kanso — browse_shell_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/app.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/models/data/design_gallery_access.dart';
import 'package:kanso/core/models/data/preferences.dart';
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
import 'package:kanso/core/views/widgets/kanso_drawer_scope.dart';
import 'package:kanso/core/views/widgets/kanso_icon_button.dart';
import 'package:kanso/core/views/widgets/kanso_sheet.dart';
import 'package:kanso/core/views/widgets/kanso_sidebar_row.dart';
import 'package:kanso/crash/state/crash_reports/crash_reports_bloc.dart';
import 'package:kanso/library/views/widgets/library_pane.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso/lock/state/privacy_cover/privacy_cover_bloc.dart';
import 'package:kanso/note/state/note_assist_intent/note_assist_intent_bloc.dart';
import 'package:kanso/note/state/note_mode_intent/note_mode_intent_bloc.dart';
import 'package:kanso/note/views/pages/note_page.dart';
import 'package:kanso/note/views/widgets/no_note_selected_pane.dart';
import 'package:kanso/note/views/widgets/note_top_bar.dart';
import 'package:kanso/note_list/views/pages/note_list_page.dart';
import 'package:kanso/search/views/pages/search_page.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/settings/state/discovery/discovery_settings_bloc.dart';
import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';
import 'package:kanso/settings/views/pages/settings_page.dart';
import 'package:kanso/sharing/state/invitations/invitations_bloc.dart';
import 'package:kanso/shell/state/focus_mode/focus_mode_bloc.dart';
import 'package:kanso/shell/state/library_pane/library_pane_bloc.dart';
import 'package:kanso/shell/state/note_creation/note_creation_bloc.dart';
import 'package:kanso/shell/views/widgets/focus_mode_exit.dart';
import 'package:kanso/shell/views/widgets/library_navigation.dart';
import 'package:kanso/tabs/state/tabs/tabs_bloc.dart';

import '../../../helpers/account_blocs.dart';
import '../../../helpers/app_feature_fixtures.dart';
import '../../../helpers/browse_blocs.dart';
import '../../../helpers/crash_samples.dart';
import '../../../helpers/fixture_discovery_repository.dart';
import '../../../helpers/register_page_blocs.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/sync_settings_blocs.dart';

const Size _landscape = Size(1180, 820);
const Size _portrait = Size(820, 1180);
const Size _narrow = Size(600, 900);
const Size _phone = Size(393, 852);

Future<GoRouter> _pump(
  WidgetTester tester,
  Size size, {
  String at = '/notes',
  bool gallery = false,
  FocusModeBloc? focusMode,
}) async {
  final bloc = SeededAppearanceBloc(const AppearanceState());
  addTearDown(bloc.close);
  final blocs = await BrowseBlocs.start();
  final appFeatures = AppFeatureFixtures()..register();
  addTearDown(appFeatures.dispose);
  registerPageBlocs(blocs.library, null, blocs.undoNotice);
  tester.view
    ..physicalSize = size
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final kansoRouter = KansoRouter(
    DesignGalleryAccess(available: gallery),
    LayoutClassNotifier.starting(
      KansoLayout.classFor(size),
      tabletDevice: size.shortestSide >= KansoLayout.tabletShortestSide,
    ),
  );
  final router = kansoRouter.config;
  addTearDown(router.dispose);
  router.go(at);
  final accounts = AccountBlocs();
  await tester.pumpWidget(
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: bloc),
        BlocProvider<TabsBloc>.value(value: blocs.tabs),
        BlocProvider<NavigationHistoryBloc>.value(value: blocs.history),
        BlocProvider<LibraryIndexBloc>.value(value: blocs.index),
        BlocProvider<InvitationsBloc>.value(value: blocs.invitations),
        BlocProvider<SyncStatusBloc>.value(value: blocs.sync),
        BlocProvider<SyncSettingsBloc>.value(value: startedSyncSettingsBloc()),
        BlocProvider<ListNavigationBloc>.value(value: blocs.listNavigation),
        BlocProvider<LibraryPaneBloc>.value(value: blocs.libraryPane),
        if (focusMode != null)
          BlocProvider<FocusModeBloc>.value(value: focusMode),
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
        BlocProvider<AiStatusBloc>.value(value: blocs.aiStatus),
      ],
      child: App(router: kansoRouter),
    ),
  );
  await tester.pumpAndSettle();
  return router;
}

Future<void> _resize(WidgetTester tester, Size size) async {
  tester.view.physicalSize = size;
  await tester.pumpAndSettle();
}

String _where(GoRouter router) =>
    router.routerDelegate.currentConfiguration.uri.toString();

void main() {
  group('BrowseShell', () {
    testWidgets('three panes: library, list and note', (tester) async {
      await _pump(tester, _landscape, at: '/notebooks/b1/note/n1');
      expect(find.byType(LibraryNavigation), findsOneWidget);
      expect(
        tester.widget<NoteListPage>(find.byType(NoteListPage)).place,
        const ListPlace.notebook(id: 'b1'),
      );
      final note = tester.widget<NotePage>(find.byType(NotePage));
      expect(note.noteId, 'n1');
      expect(note.inPane, isTrue);
      // The panes are the widths of the design, the note has the rest.
      expect(tester.getSize(find.byType(LibraryNavigation)).width, 280);
      expect(tester.getSize(find.byType(NoteListPage)).width, 340);
      expect(
        tester.getSize(find.byType(NotePage)).width,
        _landscape.width - 280 - 340 - 2,
      );
    });

    testWidgets('focus mode leaves only the note, with a way back', (
      tester,
    ) async {
      final focus = FocusModeBloc(
        SeededPreferencesRepository(const Preferences(focusMode: true)),
      )..add(const FocusModeEvent.started());
      addTearDown(focus.close);
      await _pump(
        tester,
        _landscape,
        at: '/notebooks/b1/note/n1',
        focusMode: focus,
      );

      expect(find.byType(LibraryNavigation), findsNothing);
      expect(find.byType(NoteListPage), findsNothing);
      expect(find.byType(NotePage), findsOneWidget);
      expect(find.byType(FocusModeExit), findsOneWidget);
      expect(find.byType(NoteTopBar), findsNothing);
      expect(tester.getSize(find.byType(NotePage)).width, _landscape.width);

      await tester.tap(
        find.descendant(
          of: find.byType(FocusModeExit),
          matching: find.byType(KansoIconButton),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(FocusModeExit), findsNothing);
      expect(find.byType(LibraryNavigation), findsOneWidget);
      expect(find.byType(NoteListPage), findsOneWidget);
      expect(find.byType(NoteTopBar), findsOneWidget);
      expect(focus.state.active, isFalse);
    });

    testWidgets('focus mode leaves when no note is open to show', (
      tester,
    ) async {
      final focus = FocusModeBloc(
        SeededPreferencesRepository(const Preferences(focusMode: true)),
      )..add(const FocusModeEvent.started());
      addTearDown(focus.close);
      await _pump(tester, _landscape, focusMode: focus);

      // There is no note to fill the screen, so the mode leaves rather than
      // hide a page it cannot show by itself.
      expect(focus.state.active, isFalse);
      expect(find.byType(FocusModeExit), findsNothing);
      expect(find.byType(LibraryNavigation), findsOneWidget);
      expect(find.byType(NoNoteSelectedPane), findsOneWidget);
    });

    testWidgets('with no note open the note pane says so', (tester) async {
      await _pump(tester, _landscape);
      expect(find.byType(NoNoteSelectedPane), findsOneWidget);
      expect(find.byType(NotePage), findsNothing);
    });

    testWidgets('two panes: list and note, the library in a drawer', (
      tester,
    ) async {
      await _pump(tester, _portrait, at: '/pinned/note/n1');
      expect(find.byType(LibraryNavigation), findsNothing);
      expect(
        tester.widget<NoteListPage>(find.byType(NoteListPage)).place,
        const ListPlace.pinned(),
      );
      expect(tester.getSize(find.byType(NoteListPage)).width, 340);
      expect(find.byType(NotePage), findsOneWidget);
    });

    testWidgets('one pane: the list, with the note on top of it', (
      tester,
    ) async {
      await _pump(tester, _phone, at: '/pinned/note/n1');
      expect(find.byType(NotePage), findsOneWidget);
      expect(tester.widget<NotePage>(find.byType(NotePage)).inPane, isFalse);
      expect(tester.getSize(find.byType(NotePage)).width, _phone.width);
      expect(find.byType(NoteListPage), findsNothing);
      expect(find.byType(NoteListPage, skipOffstage: false), findsOneWidget);
    });

    testWidgets('a search is the list pane of a wide window', (tester) async {
      await _pump(tester, _landscape, at: '/search?q=tea');
      final search = tester.widget<SearchPage>(find.byType(SearchPage));
      expect(search.query, 'tea');
      expect(find.byType(NoNoteSelectedPane), findsOneWidget);
    });

    testWidgets('resizing from three panes to two to one and back keeps the '
        'open note', (tester) async {
      final router = await _pump(tester, _landscape, at: '/tags/t1/note/n1');

      for (final size in <Size>[_portrait, _narrow, _landscape, _phone]) {
        await _resize(tester, size);
        expect(_where(router), '/tags/t1/note/n1', reason: '$size');
        expect(
          tester.widget<NotePage>(find.byType(NotePage)).noteId,
          'n1',
          reason: '$size',
        );
        expect(
          tester
              .widget<NoteListPage>(
                find.byType(NoteListPage, skipOffstage: false),
              )
              .place,
          const ListPlace.tag(id: 't1'),
          reason: '$size',
        );
        final layout = KansoLayout.classFor(size);
        expect(
          tester.widget<NotePage>(find.byType(NotePage)).inPane,
          layout.index > 0,
          reason: '$size',
        );
        expect(
          find.byType(LibraryNavigation).evaluate().isNotEmpty,
          layout.index == 2,
          reason: '$size',
        );
      }
      // The list the note was opened from is under it again.
      router.pop();
      await tester.pumpAndSettle();
      expect(find.byType(NotePage), findsNothing);
      expect(_where(router), '/tags/t1');
    });

    testWidgets('the settings sheet survives a change of layout', (
      tester,
    ) async {
      final router = await _pump(tester, _landscape, at: '/pinned');
      unawaited(router.push(KansoRoutes.settings));
      await tester.pumpAndSettle();
      expect(find.byType(KansoSheet), findsOneWidget);
      expect(find.byType(SettingsPage), findsOneWidget);

      // Half of a Split View: a narrow window, still a tablet's sheet.
      await _resize(tester, const Size(590, 820));
      expect(find.byType(KansoSheet), findsOneWidget);
      expect(find.byType(SettingsPage), findsOneWidget);
      expect(tester.getSize(find.byType(KansoSheet)).width, 590);

      await _resize(tester, _landscape);
      expect(find.byType(KansoSheet), findsOneWidget);
      expect(find.byType(SettingsPage), findsOneWidget);
      router.pop();
      await tester.pumpAndSettle();
      expect(find.byType(SettingsPage), findsNothing);
      expect(_where(router), '/pinned');
    });

    testWidgets('settings is a sheet on a tablet and a page on a phone', (
      tester,
    ) async {
      var router = await _pump(tester, _portrait);
      unawaited(router.push(KansoRoutes.settings));
      await tester.pumpAndSettle();
      expect(find.byType(KansoSheet), findsOneWidget);
      // A tap on the veil outside the sheet closes it.
      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();
      expect(find.byType(SettingsPage), findsNothing);

      await tester.pumpWidget(const SizedBox.shrink());
      router = await _pump(tester, _phone);
      unawaited(router.push(KansoRoutes.settings));
      await tester.pumpAndSettle();
      expect(find.byType(SettingsPage), findsOneWidget);
      expect(find.byType(KansoSheet), findsNothing);
    });
  });

  group('the library drawer', () {
    testWidgets('the list bar opens it and a row goes to its place', (
      tester,
    ) async {
      final router = await _pump(tester, _phone);
      expect(find.byType(LibraryNavigation), findsNothing);
      await tester.tap(find.bySemanticsLabel('Library'));
      await tester.pumpAndSettle();
      expect(find.byType(LibraryNavigation), findsOneWidget);
      // 85% of the phone's width.
      expect(
        tester.getSize(find.byType(LibraryNavigation)).width,
        closeTo(_phone.width * 0.85 - 1, 1),
      );

      await tester.tap(find.text('Pinned').last);
      await tester.pumpAndSettle();
      expect(_where(router), '/pinned');
      expect(find.byType(LibraryNavigation), findsNothing);
    });

    testWidgets("Android's back closes it before leaving", (tester) async {
      final router = await _pump(tester, _phone, at: '/recent');
      await tester.tap(find.bySemanticsLabel('Library'));
      await tester.pumpAndSettle();
      expect(find.byType(LibraryNavigation), findsOneWidget);

      final handled = await tester.binding.handlePopRoute();
      await tester.pumpAndSettle();
      expect(handled, isTrue);
      expect(find.byType(LibraryNavigation), findsNothing);
      expect(_where(router), '/recent');
    });

    testWidgets('a tap on the veil closes it', (tester) async {
      await _pump(tester, _phone);
      await tester.tap(find.bySemanticsLabel('Library'));
      await tester.pumpAndSettle();
      await tester.tapAt(Offset(_phone.width - 5, 400));
      await tester.pumpAndSettle();
      expect(find.byType(LibraryNavigation), findsNothing);
    });

    testWidgets('dragging from the start edge opens it on the list', (
      tester,
    ) async {
      await _pump(tester, _phone);
      await tester.dragFrom(const Offset(4, 400), const Offset(300, 0));
      await tester.pumpAndSettle();
      expect(find.byType(LibraryNavigation), findsOneWidget);
      await tester.dragFrom(const Offset(250, 400), const Offset(-250, 0));
      await tester.pumpAndSettle();
      expect(find.byType(LibraryNavigation), findsNothing);
    });

    testWidgets('a short drag from the edge lets go and closes it', (
      tester,
    ) async {
      await _pump(tester, _phone);
      await tester.dragFrom(const Offset(4, 400), const Offset(40, 0));
      await tester.pumpAndSettle();
      expect(find.byType(LibraryNavigation), findsNothing);
    });

    testWidgets('has no edge drag on a note, which the edge swipe leaves', (
      tester,
    ) async {
      final router = await _pump(tester, _phone, at: '/notes/note/n1');
      await tester.dragFrom(const Offset(4, 400), const Offset(120, 0));
      await tester.pumpAndSettle();
      // The swipe went to the page's back gesture, not the drawer.
      expect(find.byType(LibraryNavigation), findsNothing);
      expect(_where(router), anyOf('/notes', '/notes/note/n1'));
    });

    testWidgets('two panes: the same drawer, 280 wide', (tester) async {
      await _pump(tester, _portrait);
      await tester.tap(find.bySemanticsLabel('Library'));
      await tester.pumpAndSettle();
      expect(tester.getSize(find.byType(LibraryNavigation)).width, 279);
    });

    testWidgets('three panes have no drawer to open', (tester) async {
      await _pump(tester, _landscape);
      expect(find.bySemanticsLabel('Library'), findsNothing);
      expect(
        KansoDrawerScope.maybeOf(tester.element(find.byType(NoteListPage))),
        isNull,
      );
    });

    testWidgets('settings opens from the library header', (tester) async {
      await _pump(tester, _landscape);
      await tester.tap(find.bySemanticsLabel('Settings'));
      await tester.pumpAndSettle();
      expect(find.byType(SettingsPage), findsOneWidget);
    });

    testWidgets('the design gallery row is there in development only', (
      tester,
    ) async {
      await _pump(tester, _landscape, gallery: true);
      // It is the last row of the library, below its fold.
      await tester.scrollUntilVisible(
        find.text('Design gallery'),
        120,
        scrollable: find
            .descendant(
              of: find.byType(LibraryPane),
              matching: find.byType(Scrollable),
            )
            .first,
      );
      expect(find.text('Design gallery'), findsOneWidget);
      await tester.pumpWidget(const SizedBox.shrink());
      await _pump(tester, _landscape);
      expect(find.text('Design gallery', skipOffstage: false), findsNothing);
    });

    testWidgets('the row of the place being listed is selected', (
      tester,
    ) async {
      // Tall enough for every row of the library to be built.
      await _pump(tester, const Size(1180, 2000), at: '/trash');
      Map<String, bool> selected() => <String, bool>{
        for (final row in tester.widgetList<KansoSidebarRow>(
          find.byType(KansoSidebarRow),
        ))
          row.label: row.selected,
      };
      expect(selected()['Trash'], isTrue);
      expect(selected()['All Notes'], isFalse);
      expect(selected()['Pinned'], isFalse);
      await tester.tap(
        find.descendant(
          of: find.byType(LibraryNavigation),
          matching: find.text('Pinned'),
        ),
      );
      await tester.pumpAndSettle();
      expect(selected()['Pinned'], isTrue);
      expect(selected()['Trash'], isFalse);
    });
  });
}
