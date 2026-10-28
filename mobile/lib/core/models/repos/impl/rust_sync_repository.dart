// Kanso — rust_sync_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/sync.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/sync_repository.dart';

/// [SyncRepository] over the Rust core's sync.
@LazySingleton(as: SyncRepository)
class RustSyncRepository implements SyncRepository {
  RustSyncRepository(this._sync);

  final KansoSync _sync;

  @override
  Future<Either<ApplicationError, String>> defaultServer() =>
      attempt(KansoSync.defaultServer);

  @override
  Future<Either<ApplicationError, SyncStatus>> status() =>
      attempt(_sync.status);

  @override
  Future<Either<ApplicationError, ServerInfo>> checkServer(String url) =>
      attempt(() => _sync.checkServer(url: url));

  @override
  Future<Either<ApplicationError, SyncStatus>> signInPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  }) => attempt(
    () => _sync.signInPassword(
      server: server,
      username: username,
      password: password,
      deviceName: deviceName,
    ),
  );

  @override
  Future<Either<ApplicationError, Unit>> signOut() => attempt(() async {
    await _sync.signOut();
    return unit;
  });

  @override
  Future<Either<ApplicationError, SyncReportRow>> syncNow() =>
      attempt(_sync.syncNow);

  @override
  Stream<SyncStatus> statusStream() => cancelableStream(_sync.statusStream());

  @override
  Future<Either<ApplicationError, Unit>> setForeground({
    required bool foreground,
  }) => attempt(() async {
    await _sync.setForeground(foreground: foreground);
    return unit;
  });

  @override
  Future<Either<ApplicationError, SyncSettingsRow>> settings() =>
      attempt(_sync.settings);

  @override
  Future<Either<ApplicationError, SyncSettingsRow>> setSettings({
    required bool enabled,
    required int intervalSecs,
  }) => attempt(
    () => _sync.setSettings(enabled: enabled, intervalSecs: intervalSecs),
  );

  @override
  List<SyncIntervalOption> intervalOptions() => KansoSync.intervalOptions();

  @override
  SyncStatusLine statusLine(
    SyncStatus status,
    SyncSettingsRow settings, {
    required String when,
    required String resume,
  }) => KansoSync.statusLine(
    status: status,
    settings: settings,
    when: when,
    resume: resume,
  );
}
