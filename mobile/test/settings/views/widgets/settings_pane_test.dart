// Kanso — settings_pane_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

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
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_sync_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/settings_blocs.dart';

class _Settings {
  _Settings(this.router, this.appearance, this.about, this.sync);

  final GoRouter router;
  final AppearanceBloc appearance;
  final MockAboutRepository about;
  final SyncStatusBloc sync;
}

Future<_Settings> _pump(
  WidgetTester tester, {
  AppearanceState appearance = const AppearanceState(),
  SyncStatus status = signedOutStatus,
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
  final syncBloc = SyncStatusBloc(FixtureSyncRepository(current: status))
    ..add(const SyncStatusEvent.started());
  final appearanceBloc = SeededAppearanceBloc(appearance);
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
        BlocProvider<AboutBloc>.value(value: aboutBloc),
        BlocProvider<LibraryIndexBloc>.value(value: blocs.index),
        BlocProvider<DefaultNotebookBloc>.value(value: blocs.defaultNotebook),
      ],
      child: const SettingsPane(),
    ),
    size: const Size(393, 1400),
  );
  await tester.pump();
  await tester.pump();
  return _Settings(router, appearanceBloc, about, syncBloc);
}

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
        'Fonts',
        'Text size',
        'Sync',
        'Status',
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
        'Icons',
        'Account',
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

    testWidgets('the Text size row shows the size chosen', (tester) async {
      final screen = await _pump(tester);
      expect(find.text('Default'), findsOneWidget);
      screen.appearance.add(
        const AppearanceEvent.textSizeChanged(KansoTextSize.larger),
      );
      await tester.pumpAndSettle();
      expect(find.text('Larger'), findsOneWidget);
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

    testWidgets('the Sync row shows the live status, Off while signed out', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Off'), findsOneWidget);
      await _pump(
        tester,
        status: const SyncStatus(
          phase: SyncPhase.synced,
          username: 'ana',
          conflicts: 0,
        ),
      );
      expect(find.text('Synced'), findsOneWidget);
      expect(find.text('Off'), findsNothing);
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

    testWidgets('the rows go to Theme, Fonts, Text size and Licences', (
      tester,
    ) async {
      final screen = await _pump(tester);
      await tester.tap(find.text('Theme'));
      await tester.pumpAndSettle();
      expect(topOf(screen.router), '/settings/theme');
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
