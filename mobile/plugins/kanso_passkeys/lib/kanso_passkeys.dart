// Kanso — kanso_passkeys.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'src/messages.g.dart';

export 'src/messages.g.dart' show PasskeyOutcome;

/// The platform's passkey sheet (`ASAuthorizationController` on iOS,
/// Credential Manager on Android).
///
/// The plugin passes the sheet's answer, with the PRF output, straight to
/// Rust (`kanso_passkey_answer`); Dart only learns how the ceremony ended.
/// When it did not end well the plugin has also told Rust (`{"error":…}`), so
/// the ceremony never waits on a lost answer.
abstract final class KansoPasskeys {
  static PasskeysHostApi _api = PasskeysHostApi();

  /// Replaces the channel. For tests.
  static set debugApi(PasskeysHostApi api) => _api = api;

  /// Runs the sheet for [ticket]. [kind] is `register` or `assert`
  /// (`PasskeyRequestKind`); [optionsJson] is the request's `options_json`.
  ///
  /// Completes with [PasskeyOutcome.done] once Rust took the answer, and
  /// otherwise with why not: the caller reports that with `passkey_failed`.
  static Future<PasskeyOutcome> run(
    int ticket,
    String kind,
    String optionsJson,
  ) => _api.run(ticket, kind, optionsJson);
}
