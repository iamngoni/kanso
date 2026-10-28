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
    kotlinOut:
        'android/src/main/kotlin/app/kanso/kanso_background/Messages.g.kt',
    kotlinOptions: KotlinOptions(package: 'app.kanso.kanso_background'),
    swiftOut: 'ios/kanso_background/Sources/kanso_background/Messages.g.swift',
  ),
)
@HostApi()
abstract class BackgroundHostApi {
  /// Runs the background sync once, now, by the same native path the
  /// system's job takes, and returns its result code
  /// (`mobile/rust/src/background.rs`).
  @async
  int runNow();

  /// Asks the system for the next background sync.
  void schedule();
}
