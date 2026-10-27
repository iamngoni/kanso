// Kanso — settings_pane_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/app_version.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_icon_set.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';
import 'package:kanso/lock/state/biometrics/biometrics_bloc.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_device_lock_repository.dart';
import '../../../helpers/fixture_session_repository.dart';
import '../../../helpers/fixture_sync_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/seeded_account_blocs.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/settings_blocs.dart';
import '../../../helpers/sync_settings_blocs.dart';

class _Settings {
  _Settings(
    this.router,
    this.appearance,
    this.about,
    this.sync,
    this.syncRepository,
  );

  final GoRouter router;
  final AppearanceBloc appearance;
  final MockAboutRepository about;
  final SyncStatusBloc sync;
  final FixtureSyncRepository syncRepository;
}

Future<_Settings> _pump(
  WidgetTester tester, {
  AppearanceState appearance = const AppearanceState(),
  SyncStatus status = signedOutStatus,
  AccountState account = const AccountState.signedOut(),
  Preferences preferences = const Preferences(),
}) async {
  final about = MockAboutRepository();
  when(about.version).thenAnswer(
    (_) async => const Right<ApplicationError, AppVersion>(
      AppVersion(version: '0.1.0', build: '3'),
    ),
  );
  when(
    about.openWebsite,
  ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
  when(
    about.openReleaseNotes,
  ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
  when(
    about.openHelp,
  ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
  final aboutBloc = AboutBloc(about)..add(const AboutEvent.started());
  final syncRepository = FixtureSyncRepository(current: status);
  final syncBloc = SyncStatusBloc(syncRepository)
    ..add(const SyncStatusEvent.started());
  final appearanceBloc = SeededAppearanceBloc(appearance);
  final lockBloc = LockStatusBloc(
    FixtureSessionRepository(hasLockPassword: false),
    FixtureDeviceLockRepository(),
  )..add(const LockStatusEvent.started());
  final biometricsBloc = BiometricsBloc(
    FixtureSessionRepository(hasLockPassword: false),
    FixtureDeviceLockRepository(),
  );
  closeBlocLater(lockBloc);
  closeBlocLater(biometricsBloc);
  final accountBloc = SeededAccountBloc(AccountBlocState(account: account));
  closeBlocLater(accountBloc);
  closeBlocLater(aboutBloc);
  closeBlocLater(syncBloc);
  closeBlocLater(appearanceBloc);
  final blocs = await SettingsBlocs.start(preferences: preferences);
  final router = await pumpRouted(
    tester,
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: appearanceBloc),
        BlocProvider<SyncStatusBloc>.value(value: syncBloc),
        BlocProvider<SyncSettingsBloc>.value(
          value: startedSyncSettingsBloc(syncRepository),
        ),
        BlocProvider<AccountBloc>.value(value: accountBloc),
        BlocProvider<AboutBloc>.value(value: aboutBloc),
        BlocProvider<LockStatusBloc>.value(value: lockBloc),
        BlocProvider<BiometricsBloc>.value(value: biometricsBloc),
        BlocProvider<LibraryIndexBloc>.value(value: blocs.index),
        BlocProvider<DefaultNotebookBloc>.value(value: blocs.defaultNotebook),
      ],
      child: const SettingsPane(),
    ),
    size: const Size(393, 1400),
  );
  await tester.pump();
  await tester.pump();
  return _Settings(router, appearanceBloc, about, syncBloc, syncRepository);
}

const AccountState _signedIn = AccountState.signedIn(
  username: 'ana',
  server: 'https://example.invalid',
);

const SyncStatus _synced = SyncStatus(
  phase: SyncPhase.synced,
  username: 'ana',
  conflicts: 0,
  skippedCount: 0,
  account: SyncAccount.signedIn(),
);

void main() {
  group('SettingsPane', () {
    testWidgets('has the real rows and none that do not work yet', (
      tester,
    ) async {
      await _pump(tester);
      for (final label in <String>[
        'General',
        'Default notebook',
        'Appearance',
        'Theme',
        'Icons',
        'Fonts',
        'Text size',
        'Preview',
        'Preview text size',
        'Diagrams',
        'Sync',
        'Sign in',
        'About',
        'Version',
        'Website',
        'Release notes',
        'Help',
        'Licences',
      ]) {
        expect(find.text(label), findsOneWidget, reason: label);
      }
      for (final label in <String>[
        'Sign out',
        'Locked notes',
      ]) {
        expect(find.text(label), findsNothing, reason: label);
      }
      expect(find.textContaining('Wake this device'), findsNothing);
    });

    testWidgets('the Default notebook row says the first notebook until one '
        'is chosen, and opens the choice', (tester) async {
      final screen = await _pump(tester);
      expect(find.text('First notebook'), findsOneWidget);
      await tester.tap(find.text('Default notebook'));
      await tester.pumpAndSettle();
      expect(screen.router.state.uri.toString(), '/settings/notebook');
    });

    testWidgets('the Default notebook row names the notebook chosen, but '
        'not one that is gone or locked', (tester) async {
      await _pump(
        tester,
        preferences: const Preferences(defaultNotebookId: MockIds.garden),
      );
      expect(find.text('Garden'), findsOneWidget);
      await _pump(
        tester,
        preferences: const Preferences(defaultNotebookId: 'book:gone'),
      );
      expect(find.text('First notebook'), findsOneWidget);
    });

    testWidgets('the Fonts row sums up the choice and follows it', (
      tester,
    ) async {
      final screen = await _pump(tester);
      expect(
        find.text('Instrument Sans · JetBrains Mono'),
        findsOneWidget,
      );
      screen.appearance
        ..add(const AppearanceEvent.editorFontChanged(KansoFontFamily.system))
        ..add(
          const AppearanceEvent.codeFontChanged(KansoFontFamily.ibmPlexMono),
        );
      await tester.pumpAndSettle();
      expect(
        find.text('System · Instrument Sans · IBM Plex Mono'),
        findsOneWidget,
      );
    });

    testWidgets('the Icons row names the icon theme in use', (tester) async {
      await _pump(tester);
      expect(find.text('Lucide'), findsOneWidget);
      await _pump(
        tester,
        appearance: const AppearanceState(
          iconSet: KansoIconSet(id: 'phosphor', name: 'Phosphor'),
        ),
      );
      expect(find.text('Phosphor'), findsOneWidget);
      expect(find.text('Lucide'), findsNothing);
    });

    testWidgets('the Text size row shows the size chosen', (tester) async {
      final screen = await _pump(tester);
      // The text size and the preview text size.
      expect(find.text('Default'), findsNWidgets(2));
      screen.appearance.add(
        const AppearanceEvent.textSizeChanged(KansoTextSize.larger),
      );
      await tester.pumpAndSettle();
      expect(find.text('Larger'), findsOneWidget);
      expect(find.text('Default'), findsOneWidget);
    });

    testWidgets('the Preview text size row shows the size chosen apart from '
        'the editor’s', (tester) async {
      final screen = await _pump(tester);
      screen.appearance.add(
        const AppearanceEvent.previewTextSizeChanged(KansoTextSize.small),
      );
      await tester.pumpAndSettle();
      expect(find.text('Small'), findsOneWidget);
      expect(find.text('Default'), findsOneWidget);
    });

    testWidgets('the Diagrams row chooses between Pele and Kanso, Pele by '
        'default, with the desktop’s words', (tester) async {
      final screen = await _pump(tester);
      expect(find.text('Pele'), findsOneWidget);
      expect(find.text('Kanso'), findsOneWidget);
      expect(
        find.textContaining(
          'What draws Mermaid diagrams, here and in exports.',
        ),
        findsOneWidget,
      );
      expect(screen.appearance.state.diagramRenderer, DiagramRenderer.pele);
      await tester.tap(find.text('Kanso'));
      await tester.pumpAndSettle();
      expect(screen.appearance.state.diagramRenderer, DiagramRenderer.kanso);
      await tester.tap(find.text('Pele'));
      await tester.pumpAndSettle();
      expect(screen.appearance.state.diagramRenderer, DiagramRenderer.pele);
    });

    testWidgets('the Icon themes and Preview text size rows open their '
        'pages', (tester) async {
      final screen = await _pump(tester);
      await tester.tap(find.text('Icon themes'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/icon-themes');
      screen.router.pop();
      await tester.pumpAndSettle();
      await tester.tap(find.text('Preview text size'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/fonts');
    });

    testWidgets('the Theme row names both themes while the system is '
        'followed, and the one theme otherwise', (tester) async {
      await _pump(tester);
      expect(find.text('Kanso Light, Kanso Dark'), findsOneWidget);

      await _pump(
        tester,
        appearance: AppearanceState(
          selection: const ThemeSelection(followSystem: false, ui: 'nord'),
          light: KansoPalette.kansoDark.copyWith(id: 'nord', name: 'Nord'),
          dark: KansoPalette.kansoDark.copyWith(id: 'nord', name: 'Nord'),
        ),
      );
      expect(find.text('Nord'), findsOneWidget);
    });

    testWidgets('signed out, Sync has a row that signs in and no status', (
      tester,
    ) async {
      final screen = await _pump(tester);
      expect(find.text('Sign in'), findsOneWidget);
      expect(find.text('Status'), findsNothing);
      await tester.tap(find.text('Sign in'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/account/sign-in');
    });

    testWidgets('signed in, Sync has the account, the desktop’s status line '
        'and Sync now', (tester) async {
      final screen = await withClock(
        Clock.fixed(DateTime(2026, 10, 27, 15, 30)),
        () => _pump(
          tester,
          account: const AccountState.signedIn(
            username: 'ana',
            server: 'https://example.invalid',
          ),
          status: SyncStatus(
            phase: SyncPhase.synced,
            username: 'ana',
            lastSyncedMs: DateTime(2026, 10, 27, 15, 25).millisecondsSinceEpoch,
            conflicts: 0,
            skippedCount: 0,
            account: const SyncAccount.signedIn(),
          ),
        ),
      );
      expect(find.text('ana'), findsOneWidget);
      expect(find.text('Synced 5 min ago'), findsOneWidget);
      expect(
        find.text('Kanso syncs every 5 minutes and soon after you edit.'),
        findsOneWidget,
      );
      expect(find.text('Sign in'), findsNothing);

      await tester.tap(find.text('Sync now'));
      await tester.pump();
      expect(screen.syncRepository.syncs, 1);

      await tester.tap(find.text('ana'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/account');
    });

    testWidgets('Sync automatically turns sync off and on, and the status '
        'line follows', (tester) async {
      final screen = await _pump(
        tester,
        account: _signedIn,
        status: _synced,
      );
      expect(find.text('Sync automatically'), findsOneWidget);
      expect(
        find.text('In the background, and soon after you edit.'),
        findsOneWidget,
      );
      await tester.tap(find.byType(KansoSwitch));
      await tester.pump();
      expect(screen.syncRepository.syncSettings.enabled, isFalse);
      expect(find.text('Sync off'), findsOneWidget);
      await tester.tap(find.text('Sync automatically'));
      await tester.pump();
      expect(screen.syncRepository.syncSettings.enabled, isTrue);
    });

    testWidgets('Sync every shows the interval and chooses another in a '
        'sheet', (tester) async {
      final screen = await _pump(
        tester,
        account: _signedIn,
        status: _synced,
      );
      expect(find.text('Sync every'), findsOneWidget);
      expect(find.text('5 minutes'), findsOneWidget);
      await tester.tap(find.text('Sync every'));
      await tester.pumpAndSettle();
      for (final option in fixtureIntervals) {
        expect(find.text(option.label), findsWidgets, reason: option.label);
      }
      await tester.tap(find.text('1 hour'));
      await tester.pumpAndSettle();
      expect(screen.syncRepository.syncSettings.intervalSecs, 3600);
      expect(find.text('1 hour'), findsOneWidget);
      expect(
        find.text('Kanso syncs every hour and soon after you edit.'),
        findsOneWidget,
      );
    });

    testWidgets('locked, Sync has a row that opens the notes on this device', (
      tester,
    ) async {
      final screen = await _pump(
        tester,
        account: const AccountState.locked(
          LockedInfo(
            username: 'ana',
            server: 'https://example.invalid',
            hasPassword: true,
            approvalPending: false,
          ),
        ),
      );
      expect(find.text('Open your notes on this device'), findsOneWidget);
      expect(find.text('Sync now'), findsNothing);
      await tester.tap(find.text('Open your notes on this device'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/account/unlock');
    });

    testWidgets('says under Sync that the notes are not in device backups', (
      tester,
    ) async {
      await _pump(tester);
      expect(
        find.text(
          'The notes on this device aren’t included in device '
          'backups.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('the Version row shows the version and build', (tester) async {
      await _pump(tester);
      expect(find.text('0.1.0 (3)'), findsOneWidget);
    });

    testWidgets('the Website row opens the website', (tester) async {
      final screen = await _pump(tester);
      await tester.tap(find.text('Website'));
      await tester.pump();
      verify(screen.about.openWebsite).called(1);
    });

    testWidgets('the Release notes row opens the release notes', (
      tester,
    ) async {
      final screen = await _pump(tester);
      await tester.tap(find.text('Release notes'));
      await tester.pump();
      verify(screen.about.openReleaseNotes).called(1);
      verifyNever(screen.about.openWebsite);
    });

    testWidgets('the Help row opens the help page', (tester) async {
      final screen = await _pump(tester);
      await tester.tap(find.text('Help'));
      await tester.pump();
      verify(screen.about.openHelp).called(1);
      verifyNever(screen.about.openWebsite);
    });

    testWidgets('the rows go to Theme, Icons, Fonts, Text size and Licences', (
      tester,
    ) async {
      final screen = await _pump(tester);
      await tester.tap(find.text('Theme'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/theme');
      screen.router.pop();
      await tester.pumpAndSettle();
      await tester.tap(find.text('Icons'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/icons');
      screen.router.pop();
      await tester.pumpAndSettle();
      await tester.tap(find.text('Fonts'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/fonts');
      screen.router.pop();
      await tester.pumpAndSettle();
      await tester.tap(find.text('Text size'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/fonts');
      screen.router.pop();
      await tester.pumpAndSettle();
      await tester.tap(find.text('Licences'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/licences');
    });
  });
}
