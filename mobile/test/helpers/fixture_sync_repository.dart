// Kanso — fixture_sync_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/sync_repository.dart';

/// The status of a phone with no account.
const SyncStatus signedOutStatus = SyncStatus(
  phase: SyncPhase.signedOut,
  conflicts: 0,
  skippedCount: 0,
  account: SyncAccount.signedOut(),
);

/// The status of a phone signed in to an account, with nothing to do.
const SyncStatus signedInStatus = SyncStatus(
  phase: SyncPhase.idle,
  username: 'ana',
  server: 'https://example.invalid',
  conflicts: 0,
  skippedCount: 0,
  account: SyncAccount.signedIn(),
);

/// A sync service for widget tests: it has the status a test gives it,
/// counts the syncs it is asked for, and can hold one until the test lets
/// it go.
class FixtureSyncRepository implements SyncRepository {
  FixtureSyncRepository({this.current = signedOutStatus});

  /// The status the account has.
  SyncStatus current;

  /// How many times a sync was asked for.
  int syncs = 0;

  /// When set, a sync waits for it.
  Completer<void>? gate;

  @override
  Future<Either<ApplicationError, SyncReportRow>> syncNow() async {
    syncs++;
    await gate?.future;
    return const Right<ApplicationError, SyncReportRow>(
      SyncReportRow(
        pulled: 0,
        pushed: 0,
        conflicts: 0,
        pending: 0,
        queued: false,
      ),
    );
  }

  @override
  Stream<SyncStatus> statusStream() => Stream<SyncStatus>.value(current);

  /// Whether the app was last said to be in front.
  bool? foreground;

  @override
  Future<Either<ApplicationError, Unit>> setForeground({
    required bool foreground,
  }) async {
    this.foreground = foreground;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, SyncStatus>> status() async =>
      Right<ApplicationError, SyncStatus>(current);

  @override
  Future<Either<ApplicationError, String>> defaultServer() async =>
      const Right<ApplicationError, String>('https://sync.kansoapp.app');

  @override
  Future<Either<ApplicationError, ServerInfo>> checkServer(String url) =>
      throw UnimplementedError();

  @override
  Future<Either<ApplicationError, SyncStatus>> signInPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  }) => throw UnimplementedError();

  @override
  Future<Either<ApplicationError, Unit>> signOut() =>
      throw UnimplementedError();

  /// Whether Kanso syncs on its own, and how often.
  SyncSettingsRow syncSettings = const SyncSettingsRow(
    enabled: true,
    intervalSecs: 300,
  );

  /// When set, a change of the settings fails with it.
  ApplicationError? settingsError;

  @override
  Future<Either<ApplicationError, SyncSettingsRow>> settings() async =>
      Right<ApplicationError, SyncSettingsRow>(syncSettings);

  @override
  Future<Either<ApplicationError, SyncSettingsRow>> setSettings({
    required bool enabled,
    required int intervalSecs,
  }) async {
    final error = settingsError;
    if (error != null) {
      return Left<ApplicationError, SyncSettingsRow>(error);
    }
    syncSettings = SyncSettingsRow(
      enabled: enabled,
      intervalSecs: intervalSecs,
    );
    return Right<ApplicationError, SyncSettingsRow>(syncSettings);
  }

  @override
  List<SyncIntervalOption> intervalOptions() => fixtureIntervals;

  /// The desktop's words for the common cases, so pictures read true.
  @override
  SyncStatusLine statusLine(
    SyncStatus status,
    SyncSettingsRow settings, {
    required String when,
    required String resume,
  }) {
    if (status.phase == SyncPhase.signedOut) {
      return const SyncStatusLine(
        tone: SyncTone.quiet,
        label: 'Sync is off',
        detail:
            'Your notes are only on this device. With a free account they '
            'sync to all your devices and are backed up, locked so only you '
            'can read them.',
      );
    }
    if (!settings.enabled) {
      return const SyncStatusLine(
        tone: SyncTone.quiet,
        label: 'Sync off',
        detail: 'Turn on Sync automatically, or sync once with Sync now.',
      );
    }
    final minutes = settings.intervalSecs ~/ 60;
    final every = switch (minutes) {
      1 => 'every minute',
      60 => 'every hour',
      _ when minutes % 60 == 0 => 'every ${minutes ~/ 60} hours',
      _ => 'every $minutes minutes',
    };
    return SyncStatusLine(
      tone: SyncTone.good,
      label: 'Synced $when',
      detail: 'Kanso syncs $every and soon after you edit.',
    );
  }
}

/// The desktop's choices for how often sync runs.
const List<SyncIntervalOption> fixtureIntervals = <SyncIntervalOption>[
  SyncIntervalOption(minutes: 1, label: '1 minute'),
  SyncIntervalOption(minutes: 5, label: '5 minutes'),
  SyncIntervalOption(minutes: 15, label: '15 minutes'),
  SyncIntervalOption(minutes: 30, label: '30 minutes'),
  SyncIntervalOption(minutes: 60, label: '1 hour'),
  SyncIntervalOption(minutes: 240, label: '4 hours'),
];
