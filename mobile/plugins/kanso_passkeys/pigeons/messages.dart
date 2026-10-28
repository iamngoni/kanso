// Kanso — messages.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:pigeon/pigeon.dart';

/// The typed channel to the native side. Regenerate with
/// `dart run pigeon --input pigeons/messages.dart` in this folder; the
/// generated files are committed.
@ConfigurePigeon(
  PigeonOptions(
    dartOut: 'lib/src/messages.g.dart',
    kotlinOut: 'android/src/main/kotlin/app/kanso/kanso_passkeys/Messages.g.kt',
    kotlinOptions: KotlinOptions(package: 'app.kanso.kanso_passkeys'),
    swiftOut: 'ios/kanso_passkeys/Sources/kanso_passkeys/Messages.g.swift',
  ),
)
/// How a ceremony ended, as far as Dart is told. The answer itself (with the
/// PRF output) never comes here: the plugin hands it to Rust.
enum PasskeyOutcome {
  /// The sheet answered and Rust took the answer.
  done,

  /// The person dismissed the sheet.
  cancelled,

  /// This phone, its credential provider or the app's setup can't run it.
  unavailable,

  /// Anything else.
  failed,
}

@HostApi()
abstract class PasskeysHostApi {
  /// Runs the platform's passkey sheet for [ticket].
  ///
  /// [kind] is `register` or `assert`; [optionsJson] is the non-secret
  /// request of the facade's `PasskeyRequest`. The plugin passes the
  /// answer, or `{"error": ...}`, to Rust itself.
  @async
  PasskeyOutcome run(int ticket, String kind, String optionsJson);
}
