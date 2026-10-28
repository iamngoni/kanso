// Kanso — sync_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The account and sync. The session and the data key stay in the phone's
/// secret store, read by the Rust core.
abstract interface class SyncRepository {
  /// The server a new account is made on unless another is typed.
  Future<Either<ApplicationError, String>> defaultServer();

  Future<Either<ApplicationError, SyncStatus>> status();

  /// Asks the server at [url] whether it is a Kanso server.
  Future<Either<ApplicationError, ServerInfo>> checkServer(String url);

  Future<Either<ApplicationError, SyncStatus>> signInPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  });

  Future<Either<ApplicationError, Unit>> signOut();

  Future<Either<ApplicationError, SyncReportRow>> syncNow();

  /// The status now and each time it changes.
  Stream<SyncStatus> statusStream();

  /// The app is in front (true) or not. In front, sync runs after an edit and
  /// every so often; behind, it stops.
  Future<Either<ApplicationError, Unit>> setForeground({
    required bool foreground,
  });

  /// Whether Kanso syncs on its own, and how often.
  Future<Either<ApplicationError, SyncSettingsRow>> settings();

  /// Sets whether Kanso syncs on its own and how often; [intervalSecs] is
  /// one of [intervalOptions] in seconds. The schedule follows at once.
  Future<Either<ApplicationError, SyncSettingsRow>> setSettings({
    required bool enabled,
    required int intervalSecs,
  });

  /// How often sync may run, in the desktop's words.
  List<SyncIntervalOption> intervalOptions();

  /// The status line the desktop shows for [status] under [settings].
  /// [when] says when the last sync was ("2 min ago"), [resume] when a
  /// paused sync starts again ("at 14:05").
  SyncStatusLine statusLine(
    SyncStatus status,
    SyncSettingsRow settings, {
    required String when,
    required String resume,
  });
}
