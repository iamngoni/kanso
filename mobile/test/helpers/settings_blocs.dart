// Kanso — settings_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/library_index_status.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/lock/state/biometrics/biometrics_bloc.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso/push/models/data/push_setting_status.dart';
import 'package:kanso/push/state/push_setting/push_setting_bloc.dart';
import 'package:kanso/reminders/state/reminders_settings/reminders_settings_bloc.dart';
import 'package:kanso/reminders/utils/reminders_coordinator.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/settings/state/discovery/discovery_settings_bloc.dart';
import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';
import 'package:kanso/settings/state/word_goal_settings/word_goal_settings_bloc.dart';
import 'package:kanso/templates/state/daily_template/daily_template_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_about_repository.dart';
import 'fixture_device_lock_repository.dart';
import 'fixture_discovery_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_push_repository.dart';
import 'fixture_reminders_repository.dart';
import 'fixture_session_repository.dart';
import 'fixture_sync_repository.dart';
import 'page_fixtures.dart';
import 'seeded_account_blocs.dart';
import 'seeded_appearance_bloc.dart';
import 'sync_settings_blocs.dart';

/// The BLoCs the settings screens read, read: the version of the app
/// (0.1.0, build 3), the account and a sync status, by default no account,
/// and the setting that wakes the phone, by default on and working.
/// The appearance is the golden helper's.
class SettingsBlocs {
  SettingsBlocs._(
    this.about,
    this.sync,
    this.account,
    this.index,
    this.defaultNotebook,
    this.wordGoal,
    this.session,
    this.device,
    this.lock,
    this.biometrics,
    this.dailyTemplate,
    this.reminders,
    this.fixtures,
    this.push,
    this.discoverySettings,
  );

  final AboutBloc about;
  final SyncStatusBloc sync;
  final SeededAccountBloc account;

  /// The notebooks of the mock library.
  final LibraryIndexBloc index;
  final DefaultNotebookBloc defaultNotebook;
  final WordGoalSettingsBloc wordGoal;

  /// The lock session and the phone's biometrics (see
  /// [FixtureSessionRepository]), and the BLoCs over them.
  final FixtureSessionRepository session;
  final FixtureDeviceLockRepository device;
  final LockStatusBloc lock;
  final BiometricsBloc biometrics;
  final DailyTemplateBloc dailyTemplate;
  final RemindersSettingsBloc reminders;

  /// The fixtures used by the daily template and reminder settings.
  final PageFixtures fixtures;

  /// The setting that wakes the phone.
  final PushSettingBloc push;
  final DiscoverySettingsBloc discoverySettings;

  /// Makes them and waits until they have read. The account is [account],
  /// by default none.
  ///
  /// By default nothing is locked, so there is no Locked notes section; pass
  /// a [session] to have one.
  static Future<SettingsBlocs> start({
    SyncStatus status = signedOutStatus,
    AccountState account = const AccountState.signedOut(),
    FixtureLibraryRepository? library,
    Preferences preferences = const Preferences(),
    FixtureSessionRepository? session,
    FixtureDeviceLockRepository? device,
    PageFixtures? fixtures,
    FixturePushRepository? push,
  }) async {
    final page = fixtures ?? PageFixtures();
    final about = AboutBloc(FixtureAboutRepository())
      ..add(const AboutEvent.started());
    closeBlocLater(about);
    final sync = SyncStatusBloc(FixtureSyncRepository(current: status))
      ..add(const SyncStatusEvent.started());
    closeBlocLater(sync);
    final fixture = library ?? FixtureLibraryRepository();
    addTearDown(() => unawaited(fixture.close()));
    final index = LibraryIndexBloc(fixture)
      ..add(const LibraryIndexEvent.started());
    closeBlocLater(index);
    final preferencesRepository = SeededPreferencesRepository(preferences);
    final defaultNotebook = DefaultNotebookBloc(preferencesRepository)
      ..add(const DefaultNotebookEvent.started());
    closeBlocLater(defaultNotebook);
    final wordGoal = WordGoalSettingsBloc(fixture)
      ..add(const WordGoalSettingsEvent.started());
    closeBlocLater(wordGoal);
    if (about.state.status != LoadStatus.ready) {
      await about.stream.firstWhere(
        (state) => state.status == LoadStatus.ready,
      );
    }
    if (sync.state.status == null) {
      await sync.stream.firstWhere((state) => state.status != null);
    }
    if (index.state.status == LibraryIndexStatus.loading) {
      await index.stream.firstWhere(
        (state) => state.status != LibraryIndexStatus.loading,
      );
    }
    if (wordGoal.state.status != LoadStatus.ready) {
      await wordGoal.stream.firstWhere(
        (state) => state.status == LoadStatus.ready,
      );
    }
    // Its reading is a few turns of the event loop; a seed that is the
    // default emits nothing to wait for.
    for (var turn = 0; turn < 10; turn++) {
      await Future<void>.value();
    }
    final seeded = SeededAccountBloc(AccountBlocState(account: account));
    closeBlocLater(seeded);
    final lockSession =
        session ?? FixtureSessionRepository(hasLockPassword: false);
    final lockDevice = device ?? FixtureDeviceLockRepository();
    final lock = LockStatusBloc(lockSession, lockDevice)
      ..add(const LockStatusEvent.started());
    closeBlocLater(lock);
    await lock.stream.firstWhere((state) => state.loaded);
    final biometrics = BiometricsBloc(lockSession, lockDevice);
    closeBlocLater(biometrics);
    final dailyTemplate = DailyTemplateBloc(page.journal)
      ..add(const DailyTemplateEvent.started());
    closeBlocLater(dailyTemplate);
    final coordinator = RemindersCoordinator(
      preferencesRepository,
      FixtureRemindersRepository(),
      page.notifier,
      fixture,
    );
    addTearDown(() => unawaited(coordinator.dispose()));
    final reminders = RemindersSettingsBloc(
      preferencesRepository,
      page.notifier,
      coordinator,
    )..add(const RemindersSettingsEvent.started());
    closeBlocLater(reminders);
    if (dailyTemplate.state.status != LoadStatus.ready) {
      await dailyTemplate.stream.firstWhere(
        (state) => state.status == LoadStatus.ready,
      );
    }
    if (reminders.state.status != LoadStatus.ready) {
      await reminders.stream.firstWhere(
        (state) => state.status == LoadStatus.ready,
      );
    }
    final setting = PushSettingBloc(push ?? FixturePushRepository())
      ..add(const PushSettingEvent.started());
    closeBlocLater(setting);
    if (setting.state.status == PushSettingStatus.loading) {
      await setting.stream.firstWhere(
        (state) => state.status != PushSettingStatus.loading,
      );
    }
    final discoverySettings = DiscoverySettingsBloc(
      FixtureDiscoveryRepository(),
      SeededPreferencesRepository(preferences),
    )..add(const DiscoverySettingsEvent.started());
    closeBlocLater(discoverySettings);
    return SettingsBlocs._(
      about,
      sync,
      seeded,
      index,
      defaultNotebook,
      wordGoal,
      lockSession,
      lockDevice,
      lock,
      biometrics,
      dailyTemplate,
      reminders,
      page,
      setting,
      discoverySettings,
    );
  }

  /// Providers of them, to put above a screen, including one that is a
  /// route pushed later.
  List<BlocProvider<dynamic>> get providers => <BlocProvider<dynamic>>[
    BlocProvider<AboutBloc>.value(value: about),
    BlocProvider<SyncStatusBloc>.value(value: sync),
    BlocProvider<SyncSettingsBloc>.value(value: startedSyncSettingsBloc()),
    BlocProvider<AccountBloc>.value(value: account),
    BlocProvider<LibraryIndexBloc>.value(value: index),
    BlocProvider<DefaultNotebookBloc>.value(value: defaultNotebook),
    BlocProvider<WordGoalSettingsBloc>.value(value: wordGoal),
    BlocProvider<LockStatusBloc>.value(value: lock),
    BlocProvider<BiometricsBloc>.value(value: biometrics),
    BlocProvider<DailyTemplateBloc>.value(value: dailyTemplate),
    BlocProvider<RemindersSettingsBloc>.value(value: reminders),
    BlocProvider<PushSettingBloc>.value(value: push),
    BlocProvider<DiscoverySettingsBloc>.value(value: discoverySettings),
  ];

  /// [child] with them above it.
  Widget provide(Widget child) =>
      MultiBlocProvider(providers: providers, child: child);
}
