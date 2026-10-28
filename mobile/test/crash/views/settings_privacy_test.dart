// Kanso — settings_privacy_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/core/models/data/app_version.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/lock/state/biometrics/biometrics_bloc.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso/push/state/push_setting/push_setting_bloc.dart';
import 'package:kanso/reminders/state/reminders_settings/reminders_settings_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';
import 'package:kanso/settings/state/word_goal_settings/word_goal_settings_bloc.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';
import 'package:kanso/templates/state/daily_template/daily_template_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/close_bloc_later.dart';
import '../../helpers/fixture_device_lock_repository.dart';
import '../../helpers/fixture_library_repository.dart';
import '../../helpers/fixture_push_repository.dart';
import '../../helpers/fixture_session_repository.dart';
import '../../helpers/fixture_sync_repository.dart';
import '../../helpers/mock_repositories.dart';
import '../../helpers/pump_routed.dart';
import '../../helpers/seeded_account_blocs.dart';
import '../../helpers/seeded_appearance_bloc.dart';
import '../../helpers/settings_blocs.dart';
import '../../helpers/sync_settings_blocs.dart';

Future<GoRouter> _pump(WidgetTester tester, {int? patch}) async {
  final about = MockAboutRepository();
  when(about.version).thenAnswer(
    (_) async => Right<ApplicationError, AppVersion>(
      AppVersion(version: '0.9.0', build: '57', patch: patch),
    ),
  );
  final aboutBloc = AboutBloc(about)..add(const AboutEvent.started());
  final syncBloc = SyncStatusBloc(FixtureSyncRepository())
    ..add(const SyncStatusEvent.started());
  final appearance = SeededAppearanceBloc(const AppearanceState());
  final library = FixtureLibraryRepository();
  addTearDown(() => unawaited(library.close()));
  final index = LibraryIndexBloc(library)
    ..add(const LibraryIndexEvent.started());
  final defaultNotebook = DefaultNotebookBloc(
    SeededPreferencesRepository(const Preferences()),
  )..add(const DefaultNotebookEvent.started());
  closeBlocLater(index);
  closeBlocLater(defaultNotebook);
  closeBlocLater(aboutBloc);
  closeBlocLater(syncBloc);
  closeBlocLater(appearance);
  final account = SeededAccountBloc();
  closeBlocLater(account);
  // Nothing is locked, so there is no Locked notes section.
  final session = FixtureSessionRepository(hasLockPassword: false);
  addTearDown(() => unawaited(session.close()));
  final device = FixtureDeviceLockRepository();
  final lock = LockStatusBloc(session, device)
    ..add(const LockStatusEvent.started());
  closeBlocLater(lock);
  final biometrics = BiometricsBloc(session, device);
  closeBlocLater(biometrics);
  final push = PushSettingBloc(FixturePushRepository())
    ..add(const PushSettingEvent.started());
  closeBlocLater(push);
  final settings = await SettingsBlocs.start();
  final router = await pumpRouted(
    tester,
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: appearance),
        BlocProvider<SyncStatusBloc>.value(value: syncBloc),
        BlocProvider<SyncSettingsBloc>.value(value: startedSyncSettingsBloc()),
        BlocProvider<AboutBloc>.value(value: aboutBloc),
        BlocProvider<LibraryIndexBloc>.value(value: index),
        BlocProvider<DefaultNotebookBloc>.value(value: defaultNotebook),
        BlocProvider<WordGoalSettingsBloc>.value(value: settings.wordGoal),
        BlocProvider<AccountBloc>.value(value: account),
        BlocProvider<LockStatusBloc>.value(value: lock),
        BlocProvider<BiometricsBloc>.value(value: biometrics),
        BlocProvider<PushSettingBloc>.value(value: push),
        BlocProvider<DailyTemplateBloc>.value(value: settings.dailyTemplate),
        BlocProvider<RemindersSettingsBloc>.value(value: settings.reminders),
        BlocProvider<AiStatusBloc>.value(value: settings.aiStatus),
      ],
      child: const SettingsPane(),
    ),
    size: const Size(393, 1700),
  );
  await tester.pump();
  await tester.pump();
  return router;
}

Future<void> _showVersion(WidgetTester tester) async {
  await tester.scrollUntilVisible(
    find.text('Version'),
    320,
    scrollable: find.byType(Scrollable).first,
  );
  await tester.pumpAndSettle();
}

void main() {
  group('Settings: Privacy and the patch', () {
    testWidgets('Privacy has the Crash reports row, which opens its page', (
      tester,
    ) async {
      final router = await _pump(tester);
      expect(find.text('Privacy'), findsOneWidget);
      expect(
        find.text('Kanso sends a crash report only when you choose to.'),
        findsOneWidget,
      );
      await tester.scrollUntilVisible(
        find.text('Crash reports'),
        240,
        scrollable: find.byType(Scrollable).last,
      );
      await tester.tap(find.text('Crash reports'));
      await tester.pumpAndSettle();
      expect(topOf(router), '/settings/crash-reports');
    });

    testWidgets('the Version row adds the patch when there is one', (
      tester,
    ) async {
      await _pump(tester, patch: 2);
      await _showVersion(tester);
      expect(find.text('0.9.0 (57), patch 2'), findsOneWidget);
    });

    testWidgets('and is as it was without one', (tester) async {
      await _pump(tester);
      await _showVersion(tester);
      expect(find.text('0.9.0 (57)'), findsOneWidget);
    });
  });
}
