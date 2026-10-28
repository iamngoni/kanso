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
        'android/src/main/kotlin/app/kanso/kanso_auth_session/Messages.g.kt',
    kotlinOptions: KotlinOptions(package: 'app.kanso.kanso_auth_session'),
    swiftOut:
        'ios/kanso_auth_session/Sources/kanso_auth_session/Messages.g.swift',
  ),
)
@HostApi()
abstract class AuthSessionHostApi {
  /// Opens [url] (http or https only) in the system's sign-in session.
  ///
  /// Completes with the URL the session ended at when it ends at
  /// [callbackScheme], and with null when the person dismissed it or [close]
  /// ended it.
  @async
  String? open(String url, String? callbackScheme);

  /// Ends the session, if one is open, and brings the app back in front.
  void close();
}
