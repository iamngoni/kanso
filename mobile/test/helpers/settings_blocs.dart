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
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_about_repository.dart';
import 'fixture_device_lock_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_session_repository.dart';
import 'fixture_sync_repository.dart';
import 'seeded_account_blocs.dart';
import 'seeded_appearance_bloc.dart';
import 'sync_settings_blocs.dart';

/// The BLoCs the settings screens read, read: the version of the app
/// (0.1.0, build 3), the account and a sync status, by default no account.
/// The appearance is the golden helper's.
class SettingsBlocs {
  SettingsBlocs._(
    this.about,
    this.sync,
    this.account,
    this.index,
    this.defaultNotebook,
    this.session,
    this.device,
    this.lock,
    this.biometrics,
  );

  final AboutBloc about;
  final SyncStatusBloc sync;
  final SeededAccountBloc account;

  /// The notebooks of the mock library.
  final LibraryIndexBloc index;
  final DefaultNotebookBloc defaultNotebook;

  /// The lock session and the phone's biometrics (see
  /// [FixtureSessionRepository]), and the BLoCs over them.
  final FixtureSessionRepository session;
  final FixtureDeviceLockRepository device;
  final LockStatusBloc lock;
  final BiometricsBloc biometrics;

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
  }) async {
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
    final defaultNotebook = DefaultNotebookBloc(
      SeededPreferencesRepository(preferences),
    )..add(const DefaultNotebookEvent.started());
    closeBlocLater(defaultNotebook);
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
    return SettingsBlocs._(
      about,
      sync,
      seeded,
      index,
      defaultNotebook,
      lockSession,
      lockDevice,
      lock,
      biometrics,
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
    BlocProvider<LockStatusBloc>.value(value: lock),
    BlocProvider<BiometricsBloc>.value(value: biometrics),
  ];

  /// [child] with them above it.
  Widget provide(Widget child) =>
      MultiBlocProvider(providers: providers, child: child);
}
