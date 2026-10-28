// Kanso — kanso_background.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'src/messages.g.dart';

/// The sync that runs with no app: `BGAppRefreshTask` on iOS, WorkManager on
/// Android.
///
/// The system's job is set up natively (iOS: `KansoBackground.register()` in
/// `AppDelegate`; Android: `KansoApplication.onCreate`), so nothing here is
/// needed for it to run. This is for debugging and tests, and for asking the
/// system for the next run.
abstract final class KansoBackground {
  /// The channel; tests give their own messenger through [debugApi].
  static BackgroundHostApi _api = BackgroundHostApi();

  /// Replaces the channel. For tests.
  static set debugApi(BackgroundHostApi api) => _api = api;

  /// Runs the background sync once, now, by the same native path the
  /// system's job takes, and returns its code (`0` nothing changed, `1`
  /// something changed, `-1` signed out, `-2` the secret store refused, `-3`
  /// the server, `-4` out of time, `-5` internal).
  static Future<int> runNow() => _api.runNow();

  /// Asks the system for the next background sync. The native side already
  /// does this when the app goes to the background.
  static Future<void> schedule() => _api.schedule();
}
